use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Data, DeriveInput, Fields, Generics, Ident, Token};

pub struct Input {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: StructAttrs,
    pub fields: Vec<Field>,
}

#[derive(Default)]
pub struct StructAttrs {
    pub offset: usize,
    // pub skip_nones: bool,
}

pub struct Field {
    pub label: String,
    pub member: syn::Member,
    pub index: usize,
    pub skip_serializing_if: Option<syn::ExprPath>,
    // pub attrs: attr::Field,
    pub ty: syn::Type,
    pub original: syn::Field,
}

fn parse_meta(attrs: &mut StructAttrs, meta: &syn::Meta) -> Result<()> {
    if let syn::Meta::List(value) = meta {
        for meta in &value.nested {
            match meta {
                syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                    if name_value.path.is_ident("offset") {
                        if let syn::Lit::Int(offset) = &name_value.lit {
                            attrs.offset = offset.base10_parse()?;
                            // println!("shall use offset {}", attrs.offset);
                        }
                    }
                }
                // This `skip_nones` approach is tricky, as then we
                // need to detect Option types, which means a lot of path
                // manipulation, possibly in vain.
                //
                // syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                //     if path.is_ident("skip_nones") {
                //         // println!("shall skip nones");
                //         attrs.skip_nones = true;
                //     }
                // },
                _ => {}
            }
        }
    }

    Ok(())
}

fn parse_attrs(attrs: &Vec<syn::Attribute>) -> Result<StructAttrs> {
    let mut struct_attrs: StructAttrs = Default::default();

    for attr in attrs {
        if attr.path.is_ident("serde_indexed") {
            // println!("parsing serde_indexed");
            parse_meta(&mut struct_attrs, &attr.parse_meta()?)?;
        }
        if attr.path.is_ident("serde") {
            // println!("parsing serde");
            parse_meta(&mut struct_attrs, &attr.parse_meta()?)?;
        }
    }

    Ok(struct_attrs)
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let call_site = Span::call_site();
        let derive_input = DeriveInput::parse(input)?;

        let data: syn::DataStruct = match derive_input.data {
            Data::Struct(data) => data,
            _ => {
                return Err(Error::new(call_site, "input must be a struct"));
            }
        };

        let attrs: StructAttrs = parse_attrs(&derive_input.attrs)?;

        let syn_fields: syn::FieldsNamed = match data.fields {
            Fields::Named(named_fields) => named_fields,
            _ => {
                return Err(Error::new(call_site, "struct fields must be named"));
            }
        };

        let fields = fields_from_ast(&syn_fields.named);

        //serde::internals::ast calls `fields_from_ast(cx, &fields.named, attrs, container_default)`

        Ok(Input {
            ident: derive_input.ident,
            attrs,
            fields,
            generics: derive_input.generics,
        })
    }
}

fn fields_from_ast<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, Token![,]>,
) -> Vec<Field> {
    // serde::internals::ast.rs:L183
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            // these are https://docs.rs/syn/1.0.13/syn/struct.Field.html
            label: match &field.ident {
                Some(ident) => ident.to_string(),
                None => {
                    // TODO: does this happen?
                    panic!("input struct must have named fields");
                }
            },
            member: match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => {
                    // TODO: does this happen?
                    panic!("input struct must have named fields");
                }
            },
            index: i,
            // TODO: make this... more concise? handle errors? the thing with the spans?
            skip_serializing_if: {
                let mut skip_serializing_if = None;
                for attr in &field.attrs {
                    if attr.path.is_ident("serde") {
                        if let Ok(syn::Meta::List(value)) = attr.parse_meta() {
                            for meta in &value.nested {
                                match meta {
                                    syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                                        if name_value.path.is_ident("skip_serializing_if") {
                                            // println!("so close!");
                                            if let syn::Lit::Str(litstr) = &name_value.lit {
                                                let tokens =
                                                    syn::parse_str(&litstr.value()).unwrap();
                                                // println!("found something: {:?}", &litstr.value());
                                                skip_serializing_if =
                                                    Some(syn::parse2(tokens).unwrap());
                                            }
                                        } else {
                                            // safety net, remove?
                                            panic!("unknown field attribute");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                skip_serializing_if
            },
            ty: field.ty.clone(),
            original: field.clone(),
        })
        .collect()
}
