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
use quote::quote;
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
            quote! {
                Ok(Null::r#match(selector, state, ctx)?.map(|_| Self))
            },
            quote! {
                if #lib::dev::type_eq::<Self, S>() {
                    type_cast_selection::<Self, S, _, _>(|| {
                        Ok(Null::select::<Null>(selector, state, ctx)?.map(|_| Self))
                    })
                } else {
                    Null::select::<S>(selector, state, ctx)
                }
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
    ($def:ident, $meta:ident => $inner_ty:ident) => {{
        expand::impl_select(
            $meta,
            quote! {
                Ok(#$inner_ty::r#match(selector, state, ctx)?.map(Self))
            },
            quote! {
                if type_eq::<Self, S>() {
                    type_cast_selection::<Self, S, _, _>(|| {
                        Ok(#$inner_ty::select::<#$inner_ty>(selector, state, ctx)?.map(Self))
                    })
                } else {
                    #$inner_ty::select::<S>(selector, state, ctx)
                }
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
        derive_newtype_select!(self, meta => inner_ty)
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
        derive_newtype_select!(self, meta => inner_ty)
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
        derive_newtype_select!(self, meta => inner_ty)
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
        derive_newtype_select!(self, meta => inner_ty)
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
        derive_newtype_select!(self, meta => inner_ty)
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_ty = DataModelKind::String.to_ident();
        derive_newtype_conv!(@has_constructor self, meta => dm_ty)
    }
}
