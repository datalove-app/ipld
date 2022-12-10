use super::*;
use crate::dev::{schema::expand::ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for InlineUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields: Vec<TokenStream> = self.iter().map(field_typedef).collect();

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
            #vis enum #ident {
                #(#fields)*
            }
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        // impl_repr(self.iter(), meta)
        TokenStream::default()
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}

fn field_typedef(field: &UnionField<LitStr>) -> TokenStream {
    let attrs = &field.attrs;
    let key = &field.key;

    // let implicit_attr = if let Some(implicit) = &field.implicit {
    //     quote!(#[serde(default)])
    // } else {
    //     TokenStream::default()
    // };

    // let rename_attr = if let Some(rename) = &field.rename {
    //     quote!(#[serde(rename = #rename)])
    // } else {
    //     TokenStream::default()
    // };

    quote! {
        #(#attrs)*
        #key
    }
}
