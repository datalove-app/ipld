use super::*;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_macro_input, parse_quote, Ident};

impl SchemaDefinition {
    /// Expand this into a `TokenStream` of the IPLD Schema + Representation
    /// implementation.
    pub fn expand(self) -> TokenStream {
        self.into_token_stream()
    }

    // /// Expands to a private type identical to the one defined, which can be
    // /// used for deserializing and then converting into the main type.
    // fn expand_try_from(&self) -> TokenStream {
    //     macro_rules! expand {
    //         ($meta:ident, $def:ident) => {{
    //             let meta = $meta.to_try_from_meta();
    //             $def.define_type(&meta)
    //         }};
    //     }
    //
    //     let meta = &self.meta;
    //     if meta.try_from.is_none() {
    //         return TokenStream::default();
    //     }
    //
    //     match &self.repr {
    //         ReprDefinition::Int(def) => expand!(meta, def),
    //         ReprDefinition::Float(def) => expand!(meta, def),
    //         ReprDefinition::String(def) => expand!(meta, def),
    //         ReprDefinition::Bytes(BytesReprDefinition::Basic) => {
    //             let def = BytesReprDefinition::Basic;
    //             expand!(meta, def)
    //         }
    //         _ => TokenStream::default(),
    //     }
    // }
}

impl ToTokens for SchemaDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        macro_rules! expand_basic {
            ($meta:ident, $def:ident) => {{
                let name = &$meta.name;
                let typedef = $def.define_type($meta);
                let lib = &$meta.lib;

                let use_ipld = if $meta.internal {
                    quote! {
                        use crate as _ipld;
                        #[allow(unused_imports)]
                        use _ipld::dev::*;
                    }
                } else {
                    quote! {
                        // #[allow(clippy::useless_attribute)]
                        extern crate #lib as _ipld
                        #[allow(unused_imports)]
                        use _ipld::dev::*;
                    }
                };

                let defs = [
                    ("IMPL_SERDE", $def.derive_serde($meta)),
                    ("IMPL_REPR", $def.derive_repr($meta)),
                    ("IMPL_SELECT", $def.derive_select($meta)),
                    ("IMPL_CONV", $def.derive_conv($meta)),
                ];
                let scoped_impls = defs
                    .iter()
                    .map(|(kind, def)| {
                        (
                            Ident::new(&format!("_IPLD_{}_FOR_{}", kind, name), Span::call_site()),
                            def,
                        )
                    })
                    .map(|(ident, def)| {
                        quote! {
                            #[doc(hidden)]
                            const #ident: () = {
                                #use_ipld

                                #def
                            };
                        }
                    });

                quote! {
                    #typedef
                    #(#scoped_impls)*
                }
            }};
        }

        let meta = &self.meta;
        tokens.append_all::<TokenStream>(match &self.repr {
            // standard data model kinds
            ReprDefinition::Null(def) => expand_basic!(meta, def),
            ReprDefinition::Bool(def) => expand_basic!(meta, def),
            ReprDefinition::Int(def) => expand_basic!(meta, def),
            ReprDefinition::Float(def) => expand_basic!(meta, def),
            ReprDefinition::String(def) => expand_basic!(meta, def),
            ReprDefinition::Bytes(def) => expand_basic!(meta, def),
            ReprDefinition::List(def) => expand_basic!(meta, def),
            // ReprDefinition::Map(def) => expand_basic_def!(meta, def),
            ReprDefinition::Link(def) => expand_basic!(meta, def),

            // schema kinds
            ReprDefinition::Struct(def) => expand_basic!(meta, def),
            ReprDefinition::Enum(def) => expand_basic!(meta, def),
            ReprDefinition::Union(def) => expand_basic!(meta, def),
            ReprDefinition::Copy(def) => expand_basic!(meta, def),

            // advanced reprs
            // ReprDefinition::Bytes(BytesReprDefinition::Advanced(def))
            // ReprDefinition::List(ListReprDefinition::Advanced(def))
            // ReprDefinition::Map(MapReprDefinition::Advanced(def))
            // ReprDefinition::Struct(StructReprDefinition::Advanced(def))
            _ => unimplemented!(),
        });
    }
}

