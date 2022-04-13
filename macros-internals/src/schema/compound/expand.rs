use super::*;
use crate::dev::*;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result as ParseResult},
    LitStr, Path, Token, Type,
};

impl ListReprDefinition {
    fn inner_ty(&self) -> TokenStream {
        match self {
            Self::Advanced(..) => unreachable!(),
            Self::Basic { elem, nullable } => {
                if *nullable {
                    quote!(Option<#elem>)
                } else {
                    quote!(#elem)
                }
            }
        }
    }
}

impl ExpandBasicRepresentation for ListReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let name = &meta.name;
        let generics = &meta
            .generics
            .as_ref()
            .map(|g| quote!(#g))
            .unwrap_or(TokenStream::default());
        let inner_ty = self.inner_ty();

        quote! {
            #(#attrs)*
            #[repr(transparent)]
            #[derive(Deserialize, Serialize)]
            #[serde(transparent)]
            #vis struct #name #generics (List<#inner_ty>);
        }
    }

    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let inner_ty = self.inner_ty();
        let visitor = impl_context_seed_visitor(meta, "", quote! {});
        let deseed = impl_context_seed_deseed(meta, quote! {});

        quote! {
            if !#lib::dev::impls::impls!(
                ContextSeed<'_, Ctx, List<#inner_ty>>
            ) {
                #visitor
            }

            if !#lib::dev::impls::impls!(
                ContextSeed<'_, Ctx, List<#inner_ty>>
            ) {
                #deseed
            }
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        // let lib = &meta.ipld_schema_lib;
        // let repr_body = quote! {
        //     const KIND: Kind = Kind::Struct;
        // };

        impl_repr(meta, quote! {})
    }

    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        Default::default()
    }
}
