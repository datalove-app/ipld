use super::{
    BoolReprDefinition, CopyReprDefinition, FloatReprDefinition, IntReprDefinition,
    NullReprDefinition, StringReprDefinition,
};
use crate::{
    derive_newtype,
    dev::{schema::expand, SchemaMeta},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/*
///
#[macro_export(local_inner_macros)]
macro_rules! derive_newtype_select {
    / (@visitor_null $def:ident, $meta:ident => fn $visit_fn:ident) => {{
        //     expand::impl_context_seed_visitor($meta, quote! {
        //         #[inline]
        //         fn visit_unit<E>(self) -> Result<Self::Any, E>
        //         where
        //             E: serde::de::Error,
        //         {
        //             self.visit_primitive(())
        //         }
        //     })
        // }};
        // (@visitor_primitive $meta:ident => $visit_fn:ident ($ty:ty) $($expecting:tt)*) => {{
        //     let name = &$meta.name;
        //     expand::impl_context_seed_visitor(
        //         $meta,
        //         $($expecting)*,
        //         quote! {
        //             #[inline]
        //             fn $visit_fn<E>(self, v: $ty) -> Result<Self::Any, E>
        //             where
        //                 E: serde::de::Error,
        //             {
        //                 self.visit_primitive(v)
        //             }
        //         },
        //     )
        // }};
        // (@deseed $meta:ident => $deserialize_fn:ident) => {{
        //     expand::impl_context_seed_deseed(
        //         $meta,
        //         quote! {
        //             deserializer.$deserialize_fn(self)
        //         },
        //     )
        // }};
        // (@select $def:ident, $meta:ident => $inner_ty:ident) => {{
        //     expand::impl_select(
        //         $meta,
        //         // quote! {
        //         //     unimplemented!()
        //         //     // Ok(#$inner_ty::r#match(seed)?.map(Self))
        //         // },
        //         quote! {
        //             unimplemented!()
        //             // if type_eq::<Self, S>() {
        //             //     type_cast_selection::<Self, S, _, _>(|| {
        //             //         let seed = seed.into::<#$inner_ty, #$inner_ty>();
        //             //         Ok(#$inner_ty::select::<#$inner_ty>(seed)?.map(Self))
        //             //     })
        //             // } else {
        //             //     #$inner_ty::select::<S>(seed)
        //             // }
        //         },
        //     )
        // }};
}
 */

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for NullReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let schema = format!("type {} null", meta.name);
        quote!(#schema)
    }

    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let name = &meta.name;
        let generics = &meta
            .generics
            .as_ref()
            .map(|g| quote::quote!(#g))
            .unwrap_or_default();

        quote::quote! {
            #(#attrs)*
            #vis struct #name #generics;
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        self.impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Null;
                const SCHEMA_KIND: Kind = Kind::Null;
                const REPR_KIND: Kind = Kind::Null;
            },
            quote! {
                fn to_selected_node(&self) -> SelectedNode {
                    SelectedNode::Null
                }

                #[inline]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_none()
                }
                #[inline]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    <()>::deserialize(deserializer)?;
                    Ok(Self)
                }
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        quote::quote! {
            repr_serde! { @select for #name }
            repr_serde! { @visitors for #name {
                #[inline]
                fn expecting(&self, f: &mut maybestd::fmt::Formatter<'_>) -> maybestd::fmt::Result {
                    write!(f, "A nothing type {}", <#name>::NAME)
                }
                #[inline]
                fn visit_none<Er>(self) -> Result<Self::Value, Er>
                where
                    Er: de::Error,
                {
                    self.into_inner()
                        .select_scalar::<MC>(#name)
                        .map_err(Er::custom)
                }
                #[inline]
                fn visit_unit<Er>(self) -> Result<Self::Value, Er>
                where
                    Er: de::Error,
                {
                    self.visit_none()
                }
            }}
        }
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        quote! {
            #[automatically_derived]
            impl From<#name> for SelectedNode {
                fn from(t: #name) -> Self {
                    Self::Null
                }
            }
            #[automatically_derived]
            impl Into<Any> for #name {
                fn into(self) -> Any {
                    Any::Null(Default::default())
                }
            }
            #[automatically_derived]
            impl TryFrom<Any> for #name {
                type Error = Error;
                fn try_from(any: Any) -> Result<Self, Self::Error> {
                    match any {
                        Any::Null(_) => Ok(Self),
                        _ => Err(Error::MismatchedAny)
                    }
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl BoolReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(bool))
    }
}

impl expand::ExpandBasicRepresentation for BoolReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let schema = format!("type {} bool", meta.name);
        quote!(#schema)
    }
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        let schema = quote! {
            const DATA_MODEL_KIND: Kind = Kind::Bool;
            const SCHEMA_KIND: Kind = Kind::Bool;
            const REPR_KIND: Kind = Kind::Bool;
        };
        derive_newtype!(@repr self, meta => inner_ty { schema })
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        derive_newtype!(@conv @has_constructor self, meta)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for IntReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let schema = format!("type {} int", meta.name);
        quote!(#schema)
    }
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        let schema = quote! {
            const DATA_MODEL_KIND: Kind = <#inner_ty>::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind = <#inner_ty>::SCHEMA_KIND;
            const REPR_KIND: Kind = <#inner_ty>::REPR_KIND;
        };
        derive_newtype!(@repr self, meta => inner_ty { schema })
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        derive_newtype!(@conv @has_constructor self, meta)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for FloatReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let schema = format!("type {} float", meta.name);
        quote!(#schema)
    }
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        let schema = quote! {
            const DATA_MODEL_KIND: Kind = <#inner_ty>::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind = <#inner_ty>::SCHEMA_KIND;
            const REPR_KIND: Kind = <#inner_ty>::REPR_KIND;
        };
        derive_newtype!(@repr self, meta => inner_ty { schema })
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        derive_newtype!(@conv @has_constructor self, meta)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl StringReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(String))
    }
}

impl expand::ExpandBasicRepresentation for StringReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let schema = format!("type {} string", meta.name);
        quote!(#schema)
    }
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        let schema = quote! {
            const DATA_MODEL_KIND: Kind = Kind::String;
            const SCHEMA_KIND: Kind = Kind::String;
            const REPR_KIND: Kind = Kind::String;
        };
        derive_newtype!(@repr self, meta => inner_ty { schema })
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        derive_newtype!(@conv @has_constructor self, meta)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for CopyReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        let start = format!("type {} = ", meta.name);
        let inner_ty = &self.0;
        quote!(concat!(#start, stringify!(#inner_ty)))
    }
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        let schema = quote! {
            const DATA_MODEL_KIND: Kind = <#inner_ty>::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind = <#inner_ty>::SCHEMA_KIND.union(Kind::Copy);
            const REPR_KIND: Kind = <#inner_ty>::REPR_KIND;
            const REPR_STRATEGY: Strategy = <#inner_ty>::REPR_STRATEGY;
        };
        derive_newtype!(@repr self, meta => inner_ty { schema })
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        derive_newtype!(@conv @has_constructor self, meta)
    }
}