/// Helper trait for expanding a `SchemaDefinition` into a type and it's trait impls.
#[allow(unused_variables)]
pub(crate) trait ExpandBasicRepresentation {
    /// Defines the type, and applies any provided attributes.
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derives `Representation` for the defined type.
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derive Serde impls for the defined type, as well as core-logic types like [`ipld::context::ContextSeed`].
    ///
    /// Optional because many types can use `serde-derive`directly.
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }

    /// Derives `Select` for the type.
    ///
    /// TODO: support conditionals
    /// - `type ReprSelectorSeed = SelectorSeed<ReprVisitor>`
    ///     `impl Visitor for `ReprSelectorSeed`
    /// - `type IgnoredT = IgnoredRepr<T>`
    /// - defines a `NewSelector` for the type, wrapping `Selector`
    ///     `impl DeserializeSeed for NewSelector`
    /// - `impl DeserializeSeed<'de, Value = Self> for Selector`
    ///     instantiates ReprSelectorSeed(selector, repr_visitor)
    ///     matches on selector, delegates to one deserializer method
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derives conversions between the type and `Value`, as well as `ipfs::Ipld`
    /// (if `#[cfg(feature = "ipld/ipfs")]` is enabled)
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream;
}

/// Helper trait for crates that want to provide auto-implementable
/// Advanced IPLD representations.
///
/// Implementors need only implement one method.
#[allow(unused_variables)]
pub trait ExpandAdvancedRepresentation {
    /// Expands an advanced `bytes` representation definition into a `TokenStream`
    /// of a type that implements `Representation`.
    fn expand_bytes(repr: AdvancedBytesReprDefinition) -> TokenStream {
        unimplemented!()
    }

    /// Expands an advanced `list` representation definition into a `TokenStream`
    /// of a type that implements `Representation`.
    fn expand_list(repr: AdvancedListReprDefinition) -> TokenStream {
        unimplemented!()
    }

    /// Expands an advanced `map` representation definition into a `TokenStream`
    /// of a type that implements `Representation`.
    fn expand_map(repr: AdvancedMapReprDefinition) -> TokenStream {
        unimplemented!()
    }

    /// Expands an advanced `struct` representation definition into a `TokenStream`
    /// of a type that implements `Representation`.
    fn expand_struct(repr: AdvancedStructReprDefinition) -> TokenStream {
        unimplemented!()
    }
}

impl SchemaKind {
    pub(crate) fn data_model_kind(&self) -> Ident {
        Ident::new(
            match self {
                Self::Null => "Null",
                Self::Bool => "Bool",
                Self::Int
                | Self::Int8
                | Self::Int16
                | Self::Int32
                | Self::Int64
                | Self::Int128
                | Self::Uint8
                | Self::Uint16
                | Self::Uint32
                | Self::Uint64
                | Self::Uint128 => "Int",
                Self::Float | Self::Float32 | Self::Float64 => "Float",
                Self::Bytes => "Bytes",
                Self::String => "String",
                Self::List => "List",
                Self::Map => "Map",
                Self::Link => "Link",
                Self::Struct => "Struct",
                Self::Union => "Union",
                Self::Enum => "Enum",
                Self::Copy => "Copy",
            },
            Span::call_site(),
        )
    }

    pub(crate) fn selected_node_ident(&self) -> Ident {
        Ident::new(
            match self {
                Self::Null => "Null",
                Self::Bool => "Bool",
                Self::Int => "Int64",
                Self::Int8 => "Int8",
                Self::Int16 => "Int16",
                Self::Int32 => "Int32",
                Self::Int64 => "Int64",
                Self::Int128 => "Int128",
                Self::Uint8 => "Uint8",
                Self::Uint16 => "Uint16",
                Self::Uint32 => "Uint32",
                Self::Uint64 => "Uint64",
                Self::Uint128 => "Uint128",
                Self::Float => "Float64",
                Self::Float32 => "Float32",
                Self::Float64 => "Float64",
                Self::Bytes => "Bytes",
                Self::String => "String",
                Self::List => "List",
                Self::Map => "Map",
                Self::Link => "Link",
                Self::Struct => "Struct",
                Self::Union => "Union",
                Self::Enum => "Enum",
                Self::Copy => "Copy",
            },
            Span::call_site(),
        )
    }
}

