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
        let name = &meta.name;
        let (_, child_name) = self.child_ty();
        let schema = quote! {
            const SCHEMA: &'static str = concat!(
                "type ", stringify!(#name), " [", #child_name, "]",
            );
        };

        let inner_ty = self.inner_ty();
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        // let dm_ty = SchemaKind::List.data_model_kind();
        // derive_newtype!(@conv self, meta => dm_ty dm_ty)
        quote!()
    }
}

impl ListReprDefinition {
    fn child_ty(&self) -> (Type, TokenStream) {
        match self {
            Self::Basic { elem, nullable } if *nullable => (
                Type::Verbatim(quote!(Option<#elem>)),
                quote!("nullable ", stringify!(#elem)),
            ),
            Self::Basic { elem, .. } => (elem.clone(), quote!(stringify!(#elem))),
            Self::Advanced(..) => unimplemented!(),
        }
    }
    fn inner_ty(&self) -> Type {
        let (child_ty, _) = self.child_ty();
        Type::Verbatim(quote!(List<#child_ty>))
    }
}
