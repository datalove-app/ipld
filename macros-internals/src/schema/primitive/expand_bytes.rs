use super::BytesReprDefinition;
use crate::{
    derive_newtype,
    dev::{schema::expand, SchemaMeta},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

impl BytesReprDefinition {
    fn inner_ty(&self) -> Type {
        match self {
            Self::Basic => Type::Verbatim(quote!(Bytes)),
            Self::Advanced(_) => unimplemented!(),
        }
    }
}

impl expand::ExpandBasicRepresentation for BytesReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        // TODO
        let consts = quote! {
            const DATA_MODEL_KIND: Kind = Kind::Bytes;
            const SCHEMA_KIND: Kind = Kind::Bytes;
            const REPR_KIND: Kind = Kind::Bytes;
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
