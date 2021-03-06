use super::*;
use crate::dev::{schema::expand::ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for EnumReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = meta.lib();
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;

        match self {
            Self::String { fields } => quote! {
                #(#attrs)*
                #[derive(#lib::dev::Deserialize, #lib::dev::Serialize)]
                #vis enum #ident {
                    #(#fields,)*
                }
            },
            Self::Int {
                fields: EnumIntFields { repr_type, fields },
            } => quote! {
                #(#attrs)*
                #[derive(#lib::dev::serde_repr::Deserialize_repr, #lib::dev::serde_repr::Serialize_repr)]
                #[repr(#repr_type)]
                #vis enum #ident {
                    #(#fields,)*
                }
            },
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

impl ToTokens for EnumStrField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { attrs, name, alias } = &self;
        tokens.append_all(match alias {
            Some(rename) => quote! {
                #(#attrs)*
                #[serde(rename = #rename)]
                #name
            },
            None => quote! {
                #(#attrs)*
                #name
            },
        });
    }
}

impl ToTokens for EnumIntField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { attrs, name, alias } = &self;
        tokens.append_all(quote! {
            #(#attrs)*
            #name = #alias
        })
    }
}