// Helpers

/// Helpers for newtype wrappers around types that already implement
/// `Serialize`, `Deserialize`, `Representation` and `Select`.
/// TODO: manually/macro implement serialize/deserialize for these types
#[macro_export(local_inner_macros)]
macro_rules! derive_newtype {
    (@typedef $def:ident, $meta:ident => $inner_ty:ident $(#[$attr_ex:meta])*) => {{
        let attrs = &$meta.attrs;
        let vis = &$meta.vis;
        let name = &$meta.name;
        let generics = &$meta
            .generics
            .as_ref()
            .map(|g| quote::quote!(#g))
            .unwrap_or_default();

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
            $(#[$attr_ex])*
            #vis struct #name #generics (#$inner_ty);
        }
    }};
    (@typedef_transparent $def:ident, $meta:ident => $inner_ty:ident) => {{
        $crate::derive_newtype! { @typedef
            $def, $meta => $inner_ty
            #[derive(Deserialize, Serialize)]
            #[serde(transparent)]
        }
    }};
    (@repr { $tokens:tt } $meta:ident => $inner_ty:ident) => {{
        $crate::dev::impl_repr(
            $meta,
            quote::quote! {
                #$tokens
                const DATA_MODEL_KIND: Kind = <#$inner_ty>::DATA_MODEL_KIND;
                const SCHEMA_KIND: Kind = <#$inner_ty>::SCHEMA_KIND;
                const REPR_KIND: Kind = <#$inner_ty>::REPR_KIND;
                const IS_LINK: bool = <#$inner_ty>::IS_LINK;
                const HAS_LINKS: bool = <#$inner_ty>::HAS_LINKS;

                #[inline]
                fn data_model_kind(&self) -> Kind {
                    self.0.data_model_kind()
                }
                #[inline]
                fn schema_kind(&self) -> Kind {
                    self.0.schema_kind()
                }
                #[inline]
                fn repr_kind(&self) -> Kind {
                    self.0.repr_kind()
                }
                #[inline]
                fn has_links(&self) -> bool {
                    self.0.has_links()
                }

                #[inline]
                #[doc(hidden)]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    Representation::serialize::<C, _>(self.0, serializer)
                }

                #[inline]
                #[doc(hidden)]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    Ok(Self(Representation::deserialize::<C, _>(deserializer)?))
                }
            }
        )
    }};
    (@select $meta:ident => $inner_ty:ident) => {{
        let lib = &$meta.lib;
        let name = &$meta.name;
        quote::quote! {
            #lib::dev::macros::impl_selector_seed_serde! { @selector_seed_codec_deseed_newtype {} {} #name as #$inner_ty
            }
            #lib::dev::macros::impl_selector_seed_serde! {
                @selector_seed_select {} {} #name
            }
        }
    }};
    (@conv @has_constructor $def:ident, $meta:ident =>
        $dm_ty:ident $selected_node:ident) => {{
        let name = &$meta.name;
        quote::quote! {
            impl From<#name> for SelectedNode {
                fn from(t: #name) -> Self {
                    Self::#$selected_node(t.0.into())
                }
            }

            impl Into<Any> for #name {
                fn into(self) -> Any {
                    Any::#$dm_ty(self.0.into())
                }
            }

            impl TryFrom<Any> for #name {
                type Error = Error;
                fn try_from(any: Any) -> Result<Self, Self::Error> {
                    match any {
                        Any::#$dm_ty(inner) => Ok(Self(inner.into())),
                        _ => Err(Error::MismatchedAny)
                    }
                }
            }
        }
    }};
}

