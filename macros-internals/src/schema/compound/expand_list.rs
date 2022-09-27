use super::*;
use crate::dev::*;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result as ParseResult},
    LitStr, Path, Token, Type,
};

impl ExpandBasicRepresentation for ListReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let child_ty = self.child_ty();
        expand::impl_repr(
            meta,
            quote! {
                // const SCHEMA: &'static str = macros::concatcp!("type ", Self::NAME, " [", <#child_ty>::NAME, "]");
                const DATA_MODEL_KIND: Kind = Kind::List;
                const HAS_LINKS: bool = false;

                // fn has_links(&self) -> bool {
                //     self.0.has_links()
                // }
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@select meta => inner_ty)
        // quote!()
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}

impl ListReprDefinition {
    fn child_ty(&self) -> Type {
        match self {
            Self::Basic { elem, nullable } if *nullable => Type::Verbatim(quote!(Option<#elem>)),
            Self::Basic { elem, .. } => elem.clone(),
            Self::Advanced(..) => unimplemented!(),
        }
    }
    fn inner_ty(&self) -> Type {
        let child_ty = self.child_ty();
        Type::Verbatim(quote!(List<#child_ty>))
    }
}
