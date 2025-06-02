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

pub enum Skip {
    Never,
    If(syn::ExprPath),
    Always,
}

impl Skip {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::Never)
    }
    pub fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }
}

pub struct Field {
    pub label: String,
    pub member: syn::Member,
    pub index: usize,
    pub skip_serializing_if: Skip,
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
    let mut index = 0;
    fields
        .iter()
        .map(|field| {
            let current_index = index;
            index += 1;

            let mut skip_serializing_if = Skip::Never;
            for attr in &field.attrs {
                if attr.path().is_ident("serde") {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("skip_serializing_if") {
                            let litstr: LitStr = meta.value()?.parse()?;
                            let tokens = syn::parse_str(&litstr.value())?;
                            if !skip_serializing_if.is_none() {
                                return Err(meta
                                    .error("Multiple attributes for skip_serializing_if or skip"));
                            }
                            skip_serializing_if = Skip::If(syn::parse2(tokens)?);
                            Ok(())
                        } else if meta.path.is_ident("skip") {
                            if meta.input.peek(syn::token::Paren) {
                                meta.parse_nested_meta(|skip_meta| {
                                    if !skip_meta.path.is_ident("no_increment") {
                                        Err(skip_meta
                                            .error("`skip` only accepts `no_increment` as value"))
                                    } else {
                                        index -= 1;
                                        Ok(())
                                    }
                                })?;
                            }

                            if !skip_serializing_if.is_none() {
                                return Err(meta
                                    .error("Multiple attributes for skip_serializing_if or skip"));
                            }
                            skip_serializing_if = Skip::Always;
                            Ok(())
                        } else {
                            Err(meta.error("Unkown field attribute"))
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
                index: current_index,
                // TODO: make this... more concise? handle errors? the thing with the spans?
                skip_serializing_if,
            })
        })
        .collect()
}
