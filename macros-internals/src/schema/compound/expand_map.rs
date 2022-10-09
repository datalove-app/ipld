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
            Self::Stringpairs { .. } => derive_newtype!(@typedef self, meta => inner_ty),
            Self::Listpairs { .. } => derive_newtype!(@typedef self, meta => inner_ty),
            Self::Advanced(..) => unimplemented!(),
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let key_ty = self.key_ty();
        let (val_ty, val_name) = self.val_ty();
        let inner_ty = self.inner_ty();
        let repr_kind = self.repr_kind();

        let base_schema = quote! {
            "type ", stringify!(#name), " ",
            "{", stringify!(#key_ty), ":", stringify!(#val_name), "}",
        };
        match self {
            Self::Basic { .. } => {
                let schema = quote! {
                    const SCHEMA: &'static str = concat!(#base_schema);
                };
                derive_newtype!(@repr { schema } meta => inner_ty)
            }
            Self::Listpairs { .. } => {
                let schema = quote! {
                    const SCHEMA: &'static str =
                        concat!(#base_schema, "representation listpairs");
                };
                derive_newtype!(@repr { schema } meta => inner_ty)
            }
            Self::Stringpairs {
                inner_delim,
                entry_delim,
                ..
            } => {
                let schema = quote! {
                    const SCHEMA: &'static str = concat!(
                        #base_schema,
                        "representation stringpairs { ",
                            "innerDelim ", #inner_delim, " ",
                            "entryDelim ", #entry_delim, " ",
                        "}",
                    );
                };
                derive_newtype!(@repr { schema } meta => inner_ty)
            }
            _ => unimplemented!(),
        }
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

    fn val_ty(&self) -> (Type, TokenStream) {
        match self {
            Self::Basic {
                value, nullable, ..
            }
            | Self::Listpairs {
                value, nullable, ..
            }
            | Self::Stringpairs {
                value, nullable, ..
            } if *nullable => (
                Type::Verbatim(quote!(Option<#value>)),
                quote!("nullable ", stringify!(#value)),
            ),
            Self::Basic { value, .. }
            | Self::Listpairs { value, .. }
            | Self::Stringpairs { value, .. } => (value.clone(), quote!(stringify!(#value))),
            Self::Advanced(..) => unimplemented!(),
        }
    }

    fn inner_ty(&self) -> Type {
        let key_ty = self.key_ty();
        let (val_ty, _) = self.val_ty();
        match self {
            Self::Basic { .. } => Type::Verbatim(quote!(Map<#key_ty, #val_ty>)),
            Self::Listpairs { .. } => Type::Verbatim(quote!(ListPairsMap<#key_ty, #val_ty>)),
            Self::Stringpairs { .. } => Type::Verbatim(quote!(StringPairsMap<#key_ty, #val_ty>)),
            _ => unimplemented!(),
        }
    }

    fn repr_kind(&self) -> TokenStream {
        match self {
            Self::Basic { .. } => quote!(Kind::Map),
            Self::Listpairs { .. } => quote!(Kind:::List),
            Self::Stringpairs { .. } => quote!(Kind::String),
            Self::Advanced(..) => unimplemented!(),
        }
    }
}
