//! Derive `Serialize` and `Deserialize` that replaces struct keys with numerical indices.

extern crate proc_macro;

mod parse;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::parse::Input;

fn serialize_visitor(fields: &[parse::Field], offset: usize) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
             let index = field.index + offset;
             // let label = stringify!(field.label.clone());
             let member = &field.member;
             quote! {
                 // TODO: how do I get something like `self.#label` here?
                 // TODO: what the heck is "member"?
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
    let serialize_fields = serialize_visitor(&input.fields, 1);

    TokenStream::from(quote! {
        impl serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                // Somehow:
                // - serialize_struct takes an identifier
                // - serialize_map does *not*
                // let mut state = serializer.serialize_struct(stringify!(#ident), #num_fields)?;

                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(#num_fields))?;

                #(#serialize_fields)*

                map.end()
            }
        }
    })
}

