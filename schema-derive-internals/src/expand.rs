use super::{InnerAttributes, OuterAttributes, ReprDefinition, SchemaDefinition, SchemaMeta};
use crate::{
    AdvancedBytesReprDefinition, AdvancedListReprDefinition, AdvancedMapReprDefinition,
    AdvancedStructReprDefinition,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Ident;

impl SchemaDefinition {
    pub fn expand(self) -> TokenStream {
        self.into_token_stream()
    }
}

macro_rules! expand_basic_def {
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

impl ToTokens for SchemaDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let meta = &self.meta;
        let repr = &self.repr;

        let repr_tokens: TokenStream = match repr {
            ReprDefinition::Null(def) => expand_basic_def!(meta, def),
            ReprDefinition::Bool(def) => expand_basic_def!(meta, def),
            ReprDefinition::Int(def) => expand_basic_def!(meta, def),
            ReprDefinition::Float(def) => expand_basic_def!(meta, def),
            ReprDefinition::String(def) => expand_basic_def!(meta, def),
            ReprDefinition::Link(def) => expand_basic_def!(meta, def),
            ReprDefinition::Copy(def) => expand_basic_def!(meta, def),
            ReprDefinition::Enum(def) => expand_basic_def!(meta, def),
            ReprDefinition::Union(def) => expand_basic_def!(meta, def),
            // possibly advanced reprs
            // ReprDefinition::Bytes
            // ReprDefinition::List
            // ReprDefinition::Map
            // ReprDefinition::Struct
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

    /// TODO? Derives `DeserializeSeed for Selector`
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream;

    #[doc(hidden)]
    fn impl_serialize(&self, meta: &SchemaMeta, body: TokenStream) -> TokenStream {
        let name = &meta.name;
        let ipld_schema = &meta.ipld_schema;
        quote! {
            impl #ipld_schema::dev::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: #ipld_schema::dev::Serializer,
                {
                    #body
                }
            }
        }
    }

    #[doc(hidden)]
    fn impl_visitor(
        &self,
        meta: &SchemaMeta,
        expecting: &'static str,
        body: TokenStream,
    ) -> (Ident, TokenStream) {
        let name = &meta.name;
        let ipld_schema = &meta.ipld_schema;
        let visitor = visitor_name(name);
        let method = quote! {
            struct #visitor;
            impl<'de> #ipld_schema::dev::Visitor<'de> for #visitor {
                type Value = #name;
                fn expecting(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    fmt.write_str(#expecting)
                }
                #body
            }
        };
        (visitor, method)
    }

    #[doc(hidden)]
    fn impl_deserialize(&self, meta: &SchemaMeta, body: TokenStream) -> TokenStream {
        let name = &meta.name;
        let ipld_schema = &meta.ipld_schema;
        quote! {
            impl<'de> #ipld_schema::dev::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: #ipld_schema::dev::Deserializer<'de>,
                {
                    #body
                }
            }
        }
    }

    #[doc(hidden)]
    fn impl_repr(&self, meta: &SchemaMeta, body: TokenStream) -> TokenStream {
        let name = &meta.name;
        let ipld_schema = &meta.ipld_schema;
        quote! {
            impl<Ctx: #ipld_schema::dev::Context + Sync> #ipld_schema::dev::Representation<Ctx> for #name {
                const NAME: &'static str = ::std::stringify!(#name);
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

pub fn visitor_name(name: &Ident) -> Ident {
    Ident::new(&format!("{}Visitor", &name.to_string()), Span::call_site())
}

pub fn selector_name(name: &Ident) -> Ident {
    Ident::new(&format!("{}Selector", &name.to_string()), Span::call_site())
}
