/*! Derivation of [`Serialize`][serialize] and [`Deserialize`][deserialize] that replaces struct keys with numerical indices.

### Usage example

#### Struct attributes

- `auto_index`: Automatically assign indices to the fields based on the order in the source code.  It is recommended to instead use the `index` attribute for all fields to explicitly assign indices.
- `offset = ?`: If `auto_index` is set, use the given index for the first field instead of starting with zero.

### Field attributes

- `index = ?`: Set the index for this field to the given field.  This attribute is required unless `auto_index` is set.  It cannot be used together with `auto_index`.
- `skip`: Never serialize or deserialize this field.  This field still increases the assigned index if `auto_index` is used.
- `skip(no_increment)`: Never serialize or deserialize this field and don’t increment the assigned index for this field if used together with the `auto_index` attribute.

`serde-indexed` also supports these `serde` attributes:
- [`deserialize_with`][deserialize-with]
- [`serialize_with`][serialize-with]
- [`skip_serializing_if`][skip-serializing-if]
- [`with`][with]

### Generated code example
`cargo expand --test basics` exercises the macros using [`serde_cbor`][serde-cbor].

### Examples

Explicit index assignment:

```
use serde_indexed::{DeserializeIndexed, SerializeIndexed};

#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
pub struct SomeKeys {
    #[serde(index = 1)]
    pub number: i32,
    #[serde(index = 2)]
    pub option: Option<u8>,
    #[serde(skip)]
    pub ignored: bool,
    #[serde(index = 3)]
    pub bytes: [u8; 7],
}
```

Automatic index assignment:

```
use serde_indexed::{DeserializeIndexed, SerializeIndexed};

#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
#[serde(auto_index)]
pub struct SomeKeys {
    // index 1
    pub number: i32,
    // index 2
    pub option: Option<u8>,
    // index 3 (but skipped)
    #[serde(skip)]
    pub ignored: bool,
    // index 4
    pub bytes: [u8; 7],
}
```

Automatic index assignment with `skip(no_increment)`:

```
use serde_indexed::{DeserializeIndexed, SerializeIndexed};

#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
#[serde(auto_index)]
pub struct SomeKeys {
    // index 1
    pub number: i32,
    // index 2
    pub option: Option<u8>,
    #[serde(skip(no_increment))]
    pub ignored: bool,
    // index 3
    pub bytes: [u8; 7],
}
```

Automatic index assignment with `offset`:

```
use serde_indexed::{DeserializeIndexed, SerializeIndexed};

#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
#[serde(auto_index, offset = 42)]
pub struct SomeKeys {
    // index 42
    pub number: i32,
    // index 43
    pub option: Option<u8>,
    // index 44
    pub bytes: [u8; 7],
}
```

Skip serializing a field based on a condition with `skip_serializing_if`:

```
use serde_indexed::{DeserializeIndexed, SerializeIndexed};

#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
pub struct SomeKeys {
    #[serde(index = 1)]
    pub number: i32,
    #[serde(index = 2, skip_serializing_if = "Option::is_none")]
    pub option: Option<u8>,
    #[serde(index = 3)]
    pub bytes: [u8; 7],
}
```

Change the serialization or deserialization format with `deserialize_with`, `serialize_with` or `with`:

```
use serde_indexed::{DeserializeIndexed, SerializeIndexed};

#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
pub struct SomeKeys<'a> {
    #[serde(index = 1, serialize_with = "serde_bytes::serialize")]
    pub one: &'a [u8],
    #[serde(index = 2, deserialize_with = "serde_bytes::deserialize")]
    pub two: &'a [u8],
    #[serde(index = 3, with = "serde_bytes")]
    pub three: &'a [u8],
}
```

[serialize]: https://docs.serde.rs/serde/ser/trait.Serialize.html
[deserialize]: https://docs.serde.rs/serde/de/trait.Deserialize.html
[deserialize-with]: https://serde.rs/field-attrs.html#deserialize_with
[serialize-with]: https://serde.rs/field-attrs.html#serialize_with
[with]: https://serde.rs/field-attrs.html#with
[skip-serializing-if]: https://serde.rs/field-attrs.html#skip_serializing_if
[serde-cbor]: https://docs.rs/serde_cbor
*/

extern crate proc_macro;

mod parse;

use parse::Skip;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, ImplGenerics, Lifetime, LifetimeParam, TypeGenerics, TypeParamBound,
    WhereClause,
};

use crate::parse::Input;

