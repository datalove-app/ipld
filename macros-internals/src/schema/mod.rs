//! `schema!` macro.

mod display;
pub(crate) mod expand;
pub(crate) mod parse;

mod compound;
mod r#enum;
mod primitive;
mod r#struct;
mod union;

pub use compound::*;
pub use expand::*;
pub use parse::*;
pub use primitive::*;
pub use r#enum::*;
pub use r#struct::*;
pub use union::*;

use crate::{common::*, dev::*};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use std::ops::Deref;
use syn::{parse_str, Attribute, Generics, Ident, LitStr, Path, Type, Visibility};

///
#[derive(Debug)]
pub struct SchemaDefinition {
    meta: SchemaMeta,
    repr: ReprDefinition,
}

///
#[derive(Debug, Clone)]
pub struct SchemaMeta {
    pub lib: TokenStream,
    pub typedef_str: String,
    pub internal: bool,
    pub try_from: Option<LitStr>,
    pub attrs: OuterAttributes,
    pub vis: Visibility,
    pub name: Ident,
    pub generics: Option<Generics>,
}

impl SchemaMeta {
    /// Creates a `TokenStream` of the `ipld` lib used (either `crate` or the `ipld` crate name).
    pub fn lib(is_internal: bool) -> TokenStream {
        if is_internal {
            return quote!(crate);
        }

        let path = crate_name(attr::IPLD_CRATE_NAME).map_or(
            Path::from(Ident::new(attr::IPLD_CRATE_NAME, Span::call_site())),
            |found_crate_name| {
                match found_crate_name {
                    FoundCrate::Itself => parse_str::<Path>(attr::IPLD_CRATE_NAME),
                    FoundCrate::Name(name) => parse_str::<Path>(&name),
                }
                .expect("`ipld` is not present in Cargo.toml")
            },
        );

        quote!(#path)
    }

    /// Returns the `Ident` of the `Visitor` generated for this type.
    pub fn visitor_name(&self) -> Ident {
        Ident::new(
            &format!("_{}Visitor", &self.name.to_string()),
            Span::call_site(),
        )
    }

    pub fn generics_tokens(&self) -> TokenStream {
        self.generics
            .as_ref()
            .map(|g| quote!(#g))
            .unwrap_or(TokenStream::default())
    }

    // fn to_try_from_meta(&self) -> Self {
    //     SchemaMeta {
    //         typedef_str: String::new(),
    //         ipld_schema_lib: self.ipld_schema_lib.clone(),
    //         try_from: None,
    //         attrs: self.attrs.clone(),
    //         vis: Visibility::Inherited,
    //         name: self.try_from_name(),
    //         generics: self.generics.clone(),
    //     }
    // }
    //
    // fn try_from_name(&self) -> Ident {
    //     let try_from_name = self.try_from.as_ref().unwrap().value();
    //     Ident::new(&try_from_name, Span::call_site())
    // }
}

// #[derive(Debug, Clone)]
// pub struct SchemaAttrs {
//     lib: Path,
//     try_from: Option<LitStr>,
// }

///
#[derive(Debug)]
pub enum ReprDefinition {
    Null(NullReprDefinition),
    Bool(BoolReprDefinition),
    Int(IntReprDefinition),
    Float(FloatReprDefinition),
    String(StringReprDefinition),
    Bytes(BytesReprDefinition),
    List(ListReprDefinition),
    Map(MapReprDefinition),
    Link(LinkReprDefinition),
    Struct(StructReprDefinition),
    Enum(EnumReprDefinition),
    Union(UnionReprDefinition),
    Copy(CopyReprDefinition),
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

///
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SchemaKind {
    Null,
    Bool,
    Int, // same as Int128
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Float, // same as Float64
    Float32,
    Float64,
    Bytes,
    String,
    List,
    Map,
    Link,
    Struct,
    Enum,
    Union,
    Copy,
}

// #[derive(Debug)]
// pub struct ContainerAttr {
//     internal: bool,
//     try_from: Option<LitStr>,
// }

/// Wrapper around a vec of fields.
#[derive(Debug)]
pub struct Fields<T>(Vec<T>);
impl<T> Deref for Fields<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type SchemaAttrs = std::collections::HashSet<SchemaAttr>;

#[derive(Debug, PartialEq)]
pub enum SchemaAttr {
    Internal,
    TryFrom(LitStr),
    Wrapper(Type),
}

// #[derive(Debug)]
// pub struct FieldAttr {
//     wrapper: Option<Ident>,
// }

/// Keywords unique to IPLD Schemas and Representations
#[macro_use]
pub(crate) mod kw {
    // Helper macro for parsing a keyword-argument pair from IPLD schemas.
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
        null bool boolean bytes string list map link copy
        int uint8 uint16 uint32 uint64 uint128 int8 int16 int32 int64 int128
        float float32 float64
        // representation types
        listpairs stringjoin stringpairs
        tuple keyed kinded envelope inline byteprefix
        // representation type args
        field nullable optional implicit rename
        join innerDelim entryDelim
        fieldOrder discriminantKey contentKey

        // custom container and field attributes
        internal wrapper ctx_bounds
    }

    // pub struct Directive<K, T>(pub Option<T>, pub std::marker::PhantomData<K>);
}

// Exported Attributes
#[doc(hidden)]
#[macro_export]
macro_rules! def_attributes {
    ($(#[$meta:meta])* $attr:ident) => {
        $(#[$meta])*
        #[proc_macro_attribute]
        pub fn $attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    };
    ($attr:ident) => {
        #[proc_macro_attribute]
        pub fn $attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    };
}

// struct Methods(Vec<ItemFn>);
