//! `schema!` macro.

pub(crate) mod expand;
pub(crate) mod parse;

mod r#enum;
mod primitive;
mod recursive;
mod r#struct;
mod union;

pub use primitive::*;
pub use r#enum::*;
pub use r#struct::*;
pub use recursive::*;
pub use union::*;

use proc_macro2::Span;
use std::ops::Deref;
use syn::{Attribute, Ident, ItemFn, LitStr, Path, Visibility};

///
#[derive(Debug, Clone)]
pub struct SchemaMeta {
    typedef_str: String,
    ipld_schema_lib: Path,
    try_from: Option<LitStr>,
    attrs: OuterAttributes,
    vis: Visibility,
    name: Ident,
}

impl SchemaMeta {
    pub fn visitor_name(&self) -> Ident {
        Ident::new(
            &format!("{}Visitor", &self.name.to_string()),
            Span::call_site(),
        )
    }

    pub fn selector_name(&self) -> Ident {
        Ident::new(
            &format!("{}SelectorVisitor", &self.name.to_string()),
            Span::call_site(),
        )
    }
}

///
#[derive(Debug)]
pub struct SchemaDefinition {
    meta: SchemaMeta,
    repr: ReprDefinition,
}

///
#[derive(Debug)]
pub enum ReprDefinition {
    Null(NullReprDefinition),
    Bool(BoolReprDefinition),
    Int(IntReprDefinition),
    Float(FloatReprDefinition),
    String(StringReprDefinition),
    Bytes(BytesReprDefinition),
    Link(LinkReprDefinition),
    Copy(CopyReprDefinition),
    List(ListReprDefinition),
    Map(MapReprDefinition),
    Struct(StructReprDefinition),
    Enum(EnumReprDefinition),
    Union(UnionReprDefinition),
}

macro_rules! attr_vec {
    ($name:ident, $parse_fn:ident) => {
        /// Wrapper around a vec of `syn::Attribute`.
        #[derive(Debug, Clone)]
        pub struct $name(Vec<Attribute>);

        impl Deref for $name {
            type Target = Vec<Attribute>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl syn::parse::Parse for $name {
            fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
                let attrs = input.call(Attribute::$parse_fn)?;
                Ok(Self(attrs))
            }
        }
        impl quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                for attr in self.iter() {
                    attr.to_tokens(tokens);
                }
            }
        }
    };
}

attr_vec!(OuterAttributes, parse_outer);
attr_vec!(InnerAttributes, parse_inner);

/// Wrapper around a vec of fields
#[derive(Debug)]
pub struct Fields<T>(Vec<T>);

impl<T> Deref for Fields<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! define_keywords {
    ($($kw:ident)*) => {
        $(::syn::custom_keyword!($kw);)*
    };
}

/// Keywords unique to IPLD Schemas and Representations
#[macro_use]
pub(crate) mod kw {
    /// Helper macro for parsing a keyword-argument pair from IPLD schemas.
    #[macro_export(local_inner_macros)]
    macro_rules! parse_kwarg {
        ($input:ident, $kw:ident => $type:ty) => {{
            $input.parse::<crate::schema::kw::$kw>()?;
            $input.parse::<$type>()?
        }};
    }

    crate::define_keywords! {
        // main keywords
        representation advanced
        // schema data types
        null bool int float bytes string list map link copy
        // representation types
        listpairs stringjoin stringpairs
        tuple keyed kinded envelope inline byteprefix
        // representation type args
        field nullable optional implicit rename
        join innerDelim entryDelim
        fieldOrder discriminantKey contentKey
    }
}

// Exported Attributes

pub(crate) const CRATE_NAME: &'static str = "ipld-macros";
pub(crate) const INTERNAL: &'static str = "ipld_macros_internal";
pub(crate) const TRY_FROM: &'static str = "ipld_macros_try_from";

#[doc(hidden)]
#[macro_export]
macro_rules! def_attributes {
    () => {
        #[doc(hidden)]
        #[proc_macro_attribute]
        pub fn ipld_macros_internal(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }

        #[proc_macro_attribute]
        pub fn ipld_macros_try_from(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    };
}

impl SchemaMeta {
    fn to_try_from_meta(&self) -> Self {
        SchemaMeta {
            typedef_str: String::new(),
            ipld_schema_lib: self.ipld_schema_lib.clone(),
            try_from: None,
            attrs: self.attrs.clone(),
            vis: Visibility::Inherited,
            name: self.try_from_name(),
        }
    }

    fn try_from_name(&self) -> Ident {
        let try_from_name = self.try_from.as_ref().unwrap().value();
        Ident::new(&try_from_name, Span::call_site())
    }
}

impl ReprDefinition {
    fn supports_try_from(&self) -> bool {
        match self {
            Self::Int(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::Bytes(BytesReprDefinition::Basic) => true,
            _ => false,
        }
    }
}

impl OuterAttributes {
    fn is_internal_attr(attr: &Attribute) -> bool {
        attr.path.is_ident(INTERNAL) || attr.path.is_ident(TRY_FROM)
    }

    fn omit_internal_attrs(self) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|attr| !Self::is_internal_attr(attr))
                .collect::<Vec<Attribute>>(),
        )
    }
}

struct Methods(Vec<ItemFn>);
