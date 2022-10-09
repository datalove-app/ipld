use super::{
    BoolReprDefinition, CopyReprDefinition, FloatReprDefinition, IntReprDefinition,
    NullReprDefinition, StringReprDefinition,
};
use crate::{
    derive_newtype,
    dev::{
        schema::{expand, SchemaKind},
        SchemaMeta,
    },
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

impl NullReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(Null))
    }
}

impl expand::ExpandBasicRepresentation for NullReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let inner_ty = Self::inner_ty();
        let schema = quote! {
            const SCHEMA: &'static str = concat!("type ", stringify!(#name), " null");
        };
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        quote! {
            impl From<#name> for SelectedNode {
                fn from(t: #name) -> Self {
                    Self::Null
                }
            }

            impl Into<Any> for #name {
                fn into(self) -> Any {
                    Any::Null(Null)
                }
            }

            impl TryFrom<Any> for #name {
                type Error = Error;
                fn try_from(any: Any) -> Result<Self, Self::Error> {
                    match any {
                        Any::Null(inner) => Ok(Self(inner)),
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
        Type::Verbatim(quote!(Bool))
    }
}

impl expand::ExpandBasicRepresentation for BoolReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let inner_ty = Self::inner_ty();
        let schema = quote! {
            const SCHEMA: &'static str = concat!("type ", stringify!(#name), " bool");
        };
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = SchemaKind::Bool.data_model_kind();
        derive_newtype!(@conv @has_constructor self, meta => dm_ty dm_ty)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for IntReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let inner_ty = &self.0;
        let schema = quote! {
            const SCHEMA: &'static str = concat!("type ", stringify!(#name), " int");
        };
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = self.1.data_model_kind();
        let sn_ty = self.1.selected_node_ident();
        derive_newtype!(@conv @has_constructor self, meta => dm_ty sn_ty)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for FloatReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let inner_ty = &self.0;
        let schema = quote! {
            const SCHEMA: &'static str = concat!("type ", stringify!(#name), " float");
        };
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = self.1.data_model_kind();
        let sn_ty = self.1.selected_node_ident();
        derive_newtype!(@conv @has_constructor self, meta => dm_ty sn_ty)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl StringReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(IpldString))
    }
}

impl expand::ExpandBasicRepresentation for StringReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let inner_ty = Self::inner_ty();
        let schema = quote! {
            const SCHEMA: &'static str = concat!("type ", stringify!(#name), " string");
        };
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = SchemaKind::String.data_model_kind();
        derive_newtype!(@conv @has_constructor self, meta => dm_ty dm_ty)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl expand::ExpandBasicRepresentation for CopyReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@typedef self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let inner_ty = &self.0;
        let schema = quote! {
            const SCHEMA: &'static str = concat!("type ", stringify!(#name),  " = ", stringify!(#inner_ty));

            // #[inline]
            // fn name(&self) -> &'static str {
            //     Representation::name(&self.0)
            // }
            #[inline]
            fn data_model_kind(&self) -> Kind {
                Representation::data_model_kind(&self.0)
            }
            #[inline]
            fn schema_kind(&self) -> Kind {
                Representation::schema_kind(&self.0)
            }
            #[inline]
            fn repr_kind(&self) -> Kind {
                Representation::repr_kind(&self.0)
            }
            #[inline]
            fn has_links(&self) -> bool {
                Representation::has_links(&self.0)
            }
        };
        derive_newtype!(@repr { schema } meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype!(@select meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let ident = &meta.name;
        quote! {
            impl From<#ident> for SelectedNode {
                fn from(t: #ident) -> Self {
                    t.0.into()
                }
            }

            impl Into<Any> for #ident {
                fn into(self) -> Any {
                    self.0.into()
                }
            }
        }
    }
}