///
pub(crate) fn impl_serialize(meta: &SchemaMeta, body: TokenStream) -> TokenStream {
    let name = &meta.name;
    quote! {
        #[automatically_derived]
        impl Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #body
            }
        }
    }
}

// TODO: for each visitor method, incorporate try_from...
// - define TryFromType
//      - `impl Visitor<Value = TryFromType> for ReprVisitor`
//      - method bodies expanded (should only refer to Self::Value, not #name)
// - in `impl Visitor<Value = Repr> for ReprVisitor`, replacing method bodies
//      - call each equiv method on <Self as Visitor<TryFromType>>
//      - handle result with try_from().map_err() call
pub(crate) fn impl_visitor(
    meta: &SchemaMeta,
    expecting: &'static str,
    body: TokenStream,
) -> (Ident, TokenStream) {
    let name = &meta.name;
    let visitor = meta.visitor_name();

    // TODO? if try_from, add:
    //  - def ReprVisitor
    //  - def TryFromType
    //  - `impl Visitor<Value = TryFromType> for ReprVisitor`
    //      - body
    //  - `impl Visitor<Value = Repr> for ReprVisitor`
    //      - body replaced w/ try_from call
    // TODO? else:
    //  - def ReprVisitor
    //  - `impl Visitor<Value = Repr> for ReprVisitor`
    //
    // body = if let Some(try_from_name) = &meta.try_from {
    //     let try_from_ident = Ident::new(&try_from_name.value(), Span::call_site());
    //     let methods = expand_try_from_visitor_methods(body, try_from_ident);
    //     quote! {
    //             use ::std::convert::TryFrom;
    //             // let t = #try_from_ident::deserialize(deserializer)?;
    //             // Ok(#name::try_from(t).map_err(D::Error::custom)?)
    //     }
    // } else {
    //     quote! {
    //         type Value = #name;
    //         #body
    //     }
    // };

    let visitor_def = quote! {
        struct #visitor;

        #[automatically_derived]
        impl<'de> Visitor<'de> for #visitor {
            type Value = #name;
            fn expecting(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                fmt.write_str(#expecting)
            }
            #body
        }
    };

    (visitor, visitor_def)
}

pub(crate) fn impl_visitor_ext(meta: &SchemaMeta, body: Option<TokenStream>) -> TokenStream {
    let visitor = meta.visitor_name();
    quote! {
        #[automatically_derived]
        impl<'de> IpldVisitorExt<'de> for #visitor {
            #body
        }
    }
}

pub(crate) fn impl_deserialize(meta: &SchemaMeta, mut body: TokenStream) -> TokenStream {
    let name = &meta.name;
    // let lib = &meta.ipld_schema_lib;

    // body = if let Some(try_from_name) = &meta.try_from {
    //     let try_from_ident = Ident::new(&try_from_name.value(), Span::call_site());
    //     // let methods = expand_try_from_visitor_methods(body, try_from_ident);
    //     quote! {
    //     use ::std::convert::TryFrom;
    //     let t = #try_from_ident::deserialize(deserializer)?;
    //     Ok(#name::try_from(t).map_err(D::Error::custom)?)
    //     }
    // } else {
    //     body
    // };

    quote! {
        #[automatically_derived]
        impl<'de> Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #body
            }
        }
    }
}

pub(crate) fn impl_repr(meta: &SchemaMeta, consts_and_simple_methods: TokenStream) -> TokenStream {
    let lib = &meta.lib;
    let name = &meta.name;
    let typedef_str = &meta.typedef_str;
    let generics = meta.generics_tokens();
    quote! {
        #[automatically_derived]
        impl #generics #lib::dev::Representation for #name #generics {
            const NAME: &'static str = ::std::stringify!(#name);
            // const SCHEMA: &'static str = #typedef_str;

            #consts_and_simple_methods
        }
    }
}

