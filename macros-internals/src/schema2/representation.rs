use std::fmt;

use super::Ipld;
use crate::dev::*;
use darling::{
    ast::{self, Data as Body, GenericParam},
    util, Error, FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_quote,
    punctuated::Punctuated,
    token::Add,
    Attribute, Expr, Field, Generics, Ident, LitStr, Type, TypeParamBound, Variant, Visibility,
};

pub const CODE: u64 = 'R' as u64;

pub type DeriveRepresentation = Ipld<CODE>;

/// Representation kinds for schema types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum ReprKind {
    // defaults for structs (maps), enums (int/str), and unions (??)
    Default,
    // copy (newtype)
    Transparent,
    Tuple,
    Stringpairs,
    Stringjoin,
    Listpairs,
    Keyed,
    Kinded,
    Envelope,
    Inline,
    Bytesprefix,
    Stringprefix,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SchemaKind {
    Null,
    Struct(ReprKind),
    Enum,
    Union(ReprKind),
    Copy,
}

// impl Kind {
//     fn validate(&self, implied: Self) -> Option<Self> {
//         use Kind::*;
//         match (input.unwrap_or_default(), implied) {
//             (r#in, im) if r#in == im => Some(im),
//             (Default, _) => Some(implied),
//         }
//     }
// }

impl Default for ReprKind {
    fn default() -> Self {
        Self::Default
    }
}

impl DeriveRepresentation {
    fn validate(&self) -> ParseResult<()> {
        Ok(())
    }

    pub(crate) fn derive(&self) -> TokenStream {
        let name = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let body = match self.schema_kind().unwrap() {
            SchemaKind::Null => self.expand_null(),
            SchemaKind::Struct(_) => Default::default(),
            SchemaKind::Enum => Default::default(),
            SchemaKind::Union(_) => Default::default(),
            SchemaKind::Copy => self.expand_newtype(),
        };

        quote! {
            impl #impl_generics Representation for #name #ty_generics #where_clause {
                #body
            }
        }
    }

    pub(crate) fn expand_null(&self) -> TokenStream {
        let name = &self.ident;
        let schema = self.to_string();

        quote! {
            const NAME: &'static str = stringify!(#name);
            const SCHEMA: &'static str = #schema;
            const DATA_MODEL_KIND: Kind = Kind::Null;

            fn to_selected_node(&self) -> SelectedNode {
                SelectedNode::Null
            }

            #[doc(hidden)]
            #[inline]
            fn serialize<const __C: u64, __S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: Serializer,
            {
                serializer.serialize_none()
            }

            #[doc(hidden)]
            #[inline]
            fn deserialize<'de, const __C: u64, __D>(deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: Deserializer<'de>,
            {
                <()>::deserialize(deserializer)?;
                Ok(Self)
            }
        }
    }

    pub(crate) fn expand_newtype(&self) -> TokenStream {
        let name = &self.ident;
        let inner = &self.fields().next().unwrap().ty;
        let schema = self.to_string();

        quote! {
            const NAME: &'static str = stringify!(#name);
            const SCHEMA: &'static str = #schema;
            const DATA_MODEL_KIND: Kind =
                <#inner as Representation>::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind =
                <#inner as Representation>::SCHEMA_KIND.union(Kind::Copy);
            // TODO:
            const REPR_KIND: Kind =
                <#inner as Representation>::REPR_KIND;
            const REPR_STRATEGY: Strategy =
                <#inner as Representation>::REPR_STRATEGY;
            const HAS_LINKS: bool =
                <#inner as Representation>::HAS_LINKS;

            #[inline]
            fn name(&self) -> &'static str {
                Representation::name(&self.0)
            }
            #[inline]
            fn has_links(&self) -> bool {
                Representation::has_links(&self.0)
            }
            #[inline]
            fn as_field(&self) -> Option<Field<'_>> {
                Representation::as_field(&self.0)
            }
            #[inline]
            fn to_selected_node(&self) -> SelectedNode {
                Representation::to_selected_node(&self.0)
            }
            #[inline]
            fn serialize<const __C: u64, __S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: Serializer,
            {
                Representation::serialize::<__C, __S>(&self.0, serializer)
            }
            #[inline]
            fn deserialize<'de, const __C: u64, __D>(deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: Deserializer<'de>,
            {
                Ok(Self(Representation::deserialize::<__C, __D>(deserializer)?))
            }
        }
    }

    fn expand_map(&self) -> TokenStream {
        let name = &self.ident;
        let inner = &self.fields().next().unwrap().ty;
        let schema = {
            let inner: Ident = parse_quote!(#inner);
            format!("type {} = {}", name, &inner)
        };

        quote! {
            const NAME: &'static str = stringify!(#name);
            const SCHEMA: &'static str = #schema;
            const DATA_MODEL_KIND: Kind =
                <#inner as Representation>::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind =
                <#inner as Representation>::SCHEMA_KIND.union(Kind::Copy);
            // TODO:
            const REPR_KIND: Kind =
                <#inner as Representation>::REPR_KIND;
            const REPR_STRATEGY: Strategy =
                <#inner as Representation>::REPR_STRATEGY;
            const HAS_LINKS: bool =
                <#inner as Representation>::HAS_LINKS;

            #[inline]
            fn name(&self) -> &'static str {
                Representation::name(&self.0)
            }
            #[inline]
            fn has_links(&self) -> bool {
                Representation::has_links(&self.0)
            }
            #[inline]
            fn as_field(&self) -> Option<Field<'_>> {
                Representation::as_field(&self.0)
            }
            #[inline]
            fn to_selected_node(&self) -> SelectedNode {
                Representation::to_selected_node(&self.0)
            }
            #[inline]
            fn serialize<const __C: u64, __S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: Serializer,
            {
                Representation::serialize::<__C, __S>(&self.0, serializer)
            }
            #[inline]
            fn deserialize<'de, const __C: u64, __D>(deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: Deserializer<'de>,
            {
                Ok(Self(Representation::deserialize::<__C, __D>(deserializer)?))
            }
        }
    }
}

impl fmt::Display for DeriveRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.schema_kind().or(Err(fmt::Error))? {
            SchemaKind::Null => write!(f, "type {} null", &self.ident),
            SchemaKind::Struct(repr) => unimplemented!(),
            SchemaKind::Enum => unimplemented!(),
            SchemaKind::Union(repr) => unimplemented!(),
            SchemaKind::Copy => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
