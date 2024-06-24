use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Data, DeriveInput, Fields, Generics, Ident, LitInt, LitStr, Token};

pub struct Input {
    pub ident: Ident,
    pub attrs: StructAttrs,
    pub fields: Vec<Field>,
    pub generics: Generics,
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
    pub serialize_with: Option<syn::ExprPath>,
    pub deserialize_with: Option<syn::ExprPath>,
    // pub attrs: attr::Field,
    pub ty: syn::Type,
    pub original: syn::Field,
}

fn parse_meta(attrs: &mut StructAttrs, meta: ParseNestedMeta) -> Result<()> {
    if meta.path.is_ident("offset") {
        let value = meta.value()?;
        let offset: LitInt = value.parse()?;
        attrs.offset = offset.base10_parse()?;
        Ok(())
    } else {
        Err(meta.error(format_args!(
            "the only accepted struct level attribute is offset"
        )))
    }
}

fn parse_attrs(attrs: &Vec<syn::Attribute>) -> Result<StructAttrs> {
    let mut struct_attrs: StructAttrs = Default::default();

    for attr in attrs {
        if attr.path().is_ident("serde_indexed") {
            attr.parse_nested_meta(|meta| parse_meta(&mut struct_attrs, meta))?;
            // println!("parsing serde_indexed");
            // parse_meta(&mut struct_attrs, &attr.parse_meta()?)?;
        }
        if attr.path().is_ident("serde") {
            // println!("parsing serde");
            attr.parse_nested_meta(|meta| parse_meta(&mut struct_attrs, meta))?;
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

        let fields = fields_from_ast(&syn_fields.named)?;

        //serde::internals::ast calls `fields_from_ast(cx, &fields.named, attrs, container_default)`

        Ok(Input {
            ident: derive_input.ident,
            attrs,
            fields,
            generics: derive_input.generics,
        })
    }
}

fn fields_from_ast(
    fields: &syn::punctuated::Punctuated<syn::Field, Token![,]>,
) -> Result<Vec<Field>> {
    // serde::internals::ast.rs:L183
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            // Parse attributes
            let mut skip_serializing_if = None;
            let mut serialize_with = None;
            let mut deserialize_with = None;

            for attr in &field.attrs {
                if attr.path().is_ident("serde") {
                    attr.parse_nested_meta(|meta| {
                        let parse_value = |attribute: &mut Option<_>, attribute_name: &str| {
                            let litstr: LitStr = meta.value()?.parse()?;
                            let tokens = syn::parse_str(&litstr.value())?;
                            if attribute.is_some() {
                                return Err(
                                    meta.error(format!("Multiple attributes for {attribute_name}"))
                                );
                            }
                            *attribute = Some(syn::parse2(tokens)?);
                            Ok(())
                        };
                        if meta.path.is_ident("skip_serializing_if") {
                            parse_value(&mut skip_serializing_if, "skip_serializing_if")
                        } else if meta.path.is_ident("deserialize_with") {
                            parse_value(&mut deserialize_with, "deserialize_with")
                        } else if meta.path.is_ident("serialize_with") {
                            parse_value(&mut serialize_with, "serialize_with")
                        } else if meta.path.is_ident("with") {
                            let litstr: LitStr = meta.value()?.parse()?;
                            if serialize_with.is_some() {
                                return Err(meta.error(format!(
                                    "Using `with` when `serialize_with` is already used"
                                )));
                            }
                            if deserialize_with.is_some() {
                                return Err(meta.error(format!(
                                    "Using `with` when `deserialize_with` is already used"
                                )));
                            }

                            let serialize_tokens =
                                syn::parse_str(&format!("{}::serialize", litstr.value()))?;
                            let deserialize_tokens =
                                syn::parse_str(&format!("{}::deserialize", litstr.value()))?;

                            serialize_with = Some(syn::parse2(serialize_tokens)?);
                            deserialize_with = Some(syn::parse2(deserialize_tokens)?);

                            Ok(())
                        } else {
                            return Err(meta.error("Unkown field attribute"));
                        }
                    })?;
                }
            }

            Ok(Field {
                // these are https://docs.rs/syn/2.0.28/syn/struct.Field.html
                label: match &field.ident {
                    Some(ident) => ident.to_string(),
                    None => {
                        return Err(Error::new_spanned(fields, "Tuple struct are not supported"));
                    }
                },
                member: match &field.ident {
                    Some(ident) => syn::Member::Named(ident.clone()),
                    None => {
                        return Err(Error::new_spanned(fields, "Tuple struct are not supported"));
                    }
                },
                index: i,
                // TODO: make this... more concise? handle errors? the thing with the spans?
                skip_serializing_if,
                serialize_with,
                deserialize_with,
                ty: field.ty.clone(),
                original: field.clone(),
            })
        })
        .collect()
}
