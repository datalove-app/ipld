use super::LinkReprDefinition;
use crate::{
    derive_newtype,
    dev::{schema::expand, SchemaMeta},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

impl LinkReprDefinition {
    fn inner_ty(&self) -> Type {
        let child_ty = &self.0;
        Type::Verbatim(quote!(Link<#child_ty>))
    }
}

impl expand::ExpandBasicRepresentation for LinkReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let schema = format!("type {} link", meta.name_str());
        quote!(#schema)
    }
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        let consts = quote! {
            const DATA_MODEL_KIND: Kind = Kind::Link;
            const SCHEMA_KIND: Kind = Kind::Link;
            const REPR_KIND: Kind = Kind::Link;
        };
        derive_newtype!(@repr self, meta => inner_ty { consts })
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        derive_newtype!(@conv @has_constructor self, meta)
    }
}
