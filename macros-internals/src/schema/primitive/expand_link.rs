use super::LinkReprDefinition;
use crate::{
    derive_newtype,
    dev::{
        schema::{expand, SchemaKind},
        SchemaMeta,
    },
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
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = self.inner_ty();
        derive_newtype!(@typedef_transparent self, meta => inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME), " link");
                const DATA_MODEL_KIND: Kind = Kind::Link;
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = SchemaKind::Link.data_model_kind();
        let sn_ty = SchemaKind::Link.selected_node_ident();
        derive_newtype!(@conv @has_constructor self, meta => dm_ty sn_ty)
    }
}
