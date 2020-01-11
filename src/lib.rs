//! Derive `Serialize` and `Deserialize` that replaces struct keys with numerical indices.

extern crate proc_macro;

mod parse;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::parse::Input;

// TODO: make this configurable with container attribute
const OFFSET: usize = 1;

fn serialize_visitor(fields: &[parse::Field], offset: usize) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let index = field.index + offset;
            let member = &field.member;
            quote! {
                map.serialize_entry(&#index, &self.#member)?;
            }
        })
        .collect()
}

#[proc_macro_derive(SerializeIndexed)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let ident = input.ident;
    let num_fields = input.fields.len();
    let serialize_fields = serialize_visitor(&input.fields, OFFSET);

    TokenStream::from(quote! {
        impl serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(#num_fields))?;

                #(#serialize_fields)*

                map.end()
            }
        }
    })
}

fn none_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ident = syn::Ident::new(&field.label, proc_macro2::Span::call_site());
            quote! {
                let mut #ident = None;
            }
        })
        .collect()
}

fn unwrap_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let label = stringify!(field.label.clone());
            let ident = syn::Ident::new(&field.label, proc_macro2::Span::call_site());
            quote! {
                let #ident = #ident.ok_or_else(|| serde::de::Error::missing_field(#label))?;
            }
        })
        .collect()
}

fn match_fields(fields: &[parse::Field], offset: usize) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let label = stringify!(field.label.clone());
            let ident = syn::Ident::new(&field.label, proc_macro2::Span::call_site());
            let index = field.index + offset;
            quote! {
                #index => {
                    if #ident.is_some() {
                        return Err(serde::de::Error::duplicate_field(#label));
                    }
                    #ident = Some(map.next_value()?);
                },
            }
        })
        .collect()
}

fn all_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ident = syn::Ident::new(&field.label, proc_macro2::Span::call_site());
            quote! {
                #ident
            }
        })
        .collect()
}

#[proc_macro_derive(DeserializeIndexed)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let ident = input.ident;
    let none_fields = none_fields(&input.fields);
    let unwrap_fields = unwrap_fields(&input.fields);
    let match_fields = match_fields(&input.fields, OFFSET);
    let all_fields = all_fields(&input.fields);

    TokenStream::from(quote! {
        impl<'de> serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct IndexedVisitor;

                impl<'de> serde::de::Visitor<'de> for IndexedVisitor {
                    type Value = #ident;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str("struct #ident")
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<#ident, V::Error>
                    where
                        V: serde::de::MapAccess<'de>,
                    {
                        #(#none_fields)*

                        while let Some(key) = map.next_key()? {
                            match key {
                                #(#match_fields)*
                                _ => {
                                    return Err(serde::de::Error::duplicate_field("inexistent field index"));
                                }
                            }
                        }

                        #(#unwrap_fields)*

                        Ok(#ident { #(#all_fields),* })
                    }
                }

                deserializer.deserialize_map(IndexedVisitor {})
            }
        }
    })
}
