use super::*;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_macro_input, parse_quote, Ident};

impl SchemaMeta {}

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
                let imports = SchemaMeta::imports($meta.internal);
                let scope = Ident::new(&format!("_IPLD_FOR_{}", &$meta.name), Span::call_site());
                let typedef = $def.define_type($meta);
                let defs = [
                    $def.derive_repr($meta),
                    $def.derive_select($meta),
                    $def.derive_conv($meta),
                ];

                quote! {
                    #typedef
                    const #scope: () = {
                        #imports
                        #(#defs)*
                    };
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
            ReprDefinition::Map(def) => expand_basic!(meta, def),
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
    ///
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        Default::default()
    }

    /// Defines the type, and applies any provided attributes.
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derives `Representation` for the defined type.
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derives `Select` for the type.
    ///
    /// TODO: support conditionals
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream;

    /// Derives conversions between the type and `Value`, as well as `ipfs::Ipld`
    /// (if `#[cfg(feature = "ipld/ipfs")]` is enabled)
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream;

    fn impl_repr(
        &self,
        meta: &SchemaMeta,
        consts: TokenStream,
        methods: TokenStream,
    ) -> TokenStream {
        let name = &meta.name;
        let generics = meta.generics_tokens();

        let name_str = meta.name_str();
        let schema = self.schema(meta);
        quote! {
            #[automatically_derived]
            impl #generics Representation for #name #generics {
                const NAME: &'static str = #name_str;
                const SCHEMA: &'static str = concat!(#schema);
                #consts
                #methods
            }
        }
    }

    fn impl_select(&self, meta: &SchemaMeta, rest: Option<TokenStream>) -> TokenStream {
        let methods = rest.unwrap_or(quote::quote! {
            #[doc(hidden)]
            #[inline]
            fn __select_de<'a, 'de, const C: u64, D>(
                seed: SelectorSeed<'a, Ctx, Self>,
                deserializer: D,
            ) -> Result<(), D::Error>
            where
                D: Deserializer<'de>,
            {
                Seed::<C, _, Self>::from(seed).deserialize(deserializer)
            }
        });

        let name = &meta.name;
        // let (impl_gen, ty_gen, where_gen) = match &meta.generics {
        //     Some(generics) => generics.split_for_impl(),
        //     None => Generics::default().split_for_impl(),
        // };
        quote::quote! {
            #[automatically_derived]
            impl<Ctx> Select<Ctx> for #name
            where
                Ctx: Context,
            {
                #methods
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
            match *self {
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
                _ => unreachable!(),
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
    (@repr $def:ident, $meta:ident => $inner_ty:ident { $consts:tt }) => {{
        $def.impl_repr($meta,
            quote::quote! {
                #$consts
                const HAS_LINKS: bool = <#$inner_ty>::HAS_LINKS;
            }, quote::quote! {
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
            fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: Serializer,
            {
                Representation::serialize::<MC, Se>(&self.0, serializer)
            }
            #[inline]
            fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
            where
                De: Deserializer<'de>,
            {
                Ok(Self(Representation::deserialize::<MC, De>(deserializer)?))
            }
        })
    }};
    (@select $def:ident, $meta:ident => $inner_ty:ident) => {{
        let name = &$meta.name;
        $def.impl_select($meta, Some(quote::quote! {
            #[doc(hidden)]
            #[inline]
            fn __select<'a>(
                seed: SelectorSeed<'a, Ctx, Self>,
            ) -> Result<(), Error> {
                let seed = seed.wrap::<#$inner_ty, _>(#name);
                <#$inner_ty as Select<Ctx>>::__select(seed)
            }

            #[doc(hidden)]
            #[inline]
            fn __select_de<'a, 'de, const C: u64, D>(
                seed: SelectorSeed<'a, Ctx, Self>,
                deserializer: D,
            ) -> Result<(), D::Error>
            where
                D: Deserializer<'de>,
            {
                let seed = seed.wrap::<#$inner_ty, _>(#name);
                <#$inner_ty as Select<Ctx>>::__select_de::<C, D>(seed, deserializer)
            }
        }))
    }};
    (@conv @has_constructor $def:ident, $meta:ident) => {{
        let name = &$meta.name;
        quote::quote! {
            #[automatically_derived]
            impl From<#name> for SelectedNode {
                fn from(t: #name) -> Self {
                    t.0.into()
                }
            }
            #[automatically_derived]
            impl Into<Any> for #name {
                fn into(self) -> Any {
                    self.0.into()
                }
            }
            #[automatically_derived]
            impl TryFrom<Any> for #name {
                type Error = Error;
                fn try_from(any: Any) -> Result<Self, Self::Error> {
                    let variant = Representation::name(&any);
                    let inner = TryFrom::try_from(any)
                        .map_err(|_| Error::failed_any_conversion::<Self>(variant))?;
                    Ok(Self(inner))
                }
            }
        }
    }}
}
