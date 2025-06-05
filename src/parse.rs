use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Generics, Ident, LitInt, LitStr, Token};

pub struct Input {
    pub ident: Ident,
    pub attrs: StructAttrs,
    pub fields: Vec<Field>,
    pub generics: Generics,
}

#[derive(Default)]
pub struct StructAttrs {
    pub auto_index: bool,
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
    pub index: Option<usize>,
    pub skip_serializing_if: Skip,
    pub serialize_with: Option<syn::ExprPath>,
    pub deserialize_with: Option<syn::ExprPath>,
    pub no_increment: bool,
    pub ty: syn::Type,
    pub original_span: Span,
}

fn parse_meta(attrs: &mut StructAttrs, meta: ParseNestedMeta) -> Result<()> {
    if meta.path.is_ident("auto_index") {
        attrs.auto_index = true;
        Ok(())
    } else if meta.path.is_ident("offset") {
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

        let fields = fields_from_ast(&attrs, &syn_fields.named)?;

        //serde::internals::ast calls `fields_from_ast(cx, &fields.named, attrs, container_default)`

        Ok(Input {
            ident: derive_input.ident,
            attrs,
            fields,
            generics: derive_input.generics,
        })
    }
}

fn parse_field(
    attrs: &StructAttrs,
    auto_index: usize,
    field: &syn::Field,
    indices: &mut Vec<usize>,
) -> Result<Field> {
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new_spanned(field, "Tuple structs are not supported"))?;

    let mut skip_serializing_if = Skip::Never;
    let mut deserialize_with = None;
    let mut serialize_with = None;
    let mut no_increment = false;
    let mut explicit_index = None;

    for attr in &field.attrs {
        if attr.path().is_ident("serde") {
            attr.parse_nested_meta(|meta| {
                let parse_value = |attribute: &mut Option<_>, attribute_name: &str| {
                    let litstr: LitStr = meta.value()?.parse()?;
                    let tokens = syn::parse_str(&litstr.value())?;
                    if attribute.is_some() {
                        return Err(meta.error(format!("Multiple attributes for {attribute_name}")));
                    }
                    *attribute = Some(syn::parse2(tokens)?);
                    Ok(())
                };

                if meta.path.is_ident("skip_serializing_if") {
                    let litstr: LitStr = meta.value()?.parse()?;
                    let tokens = syn::parse_str(&litstr.value())?;
                    if !skip_serializing_if.is_none() {
                        return Err(
                            meta.error("Multiple attributes for skip_serializing_if or skip")
                        );
                    }
                    skip_serializing_if = Skip::If(syn::parse2(tokens)?);
                    Ok(())
                } else if meta.path.is_ident("skip") {
                    if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|skip_meta| {
                            if !skip_meta.path.is_ident("no_increment") {
                                Err(skip_meta.error("`skip` only accepts `no_increment` as value"))
                            } else {
                                no_increment = true;
                                Ok(())
                            }
                        })?;
                    }

                    if !skip_serializing_if.is_none() {
                        return Err(
                            meta.error("Multiple attributes for skip_serializing_if or skip")
                        );
                    }
                    skip_serializing_if = Skip::Always;
                    Ok(())
                } else if meta.path.is_ident("deserialize_with") {
                    parse_value(&mut deserialize_with, "deserialize_with")
                } else if meta.path.is_ident("serialize_with") {
                    parse_value(&mut serialize_with, "serialize_with")
                } else if meta.path.is_ident("with") {
                    let litstr: LitStr = meta.value()?.parse()?;
                    if serialize_with.is_some() {
                        return Err(meta.error(
                            "Using `with` when `serialize_with` is already used".to_string(),
                        ));
                    }
                    if deserialize_with.is_some() {
                        return Err(meta.error(
                            "Using `with` when `deserialize_with` is already used".to_string(),
                        ));
                    }

                    let serialize_tokens =
                        syn::parse_str(&format!("{}::serialize", litstr.value()))?;
                    let deserialize_tokens =
                        syn::parse_str(&format!("{}::deserialize", litstr.value()))?;

                    serialize_with = Some(syn::parse2(serialize_tokens)?);
                    deserialize_with = Some(syn::parse2(deserialize_tokens)?);

                    Ok(())
                } else if meta.path.is_ident("index") {
                    if explicit_index.is_some() {
                        return Err(meta.error("Multiple attributes for index"));
                    }
                    if attrs.auto_index {
                        return Err(meta.error(
                            "The index attribute cannot be combined with the auto_index attribute",
                        ));
                    }
                    let litint: LitInt = meta.value()?.parse()?;
                    let int = litint.base10_parse()?;
                    if indices.contains(&int) {
                        return Err(meta.error("This index has already been assigned"));
                    }
                    explicit_index = Some(int);
                    Ok(())
                } else {
                    return Err(meta.error("Unkown field attribute"));
                }
            })?;
        }
    }

    if explicit_index.is_some() && skip_serializing_if.is_always() {
        return Err(Error::new_spanned(
            field,
            "`#[serde(index = ?]` and `#[serde(skip)]` cannot be combined",
        ));
    }

    let index = if skip_serializing_if.is_always() {
        None
    } else if attrs.auto_index {
        Some(auto_index)
    } else if let Some(index) = explicit_index {
        indices.push(index);
        Some(index)
    } else {
        return Err(Error::new_spanned(
            field,
            "Field without index or skip attribute and `#[serde(auto_index)]` is not enabled on the struct",
        ));
    };
    Ok(Field {
        label: ident.to_string(),
        member: syn::Member::Named(ident.clone()),
        index,
        ty: field.ty.clone(),
        skip_serializing_if,
        serialize_with,
        deserialize_with,
        no_increment,
        original_span: field.span(),
    })
}

fn fields_from_ast(
    attrs: &StructAttrs,
    fields: &syn::punctuated::Punctuated<syn::Field, Token![,]>,
) -> Result<Vec<Field>> {
    let mut indices = Vec::new();
    let mut index = 0;
    fields
        .iter()
        .map(|field| {
            let field = parse_field(attrs, index, field, &mut indices)?;
            if !field.no_increment {
                index += 1;
            }
            Ok(field)
        })
        .collect()
}
