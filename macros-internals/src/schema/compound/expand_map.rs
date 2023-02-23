use super::*;
use crate::dev::*;
use quote::quote;
use syn::Type;

impl ExpandBasicRepresentation for MapReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let name_str = meta.name_str();
        let (_, key_name) = self.key();
        let (_, val_name) = self.val();
        let base_schema = quote! {
            concat!("type ", #name_str, " {", #key_name, ":", #val_name, "}")
        };

        match self {
            Self::Basic { .. } => quote! {
                concat!(#base_schema, " representation map")
            },
            Self::Listpairs { .. } => quote! {
                concat!(#base_schema, " representation listpairs")
            },
            Self::Stringpairs {
                inner_delim,
                entry_delim,
                ..
            } => quote! {
                concat!(#base_schema, " representation stringpairs { ",
                    "innerDelim ", #inner_delim, " ",
                    "entryDelim ", #entry_delim, " ",
                "}")
            },
            _ => unimplemented!(),
        }
    }

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
        let inner_ty = self.inner_ty();

        let (repr_kind, repr_strategy) = self.repr();
        let consts = quote! {
            const DATA_MODEL_KIND: Kind = Kind::Map;
            const SCHEMA_KIND: Kind = Kind::Map;
            const REPR_KIND: Kind = #repr_kind;
            const REPR_STRATEGY: Strategy = #repr_strategy;
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

impl MapReprDefinition {
    fn repr(&self) -> (TokenStream, TokenStream) {
        match self {
            Self::Basic { .. } => (quote!(Kind::Map), quote!(Strategy::Basic)),
            Self::Listpairs { .. } => (quote!(Kind::List), quote!(Strategy::Listpairs)),
            Self::Stringpairs { .. } => (quote!(Kind::String), quote!(Strategy::Stringpairs)),
            _ => unimplemented!(),
        }
    }

    fn inner_ty(&self) -> Type {
        let (key, _) = self.key();
        let (val, _) = self.val();
        Type::Verbatim(match self {
            Self::Basic { .. } => quote!(Map<#key, #val>),
            Self::Listpairs { .. } => quote!(ListpairsMap<#key, #val>),
            Self::Stringpairs { .. } => quote!(StringpairsMap<#key, #val>),
            _ => unimplemented!(),
        })
    }

    fn key(&self) -> (Type, TokenStream) {
        match self {
            Self::Basic { key, .. }
            | Self::Listpairs { key, .. }
            | Self::Stringpairs { key, .. } => {
                (key.clone(), quote!(<#key as Representation>::NAME))
            }
            _ => unimplemented!(),
        }
    }

    fn val(&self) -> (Type, TokenStream) {
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
                quote!("nullable ", <#value as Representation>::NAME),
            ),
            Self::Basic { value, .. }
            | Self::Listpairs { value, .. }
            | Self::Stringpairs { value, .. } => {
                (value.clone(), quote!(<#value as Representation>::NAME))
            }
            _ => unimplemented!(),
        }
    }
}
