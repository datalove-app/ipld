use super::*;
use crate::dev::{
    schema::expand::{self, ExpandAdvancedRepresentation, ExpandBasicRepresentation},
    SchemaMeta,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Type};

impl ExpandBasicRepresentation for BasicStructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = meta.lib();
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let generics = &meta
            .generics
            .as_ref()
            .map(|g| quote!(#g))
            .unwrap_or(TokenStream::default());
        let fields: Vec<TokenStream> = self.iter().map(field_typedef).collect();

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::Deserialize, #lib::dev::Serialize)]
            #vis struct #ident #generics {
                #(#fields,)*
            }
        }
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        impl_repr(self.iter(), meta)
    }
    fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

fn field_typedef(field: &StructField) -> TokenStream {
    let attrs = &field.attrs;
    let vis = &field.vis;
    let key = &field.key;
    let value = field_value(field);
    let generics = field.generics.as_ref().map(|g| quote!(#g));

    let implicit_attr = field.implicit.as_ref().map(|_| quote!(#[serde(default)]));
    let rename_attr = field
        .rename
        .as_ref()
        .map(|name| quote!(#[serde(rename = #name)]));

    quote! {
        #(#attrs)*
        #implicit_attr
        #rename_attr
        #vis #key: #value #generics
    }
}

pub(super) fn impl_repr<'a>(
    iter: impl Iterator<Item = &'a StructField>,
    meta: &SchemaMeta,
) -> TokenStream {
    // let lib = &meta.ipld_schema_lib;
    let name = &meta.name;
    let fields: Vec<TokenStream> = iter
        .map(
            |StructField {
                 key, value, rename, ..
             }| {
                let key = key.to_string();
                let rename = rename
                    .as_ref()
                    .map(|s| s.value())
                    .unwrap_or_else(|| key.clone());
                quote! {
                    (#key, _ipld::dev::Field::new::<#value>(#rename))
                }
            },
        )
        .collect();

    let repr_body = expand::impl_repr(
        meta,
        quote! {
            // const KIND: #lib::dev::Kind = #lib::dev::Kind::Struct;
            // const FIELDS: #lib::dev::Fields = #lib::dev::Fields::Struct(&[#(#fields,)*]);
        },
    );
    let selector_bodies = quote! {};
    repr_body
}

pub(super) fn field_value(field: &StructField) -> TokenStream {
    let value = &field.value;
    if field.optional || field.nullable {
        quote!(Option<#value>)
    } else {
        quote!(#value)
    }
}

pub(crate) fn default_field_typdef(field: &StructField) -> TokenStream {
    let attrs = &field.attrs;
    let vis = &field.vis;
    let key = &field.key;
    let value = super::expand::field_value(field);

    quote! {
        #(#attrs)*
        #vis #key: #value
    }
}

impl ExpandBasicRepresentation for StructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.define_type(meta),
            Self::Listpairs(repr) => repr.define_type(meta),
            Self::Tuple(repr) => repr.define_type(meta),
            Self::Stringpairs(repr) => repr.define_type(meta),
            Self::Stringjoin(repr) => repr.define_type(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }

    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.derive_serde(meta),
            Self::Listpairs(repr) => repr.derive_serde(meta),
            Self::Tuple(repr) => repr.derive_serde(meta),
            Self::Stringpairs(repr) => repr.derive_serde(meta),
            Self::Stringjoin(repr) => repr.derive_serde(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        // let lib = &meta.ipld_schema_lib;
        // let repr_body = quote! {
        //     const KIND: #lib::dev::Kind = #lib::dev::Kind::Struct;
        // };
        // expand::impl_repr(meta, repr_body)
        let repr = match self {
            Self::Map(repr) => repr.derive_repr(meta),
            Self::Listpairs(repr) => repr.derive_repr(meta),
            Self::Tuple(repr) => repr.derive_repr(meta),
            Self::Stringpairs(repr) => repr.derive_repr(meta),
            Self::Stringjoin(repr) => repr.derive_repr(meta),
            Self::Advanced(_) => unreachable!(),
        };

        // TODO:
        quote! {
            #repr
            // impl_root_select!(#name => Matcher);
        }
    }

    fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.derive_selects(meta),
            Self::Listpairs(repr) => repr.derive_selects(meta),
            Self::Tuple(repr) => repr.derive_selects(meta),
            Self::Stringpairs(repr) => repr.derive_selects(meta),
            Self::Stringjoin(repr) => repr.derive_selects(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }
}

impl ExpandAdvancedRepresentation for StructReprDefinition {
    fn expand_struct(repr: AdvancedStructReprDefinition) -> TokenStream {
        unimplemented!()
    }
}
