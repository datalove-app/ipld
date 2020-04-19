//! `schema!` macro.

mod display;
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

use super::common::*;
use crate::dev::*;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use std::ops::Deref;
use syn::{parse_str, Attribute, Generics, Ident, ItemFn, LitStr, Path, Visibility};

///
#[derive(Debug, Clone)]
pub struct SchemaMeta {
    pub typedef_str: String,
    pub internal: bool,
    pub try_from: Option<LitStr>,
    pub attrs: OuterAttributes,
    pub vis: Visibility,
    pub name: Ident,
    pub generics: Option<Generics>,
}

impl SchemaMeta {
    pub fn lib(&self) -> TokenStream {
        if self.internal {
            quote!(crate)
        } else {
            let path = crate_name(attr::IPLD_CRATE_NAME)
                .or(Err(()))
                .and_then(|name| parse_str::<Path>(&name).or(Err(())))
                .expect("`ipld` is not present in Cargo.toml");
            quote!(#path)
        }
    }

    pub fn visitor_name(&self) -> Ident {
        Ident::new(
            &format!("__{}Visitor", &self.name.to_string()),
            Span::call_site(),
        )
    }

    pub fn selector_name(&self) -> Ident {
        Ident::new(
            &format!("__{}SelectorSeed", &self.name.to_string()),
            Span::call_site(),
        )
    }
}

// #[derive(Debug, Clone)]
// pub struct SchemaAttrs {
//     lib: Path,
//     try_from: Option<LitStr>,
// }

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

/// Wrapper around a vec of `Field`s.
#[derive(Debug)]
pub struct Fields<T>(Vec<T>);
impl<T> Deref for Fields<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct FieldAttr {
    wrapper: Option<Ident>,
}

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
        null bool int float bytes string list map link copy
        // representation types
        listpairs stringjoin stringpairs
        tuple keyed kinded envelope inline byteprefix
        // representation type args
        field nullable optional implicit rename
        join innerDelim entryDelim
        fieldOrder discriminantKey contentKey

        // custom container and field attributes
        schema internal try_from wrapper
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

impl SchemaMeta {
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

    // fn try_from_name(&self) -> Ident {
    //     let try_from_name = self.try_from.as_ref().unwrap().value();
    //     Ident::new(&try_from_name, Span::call_site())
    // }
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

struct Methods(Vec<ItemFn>);
