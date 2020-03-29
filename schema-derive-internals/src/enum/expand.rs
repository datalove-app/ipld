use super::*;
use crate::{ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for EnumReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let lib = &meta.ipld_schema;

        match self {
            Self::String { fields } => {
                println!("outputting fields: {:?}", quote!(#fields));
                quote! {
                    #attrs
                    #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
                    #vis enum #ident {
                        #fields
                    }
                }
            }
            Self::Int { fields } => {
                let repr_type = &fields.repr_type;
                quote! {
                    #attrs
                    #[derive(#lib::dev::serde_repr::Deserialize_repr, #lib::dev::serde_repr::Serialize_repr)]
                    #[repr(#repr_type)]
                    #vis enum #ident {
                        #fields
                    }
                }
            }
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

impl ToTokens for EnumStrFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.iter().map(|field| {
            let EnumStrField { attrs, name, alias } = field;
            match alias {
                Some(rename) => quote! {
                    #attrs
                    #[serde(rename = #rename)]
                    #name,
                },
                None => quote! {
                    #attrs
                    #name,
                },
            }
        }))
    }
}

impl ToTokens for EnumIntFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.fields.iter().map(|field| {
            let EnumIntField { attrs, name, alias } = field;
            quote! {
                #attrs
                #name = #alias,
            }
        }))
    }
}
