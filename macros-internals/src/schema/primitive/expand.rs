use super::{
    BoolReprDefinition, BytesReprDefinition, CopyReprDefinition, FloatReprDefinition,
    IntReprDefinition, NullReprDefinition, StringReprDefinition,
};
use crate::{
    define_newtype,
    dev::{
        schema::{expand, SchemaKind},
        SchemaMeta,
    },
};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::Type;

/// Defines a newtype wrapper around an inner type already implementing
/// `Serialize`, `Deserialize`, and `Representation`.
/// TODO: manually/macro implement serialize/deserialize for these types
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! define_newtype {
    ($def:ident, $meta:ident => $inner_ty:ident) => {{
        let attrs = &$meta.attrs;
        let vis = &$meta.vis;
        let name = &$meta.name;

        // let (try_from_typedef, try_from_serde_attr) = if let Some(try_from_name) = &$meta.try_from {
        //     let ident = &$meta.try_from_name();
        //     (
        //         // creates an inner type that transparently (de)serializes itself
        //         quote::quote! {
        //             #[derive(_ipld::dev::Deserialize, _ipld::dev::Serialize)]
        //             #[serde(transparent)]
        //             struct #ident(#$inner_ty);
        //         },
        //         // tells serde to delegate to a user-defined TryFrom impl
        //         quote::quote!(#[serde(try_from = #try_from_name)]),
        //     )
        // } else {
        //     (TokenStream::default(), TokenStream::default())
        // };


        quote::quote! {
            #(#attrs)*
            #[repr(transparent)]
            #[derive(Deserialize, Serialize)]
            #[serde(transparent)]
            // #try_from_serde_attr
            #vis struct #name(#$inner_ty);
        }
    }};
}

///
// macro_rules! derive_newtype_repr {
//     ($def:ident, $meta:ident => $inner_ty:ident { $($rest:tt)* }) => {{
//         expand::impl_repr(
//             $meta,
//             quote! {
//                 const DATA_MODEL_KIND: Kind = <#$inner_ty as Representation>::DATA_MODEL_KIND;

//                 $($rest)*
//             },
//         )
//     }};
// }

///
macro_rules! derive_newtype_select {
    ($meta:ident => $inner_ty:ident) => {{
        let name = &$meta.name;
        quote::quote! {
            impl_ipld_serde! {@context_seed_deseed_newtype {} {} #name as #$inner_ty }
            impl_ipld_serde! {@context_seed_select {} {} #name }
            // impl_ipld_serde! {@select_newtype {} {} #name as #$inner_ty}
        }
    }}; // (@visitor_null $def:ident, $meta:ident => fn $visit_fn:ident) => {{
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

///
macro_rules! derive_newtype_conv {
    (@has_constructor $def:ident, $meta:ident =>
        $dm_ty:ident $selected_node:ident) => {{
        let ident = &$meta.name;

        quote::quote! {
            impl From<#ident> for SelectedNode {
                fn from(t: #ident) -> Self {
                    Self::#$selected_node(t.0)
                }
            }

            impl Into<Any> for #ident {
                fn into(self) -> Any {
                    Any::#$dm_ty(self.0)
                }
            }
        }
    }};
}

// macro_rules! expand_def {
//     ($def:ident, $inner_ty:ident) => {
//         impl expand::ExpandBasicRepresentation for $def {
//             fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
//                 define_newtype!(self, meta => $inner_ty)
//             }
//             fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
//                 derive_newtype_repr!(self, meta => $inner_ty)
//             }
//             fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
//                 unimplemented!()
//             }
//         }
//     };
// }

impl NullReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(Null))
    }
}

impl expand::ExpandBasicRepresentation for NullReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let name = &meta.name;

        quote! {
            #(#attrs)*
            #[derive(Deserialize, Serialize)]
            #vis struct #name(Null);
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Null;
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME), " null");
            },
        )
    }

    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype_select!(meta => inner_ty)
    }

    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let ident = &meta.name;
        quote! {
            impl From<Null> for #ident {
                fn from(_: Null) -> Self {
                    Self(Null)
                }
            }

            impl From<#ident> for SelectedNode {
                fn from(t: #ident) -> Self {
                    Self::Null
                }
            }

            impl Into<Any> for #ident {
                fn into(self) -> Any {
                    Any::Null(Null)
                }
            }
        }
    }
}

impl BoolReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(Bool))
    }
}

impl expand::ExpandBasicRepresentation for BoolReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Bool;
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME), " bool");
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype_select!(meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = SchemaKind::Bool.data_model_kind();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty dm_ty)
    }
}

impl expand::ExpandBasicRepresentation for IntReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Int;
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME), " int");
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_select!(meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = self.1.data_model_kind();
        let sn_ty = self.1.selected_node_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty sn_ty)
    }
}

impl expand::ExpandBasicRepresentation for FloatReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Float;
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME), " float");
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_select!(meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = self.1.data_model_kind();
        let sn_ty = self.1.selected_node_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty sn_ty)
    }
}

impl StringReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(IpldString))
    }
}

impl expand::ExpandBasicRepresentation for StringReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::String;
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME), " string");
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype_select!(meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = SchemaKind::String.data_model_kind();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty dm_ty)
    }
}

impl expand::ExpandBasicRepresentation for CopyReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Null;
                const SCHEMA: &'static str = concat!("type ", stringify!(Self::NAME),  " = ", stringify!(<#inner_ty>::NAME));
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_select!(meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        // let ident = meta.name;
        // let dm_ty = SchemaKind::String.to_ident();
        // derive_newtype_conv!(@has_constructor self, meta => dm_ty)
        quote! {
            // impl Into<SelectedNode> for #ident {
            //     fn into(self) -> SelectedNode {
            //         SelectedNode::#$selected_node
            //     }
            // }

            // impl Into<Any> for #ident {
            //     fn into(self) -> Any {
            //         Any::#$dm_ty
            //     }
            // }
        }
    }
}
