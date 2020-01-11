use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Data, DeriveInput, Fields, Ident, Token};

pub struct Input {
    pub ident: Ident,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub label: String,
    pub member: syn::Member,
    pub index: usize,
    // pub attrs: attr::Field,
    pub ty: syn::Type,
    pub original: syn::Field,
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
            fields,
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
            ty: field.ty.clone(),
            original: field.clone(),
        })
        .collect()
}
