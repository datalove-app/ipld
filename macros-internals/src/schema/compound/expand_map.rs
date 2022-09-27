use super::*;
use crate::dev::*;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result as ParseResult},
    LitStr, Path, Token, Type,
};

impl ExpandBasicRepresentation for MapReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();

        match self {
            Self::Basic { .. } => derive_newtype!(@typedef self, meta => inner_ty),
            Self::Stringpairs { .. } => unimplemented!(),
            Self::Listpairs { .. } => unimplemented!(),
            Self::Advanced(..) => unimplemented!(),
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let repr_kind = self.repr_kind();
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Map;
                const REPR_KIND: Kind = #repr_kind;
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@select meta => inner_ty)
        // quote!()
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        // let val_ty = self.val_ty();
        // derive_newtype!(@select meta => val_ty)
        quote!()
    }
}

impl MapReprDefinition {
    fn key_ty(&self) -> Type {
        match self {
            Self::Basic { key, .. }
            | Self::Listpairs { key, .. }
            | Self::Stringpairs { key, .. } => key.clone(),
            _ => unimplemented!(),
        }
    }

    fn val_ty(&self) -> Type {
        match self {
            Self::Basic {
                value, nullable, ..
            }
            | Self::Stringpairs {
                value, nullable, ..
            }
            | Self::Listpairs {
                value, nullable, ..
            } if *nullable => Type::Verbatim(quote!(Option<#value>)),
            Self::Basic { value, .. }
            | Self::Stringpairs { value, .. }
            | Self::Listpairs { value, .. } => value.clone(),
            Self::Advanced(..) => unimplemented!(),
        }
    }

    fn inner_ty(&self) -> Type {
        let key_ty = self.key_ty();
        let val_ty = self.val_ty();
        Type::Verbatim(quote!(Map<#key_ty, #val_ty>))
    }

    fn repr_kind(&self) -> TokenStream {
        match self {
            Self::Basic { .. } => quote!(Kind::Map),
            Self::Stringpairs { .. } => quote!(Kind::String),
            Self::Listpairs { .. } => quote!(Kind:::List),
            Self::Advanced(..) => unimplemented!(),
        }
    }
}
