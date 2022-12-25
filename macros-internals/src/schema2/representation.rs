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
        let body = self.body();

        quote! {
            impl #impl_generics Representation for #name #ty_generics
            #where_clause {
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

    pub(crate) fn expand_struct(&self, repr: &ReprKind) -> TokenStream {
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

    pub(crate) fn expand_enum(&self) -> TokenStream {
        Default::default()
    }

    pub(crate) fn expand_union(&self, repr: &ReprKind) -> TokenStream {
        let name = &self.ident;
        let schema = self.to_string();

        let expecting = self.expecting();

        quote! {
            const NAME: &'static str = stringify!(#name);
            const SCHEMA: &'static str = #schema;
            // todo:
            const DATA_MODEL_KIND: Kind = Kind::Null;
            const SCHEMA_KIND: Kind = Kind::Union;
            const REPR_KIND: Kind = Kind::Any;
            const REPR_STRATEGY: Strategy = Strategy::Kinded;
            // todo:
            const HAS_LINKS: bool = false;

            #[inline]
            fn name(&self) -> &'static str {
                unimplemented!()
            }
            #[inline]
            fn has_links(&self) -> bool {
                unimplemented!()
            }
            #[inline]
            fn as_field(&self) -> Option<Field<'_>> {
                unimplemented!()
            }
            #[inline]
            fn to_selected_node(&self) -> SelectedNode {
                unimplemented!()
            }
            #[inline]
            fn serialize<const __C: u64, __S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: Serializer,
            {
                match self {
                    _ => unimplemented!(),
                }
            }
            #[inline]
            fn deserialize<'de, const __C: u64, __D>(de: __D) -> Result<Self, __D::Error>
            where
                __D: Deserializer<'de>,
            {
                struct V<const MC: u64>;
                impl<'de, const MC: u64> Visitor<'de> for V<MC> {
                    type Value = #name;
                    #expecting
                    // #(#non_link_visitors)*
                }
                impl<'de, const MC: u64> LinkVisitor<'de, MC> for V<MC> {
                    // #(#link_visitors)*
                }

                Multicodec::deserialize_any::<__C, __D, _>(de, V::<__C>)
            }
        }
    }
}

impl fmt::Display for DeriveRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.schema_kind().or(Err(fmt::Error))? {
            SchemaKind::Null => write!(f, "type {} null", &self.ident),
            SchemaKind::Copy => write!(
                f,
                "type {} = {:?}",
                &self.ident,
                &self.fields().next().unwrap().ty
            ),
            SchemaKind::Enum => write!(f, "unimplemented"),
            SchemaKind::Struct(repr) => match repr {
                _ => write!(f, "unimplemented"),
            },
            SchemaKind::Union(repr) => match repr {
                ReprKind::Kinded => {
                    write!(f, "type {} union {{\n", &self.ident)?;
                    for variant in self.variants() {
                        // todo: add dm kind
                        write!(f, "| {}", &variant.ident,)?;
                    }
                    write!(f, "}}")
                }
                ReprKind::Keyed => {
                    write!(f, "type {} union {{\n", &self.ident)?;
                    for variant in self.variants() {
                        write!(f, "| {} \"{:?}\"", &variant.ident, &variant.rename)?;
                    }
                    write!(f, "}}")
                }
                _ => write!(f, "unimplemented"),
            },
            _ => write!(f, "unimplemented"),
        }
    }
}