/*
pub(crate) fn impl_context_seed_visitor(
    meta: &SchemaMeta,
    expecting: &'static str,
    mut body: TokenStream,
) -> TokenStream {
    let name = &meta.name;
    let generics = meta.generics_tokens();

    quote! {
        #[automatically_derived]
        impl<'a, 'de, #generics> Visitor<'de> for ContextSeed<'a, C, #name>
        where
            C: Context
            U: Representation,
        {
            type Value = ();

            fn expecting(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                fmt.write_str(#expecting)
            }

            #body
        }
    }
}

pub(crate) fn impl_context_seed_deseed(meta: &SchemaMeta, mut body: TokenStream) -> TokenStream {
    let name = &meta.name;
    let generics = meta.generics_tokens();

    quote! {
        #[automatically_derived]
        impl<'a, 'de, #generics> DeserializeSeed<'de> for ContextSeed<'a, C, #name>
        where
            C: Context
            U: Representation,
            ContextSeed<'a, C, #name>: Visitor<'de, Value = ()>,
        {
            type Value = ();

            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                #body
            }
        }
    }
}

pub(crate) fn impl_select(
    meta: &SchemaMeta,
    // match_impl: TokenStream,
    select_impl: TokenStream,
) -> TokenStream {
    let name = &meta.name;
    let generics = meta.generics_tokens();
    quote! {
        #[automatically_derived]
        impl<Ctx: Context> Select<Ctx> for #name {
            // fn r#match(
            //     // selector: &Selector,
            //     // state: &mut SelectorState,
            //     // params: SelectionParams<'_, Ctx, Self>,
            //     // ctx: &mut Ctx,
            //     seed: ContextSeed<'_, Ctx, Self>,
            // ) -> Result<Option<Self>, Error> {
            //     #match_impl
            // }

            /// Produces a stream of [`Selection`]s.
            fn select(params: SelectionParams<'_, Ctx, Self>, ctx: &mut Ctx) -> Result<(), Error> {
                #select_impl
            }
        }
    }
}

 */

//
// pub(crate) fn impl_de_seed_for(
//     meta: &SchemaMeta,
//     selector: Ident,
//     body: TokenStream,
// ) -> TokenStream {
//     let name = &meta.name;
//     // let lib = &meta.ipld_schema_lib;
//     quote! {
//         #[automatically_derived]
//         impl<'de> DeserializeSeed<'de> for SelectorSeed<'de, #selector, #name> {
//             type Value = #name;
//             fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//             where
//                 D: Deserializer<'de>,
//             {
//                 #body
//             }
//         }
//     }
// }

// pub(crate) fn impl_primtive_de_seed(meta: &SchemaMeta) -> TokenStream {
//     let name = &meta.name;
//     let lib = &meta.ipld_schema_lib;
//     impl_de_seed(
//         meta,
//         quote! {
//             match (&self).as_selector() {
//                 Selector::Matcher(sel) => <#name as de::Deserialize<'de>>::deserialize(deserializer),
//                 Selector::ExploreConditional(sel) => {
//                     use std::borrow::Borrow;
//                     let ExploreConditional { condition, next } = sel.borrow();
//                     unimplemented!()
//                 },
//                 sel => Err(de::Error::custom(
//                     #lib::Error::Selector::invalid_selector::<#name>(sel)
//                 )),
//             }
//         },
//     )
// }

// fn expand_try_from_visitor_methods(tokens: TokenStream) -> TokenStream {
//     let tokens = tokens.into::<proc_macro::TokenStream>();
//     let methods = parse_macro_input!(tokens as super::Methods);
//
//     &methods.0.iter().map(|item_fn| {
//         let sig = &item_fn.sig;
//         let block = &item_fn.block;
//
//         quote! {
//             use ::std::convert::TryFrom;
//             // let t = #try_from_ident::deserialize(deserializer)?;
//             // Ok(#name::try_from(t).map_err(D::Error::custom)?)
//
//             #sig {
//                 let
//             }
//         }
//     });
//     tokens
// }