fn serialize_fields(
    fields: &[parse::Field],
    offset: usize,
    impl_generics_serialize: ImplGenerics<'_>,
    ty_generics_serialize: TypeGenerics<'_>,
    ty_generics: &TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    ident: &Ident,
) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter(|field| !field.skip_serializing_if.is_always())
        .map(|field| {
            // index should only be none if the field is always skipped, so this should never panic
            let index = field.index.expect("index must be set for fields that are not skipped") + offset;
            let member = &field.member;
            let serialize_member = match &field.serialize_with {
                None => quote!(&self.#member),
                Some(f) => {
                    let ty = &field.ty;
                    quote!({
                            struct __InternalSerdeIndexedSerializeWith #impl_generics_serialize {
                                value: &'__serde_indexed_lifetime #ty,
                                phantom: ::core::marker::PhantomData<#ident #ty_generics>,
                            }

                            impl #impl_generics_serialize serde::Serialize for __InternalSerdeIndexedSerializeWith #ty_generics_serialize #where_clause {
                                fn serialize<__S>(
                                    &self,
                                    __s: __S,
                                ) -> ::core::result::Result<__S::Ok, __S::Error>
                                where
                                    __S: serde::Serializer,
                                {
                                    #f(self.value, __s)
                                }
                            }

                            &__InternalSerdeIndexedSerializeWith { value: &self.#member, phantom: ::core::marker::PhantomData::<#ident #ty_generics> }
                    })
                }
            };

            // println!("field {:?} index {:?}", &field.label, field.index);
            match &field.skip_serializing_if {
                Skip::If(path) => quote! {
                    if !#path(&self.#member) {
                        map.serialize_entry(&#index, #serialize_member)?;
                    }
                },
                Skip::Always => unreachable!(),
                Skip::Never => quote! {
                    map.serialize_entry(&#index, #serialize_member)?;
                },
            }
        })
        .collect()
}

fn count_serialized_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            // let index = field.index + offset;
            let member = &field.member;
            match &field.skip_serializing_if {
                Skip::If(path) => {
                    quote! { if #path(&self.#member) { 0 } else { 1 } }
                }
                Skip::Always => quote! { 0 },

                Skip::Never => {
                    quote! { 1 }
                }
            }
        })
        .collect()
}

#[proc_macro_derive(SerializeIndexed, attributes(serde, serde_indexed))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let ident = input.ident;
    let num_fields = count_serialized_fields(&input.fields);
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut generics_cl = input.generics.clone();
    generics_cl.type_params_mut().for_each(|t| {
        t.bounds
            .push_value(TypeParamBound::Verbatim(quote!(serde::Serialize)));
    });
    let (impl_generics, _, _) = generics_cl.split_for_impl();

    let mut generics_cl2 = generics_cl.clone();

    generics_cl2
        .params
        .push(syn::GenericParam::Lifetime(LifetimeParam::new(
            Lifetime::new("'__serde_indexed_lifetime", Span::call_site()),
        )));

    let (impl_generics_serialize, ty_generics_serialize, _) = generics_cl2.split_for_impl();

    let serialize_fields = serialize_fields(
        &input.fields,
        input.attrs.offset,
        impl_generics_serialize,
        ty_generics_serialize,
        &ty_generics,
        where_clause,
        &ident,
    );

    TokenStream::from(quote! {
        #[automatically_derived]
        impl #impl_generics serde::Serialize for #ident #ty_generics #where_clause  {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                use serde::ser::SerializeMap;
                let num_fields = 0 #( + #num_fields)*;
                let mut map = serializer.serialize_map(Some(num_fields))?;

                #(#serialize_fields)*

                map.end()
            }
        }
    })
}

fn none_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter(|f| !f.skip_serializing_if.is_always())
        .map(|field| {
            let ident = format_ident!("{}", &field.label);
            let span = field.original_span;
            quote_spanned! { span =>
                let mut #ident = None;
            }
        })
        .collect()
}

fn unwrap_expected_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let label = field.label.clone();
            let ident = format_ident!("{}", &field.label);
            let span = field.original_span;
            match field.skip_serializing_if {
                Skip::Never => quote! {
                    let #ident = #ident.ok_or_else(|| serde::de::Error::missing_field(#label))?;
                },
                Skip::If(_) => quote_spanned! { span =>
                    let #ident = #ident.unwrap_or_default();
                },
                Skip::Always => quote! {
                    let #ident = ::core::default::Default::default();
                },
            }
        })
        .collect()
}

