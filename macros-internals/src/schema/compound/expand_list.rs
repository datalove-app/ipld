use super::*;
use crate::dev::*;
use quote::quote;
use syn::Type;

impl ExpandBasicRepresentation for ListReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let name_str = meta.name_str();
        let (_, child_name) = self.child();
        quote!(concat!("type ", #name_str, " [", #child_name, "]"))
    }

    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = self.inner_ty();
        let consts = quote! {
            const DATA_MODEL_KIND: Kind = Kind::List;
            const SCHEMA_KIND: Kind = Kind::List;
            const REPR_KIND: Kind = Kind::List;
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

impl ListReprDefinition {
    fn child(&self) -> (Type, TokenStream) {
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
        let (child_ty, _) = self.child();
        Type::Verbatim(quote!(List<#child_ty>))
    }
}
