use super::{
    BoolReprDefinition, BytesReprDefinition, CopyReprDefinition, FloatReprDefinition,
    IntReprDefinition, NullReprDefinition, StringReprDefinition,
};
use crate::{
    define_newtype,
    dev::{
        schema::{expand, DataModelKind},
        SchemaMeta,
    },
};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::Type;

impl expand::ExpandBasicRepresentation for NullReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let name = &meta.name;

        quote! {
            #(#attrs)*
            #[derive(Deserialize, Serialize)]
            #vis struct #name;
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        expand::impl_repr(
            meta,
            quote! {
                const KIND: Kind = Kind::Null;
                // const SCHEMA: &'static str = format!("pub type {} null;", Self::NAME);
            },
        )
    }

    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        expand::impl_select(
            meta,
            // quote! {
            //     // unimplemented!()
            //     Ok(Null::r#match(seed)?.map(|_| Self))
            // },
            quote! {
                unimplemented!()

                // if #lib::dev::type_eq::<Self, S>() {
                //     type_cast_selection::<Self, S, _, _>(|| {
                //         Ok(Null::select::<Null>(seed)?.map(|_| Self))
                //     })
                // } else {
                //     Null::select::<S>(seed)
                // }
            },
        )
    }

    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let ident = &meta.name;
        quote! {
            impl Into<Node> for #ident {
                fn into(self) -> Node {
                    Node::Null
                }
            }

            impl Into<Value> for #ident {
                fn into(self) -> Value {
                    Value::Null
                }
            }
        }
    }
}

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

            // impl ::std::ops::Deref for #ident {
            //     type Target = #$inner_ty;
            //     fn deref(&self) -> &Self::Target {
            //         &self.0
            //     }
            // }
        }
    }};
}

///
macro_rules! derive_newtype_repr {
    ($def:ident, $meta:ident => $inner_ty:ident) => {{
        expand::impl_repr(
            $meta,
            quote! {
                const KIND: Kind = <#$inner_ty as Representation>::KIND;
            },
        )
    }};
}

///
macro_rules! derive_newtype_select {
    // (@visitor_null $def:ident, $meta:ident => fn $visit_fn:ident) => {{
    //     expand::impl_context_seed_visitor($meta, quote! {
    //         #[inline]
    //         fn visit_unit<E>(self) -> Result<Self::Value, E>
    //         where
    //             E: serde::de::Error,
    //         {
    //             self.visit_primitive(())
    //         }
    //     })
    // }};
    (@visitor_primitive $meta:ident => $visit_fn:ident ($ty:ty) $($expecting:tt)*) => {{
        let name = &$meta.name;
        expand::impl_context_seed_visitor(
            $meta,
            $($expecting)*,
            quote! {
                #[inline]
                fn $visit_fn<E>(self, v: $ty) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_primitive(v)
                }
            },
        )
    }};
    (@deseed $meta:ident => $deserialize_fn:ident) => {{
        expand::impl_context_seed_deseed(
            $meta,
            quote! {
                deserializer.$deserialize_fn(self)
            },
        )
    }};
    (@select $def:ident, $meta:ident => $inner_ty:ident) => {{
        expand::impl_select(
            $meta,
            // quote! {
            //     unimplemented!()
            //     // Ok(#$inner_ty::r#match(seed)?.map(Self))
            // },
            quote! {
                unimplemented!()
                // if type_eq::<Self, S>() {
                //     type_cast_selection::<Self, S, _, _>(|| {
                //         let seed = seed.into::<#$inner_ty, #$inner_ty>();
                //         Ok(#$inner_ty::select::<#$inner_ty>(seed)?.map(Self))
                //     })
                // } else {
                //     #$inner_ty::select::<S>(seed)
                // }
            },
        )
    }};
}

///
macro_rules! derive_newtype_conv {
    ($def:ident, $meta:ident => $dm_ty:ident) => {{
        let ident = &$meta.name;

        quote! {
            impl Into<Node> for #ident {
                fn into(self) -> Node {
                    Node::#$dm_ty
                }
            }

            impl Into<Value> for #ident {
                fn into(self) -> Value {
                    Value::#$dm_ty
                }
            }
        }
    }};
    (@has_constructor $def:ident, $meta:ident => $dm_ty:ident) => {{
        let ident = &$meta.name;

        quote! {
            impl Into<Node> for #ident {
                fn into(self) -> Node {
                    Node::#$dm_ty(self.0)
                }
            }

            impl Into<Value> for #ident {
                fn into(self) -> Value {
                    Value::#$dm_ty(self.0)
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

impl BoolReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(bool))
    }
}

impl expand::ExpandBasicRepresentation for BoolReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype_repr!(self, meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();

        let mut tokens = derive_newtype_select!(@visitor_primitive
            meta => visit_bool(bool)
            "expecting a `bool`-ish type"
        );
        tokens.append_all(derive_newtype_select!(@select self, meta => inner_ty));
        tokens.append_all(derive_newtype_select!(@deseed meta => deserialize_bool));

        tokens
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = DataModelKind::Bool.to_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty)
    }
}

impl expand::ExpandBasicRepresentation for IntReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_repr!(self, meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_select!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = DataModelKind::Int.to_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty)
    }
}

impl expand::ExpandBasicRepresentation for FloatReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_repr!(self, meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_select!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = DataModelKind::Float.to_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty)
    }
}

impl StringReprDefinition {
    fn inner_ty() -> Type {
        Type::Verbatim(quote!(String))
    }
}

impl expand::ExpandBasicRepresentation for StringReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        define_newtype!(self, meta => inner_ty)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype_repr!(self, meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = Self::inner_ty();
        derive_newtype_select!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = DataModelKind::String.to_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty)
    }
}

impl expand::ExpandBasicRepresentation for CopyReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_repr!(self, meta => inner_ty)
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_ty = &self.0;
        derive_newtype_select!(@select self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = DataModelKind::String.to_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty)
    }
}
