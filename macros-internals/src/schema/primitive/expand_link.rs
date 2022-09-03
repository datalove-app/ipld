use super::LinkReprDefinition;
use crate::{
    define_newtype,
    dev::{
        schema::{expand, SchemaKind},
        SchemaMeta,
    },
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

impl expand::ExpandBasicRepresentation for LinkReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner = &self.0;
        let inner_type = Type::Verbatim(quote!(Link<#inner>));
        define_newtype!(self, meta => inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner = &self.0;
        let inner_type = Type::Verbatim(quote!(Link<#inner>));

        // TODO:
        let repr_def = quote! {};
        let wrapper_repr_def = quote! {};

        quote! {
            #repr_def
            #wrapper_repr_def
        }
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}
