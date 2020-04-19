use super::*;
use crate::dev::{
    schema::expand::{self, ExpandAdvancedRepresentation, ExpandBasicRepresentation},
    SchemaMeta,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Type};

impl ExpandBasicRepresentation for StringpairsStructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = meta.lib();
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields: Vec<TokenStream> = self
            .iter()
            .map(super::expand::default_field_typdef)
            .collect();

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::Deserialize, #lib::dev::Serialize)]
            #vis struct #ident {
                #(#fields)*
            }
        }
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        super::expand::impl_repr(self.iter(), meta)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

fn field_to_tokens(field: &StructField) -> TokenStream {
    let attrs = &field.attrs;
    let key = &field.key;
    let value = &field.value;

    let value = if field.optional {
        quote!(Option<#value>)
    } else {
        quote!(#value)
    };
    let implicit_attr = if let Some(implicit) = &field.implicit {
        quote!(#[serde(default)])
    } else {
        TokenStream::default()
    };
    let rename_attr = if let Some(rename) = &field.rename {
        quote!(#[serde(rename = #rename)])
    } else {
        TokenStream::default()
    };

    quote! {
        #(#attrs)*
        #implicit_attr
        #rename_attr
        #key: #value
    }
}