fn match_fields(
    fields: &[parse::Field],
    offset: usize,
    impl_generics_with_de: &ImplGenerics<'_>,
    ty_generics: &TypeGenerics<'_>,
    ty_generics_with_de: &TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    struct_ident: &Ident,
) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter(|f| !f.skip_serializing_if.is_always())
        .map(|field| {
            let label = field.label.clone();
            let ident = format_ident!("{}", &field.label);
            // index should only be none if the field is always skipped, so this should never panic
            let index = field.index.expect("index must be set for fields that are not skipped") + offset;
            let span = field.original_span;

            let next_value = match &field.deserialize_with {
                Some(f) => {
                    let ty = &field.ty;
                    quote_spanned!(span => {
                            struct __InternalSerdeIndexedDeserializeWith #impl_generics_with_de {
                                value: #ty,
                                phantom: ::core::marker::PhantomData<#struct_ident #ty_generics>,
                                lifetime: ::core::marker::PhantomData<&'de ()>,
                            }
                            impl #impl_generics_with_de serde::Deserialize<'de> for __InternalSerdeIndexedDeserializeWith #ty_generics_with_de #where_clause {
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> Result<Self, __D::Error>
                                where
                                    __D: serde::Deserializer<'de>,
                                {

                                    Ok(__InternalSerdeIndexedDeserializeWith {
                                        value: #f(__deserializer)?,
                                        phantom: ::core::marker::PhantomData,
                                        lifetime: ::core::marker::PhantomData,
                                    })
                                }
                            }

                            let __InternalSerdeIndexedDeserializeWith { value, lifetime: _, phantom: _ } = map.next_value()?;
                            value
                        }
                    )
                }
                None => quote_spanned!(span => map.next_value()?),
            };

            quote_spanned!{ span =>
                #index => {
                    if #ident.is_some() {
                        return Err(serde::de::Error::duplicate_field(#label));
                    }
                    let next_value = #next_value;
                    #ident = Some(next_value);
                },
            }
        })
        .collect()
}

fn all_fields(fields: &[parse::Field]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ident = format_ident!("{}", &field.label);
            let span = field.original_span;
            quote_spanned! { span =>
                #ident
            }
        })
        .collect()
}

#[proc_macro_derive(DeserializeIndexed, attributes(serde, serde_indexed))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let ident = input.ident;
    let none_fields = none_fields(&input.fields);
    let unwrap_expected_fields = unwrap_expected_fields(&input.fields);
    let all_fields = all_fields(&input.fields);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut generics_cl = input.generics.clone();
    generics_cl.params.insert(
        0,
        syn::GenericParam::Lifetime(LifetimeParam {
            attrs: Vec::new(),
            lifetime: Lifetime {
                apostrophe: Span::call_site(),
                ident: Ident::new("de", Span::call_site()),
            },
            colon_token: None,
            bounds: input
                .generics
                .lifetimes()
                .map(|l| l.lifetime.clone())
                .collect(),
        }),
    );
    generics_cl.type_params_mut().for_each(|t| {
        t.bounds
            .push_value(TypeParamBound::Verbatim(quote!(serde::Deserialize<'de>)));
    });

    let (impl_generics_with_de, ty_generics_with_de, _) = generics_cl.split_for_impl();

    let match_fields = match_fields(
        &input.fields,
        input.attrs.offset,
        &impl_generics_with_de,
        &ty_generics,
        &ty_generics_with_de,
        where_clause,
        &ident,
    );

    let the_loop = if !input.fields.is_empty() {
        // NB: In the previous "none_fields", we use the actual struct's
        // keys as variable names. If the struct happens to have a key
        // named "key", it would clash with __serde_indexed_internal_key,
        // if that were named key.
        quote! {
            while let Some(__serde_indexed_internal_key) = map.next_key()? {
                match __serde_indexed_internal_key {
                    #(#match_fields)*
                    _ => {
                        // Ignore unknown keys by consuming their value
                        let _ = map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let res = quote! {
        #[automatically_derived]
        impl #impl_generics_with_de serde::Deserialize<'de> for #ident #ty_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct IndexedVisitor #impl_generics (core::marker::PhantomData<#ident #ty_generics>);

                impl #impl_generics_with_de serde::de::Visitor<'de> for IndexedVisitor #ty_generics {
                    type Value = #ident #ty_generics;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str(stringify!(#ident))
                    }

                    fn visit_map<V>(self, mut map: V) -> core::result::Result<Self::Value, V::Error>
                    where
                        V: serde::de::MapAccess<'de>,
                    {
                        #(#none_fields)*

                        #the_loop

                        #(#unwrap_expected_fields)*

                        Ok(#ident { #(#all_fields),* })
                    }
                }

                deserializer.deserialize_map(IndexedVisitor(Default::default()))
            }
        }
    };
    TokenStream::from(res)
}
