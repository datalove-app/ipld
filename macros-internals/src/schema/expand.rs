use super::*;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_macro_input, Ident};

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

    //     let meta = &self.meta;
    //     if meta.try_from.is_none() {
    //         return TokenStream::default();
    //     }

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
                let typedef = $def.define_type($meta);
                let serde = $def.derive_serde($meta);
                let repr = $def.derive_repr($meta);
                quote! {
                    #typedef
                    #serde
                    #repr
                }
            }};
        }

        let meta = &self.meta;
        let repr_tokens: TokenStream = match &self.repr {
            ReprDefinition::Null(def) => expand_basic!(meta, def),
            ReprDefinition::Bool(def) => expand_basic!(meta, def),
            ReprDefinition::Int(def) => expand_basic!(meta, def),
            ReprDefinition::Float(def) => expand_basic!(meta, def),
            ReprDefinition::String(def) => expand_basic!(meta, def),
            ReprDefinition::Link(def) => expand_basic!(meta, def),
            ReprDefinition::Copy(def) => expand_basic!(meta, def),
            ReprDefinition::Enum(def) => expand_basic!(meta, def),
            ReprDefinition::Union(def) => expand_basic!(meta, def),

            // possibly advanced reprs
            // ReprDefinition::Bytes(BytesReprDefinition::Advanced(def))
            // ReprDefinition::List(ListReprDefinition::Advamced(def))
            // ReprDefinition::Map(MapReprDefinition::Advamced(def))
            // ReprDefinition::Struct(StructReprDefinition::Advamced(def))

            // non-advanced reprs
            // ReprDefinition::Bytes(def) => expand_basic_def!(meta, def),
            // ReprDefinition::List(def) => expand_basic_def!(meta, def),
            // ReprDefinition::Map(def) => expand_basic_def!(meta, def),
            // ReprDefinition::Struct(def) => expand_basic_def!(meta, def),
            _ => unimplemented!(),
        };

        tokens.append_all(repr_tokens);
    }
}

///
#[allow(unused_variables)]
pub(crate) trait ExpandBasicRepresentation {
    /// Defines the type, and applies any provided attributes.
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derive Serde impls for the defined type.
    ///
    /// Optional because many types can use `#[derive(Deserialize, Serialize)]`
    /// directly.
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }

    /// Derives `Representation` for the defined type.
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream;

    /// - `type ReprSelectorVisitor = SelectorVisitor<ReprVisitor>`
    ///     `impl Visitor for `ReprSelectorVisitor`
    /// - define `IgnoredRepr ` type
    /// - `impl DeserializeSeed<'de, Value = Self> for Selector`
    ///     instantiates ReprSelectorVisitor(selector, repr_visitor)
    ///     matches on selector, delegates to one deserializer method
    /// - defines a `NewSelector` for the type, wrapping `Selector`
    ///     `impl DeserializeSeed for NewSelector`
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream;

    #[doc(hidden)]
    fn impl_serialize(&self, meta: &SchemaMeta, body: TokenStream) -> TokenStream {
        let name = &meta.name;
        let lib = &meta.ipld_schema_lib;
        quote! {
            #[automatically_derived]
            impl #lib::dev::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: #lib::dev::Serializer,
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
    #[doc(hidden)]
    fn impl_visitor(
        &self,
        meta: &SchemaMeta,
        expecting: &'static str,
        body: TokenStream,
    ) -> (Ident, TokenStream) {
        let name = &meta.name;
        let lib = &meta.ipld_schema_lib;
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
            impl<'de> #lib::dev::Visitor<'de> for #visitor {
                type Value = #name;

                fn expecting(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    fmt.write_str(#expecting)
                }

                #body
            }
        };

        (visitor, visitor_def)
    }

    #[doc(hidden)]
    fn impl_deserialize(&self, meta: &SchemaMeta, mut body: TokenStream) -> TokenStream {
        let name = &meta.name;
        let lib = &meta.ipld_schema_lib;

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
            impl<'de> #lib::dev::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: #lib::dev::Deserializer<'de>,
                {
                    #body
                }
            }
        }
    }

    #[doc(hidden)]
    fn impl_repr(&self, meta: &SchemaMeta, body: TokenStream) -> TokenStream {
        let name = &meta.name;
        let lib = &meta.ipld_schema_lib;
        let typedef_str = &meta.typedef_str;
        quote! {
            #[automatically_derived]
            impl #lib::dev::Representation for #name {
                const NAME: &'static str = ::std::stringify!(#name);
                const SCHEMA: &'static str = #typedef_str;
                #body
            }
        }
    }
}

/// Helper trait for crates that want to provide auto-implementable
/// Advanced IPLD representations.
///
/// Implementors need only implement one method.
#[allow(unused_variables)]
pub trait ExpandAdvancedRepresentation {
    /// Expands an advanced `bytes` representation definition into a `TokenStream`
    /// that of a type that implements `Representation`.
    fn expand_bytes(repr: AdvancedBytesReprDefinition) -> TokenStream {
        unimplemented!()
    }

    /// Expands an advanced `list` representation definition into a `TokenStream`
    /// that of a type that implements `Representation`.
    fn expand_list(repr: AdvancedListReprDefinition) -> TokenStream {
        unimplemented!()
    }

    /// Expands an advanced `map` representation definition into a `TokenStream`
    /// that of a type that implements `Representation`.
    fn expand_map(repr: AdvancedMapReprDefinition) -> TokenStream {
        unimplemented!()
    }

    /// Expands an advanced `struct` representation definition into a `TokenStream`
    /// that of a type that implements `Representation`.
    fn expand_struct(repr: AdvancedStructReprDefinition) -> TokenStream {
        unimplemented!()
    }
}

// fn expand_try_from_visitor_methods(tokens: TokenStream) -> TokenStream {
//     let tokens = tokens.into::<proc_macro::TokenStream>();
//     let methods = parse_macro_input!(tokens as super::Methods);

//     &methods.0.iter().map(|item_fn| {
//         let sig = &item_fn.sig;
//         let block = &item_fn.block;

//         quote! {
//             use ::std::convert::TryFrom;
//             // let t = #try_from_ident::deserialize(deserializer)?;
//             // Ok(#name::try_from(t).map_err(D::Error::custom)?)

//             #sig {
//                 let
//             }
//         }
//     });
//     tokens
// }
