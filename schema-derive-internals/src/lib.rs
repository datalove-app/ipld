//!

mod r#enum;
mod expand;
mod parse;
mod primitive;
mod recursive;
mod r#struct;
mod union;

use crate::{
    expand::*,
    primitive::{
        AdvancedBytesReprDefinition, BoolReprDefinition, BytesReprDefinition, CopyReprDefinition,
        FloatReprDefinition, IntReprDefinition, LinkReprDefinition, NullReprDefinition,
        StringReprDefinition,
    },
    r#enum::EnumReprDefinition,
    r#struct::{AdvancedStructReprDefinition, StructReprDefinition},
    recursive::{
        AdvancedListReprDefinition, AdvancedMapReprDefinition, ListReprDefinition,
        MapReprDefinition,
    },
    union::UnionReprDefinition,
};
use std::ops::Deref;
use syn::{parse::Parse, Attribute, Ident, Path, Visibility};

///
#[derive(Debug)]
pub struct SchemaMeta {
    attrs: OuterAttributes,
    ipld_schema: Path,
    vis: Visibility,
    name: Ident,
}

///
#[derive(Debug)]
pub struct SchemaDefinition {
    meta: SchemaMeta,
    repr: ReprDefinition,
}

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
        #[derive(Debug)]
        pub struct $name(Vec<Attribute>);
        impl Deref for $name {
            type Target = Vec<Attribute>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl From<Vec<Attribute>> for $name {
            fn from(vec: Vec<Attribute>) -> Self {
                $name(vec)
            }
        }
        impl IntoIterator for $name {
            type Item = Attribute;
            type IntoIter = std::vec::IntoIter<Self::Item>;
            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
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

#[derive(Debug)]
pub struct Fields<T>(Vec<T>);
impl<T> Deref for Fields<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_use]
pub(crate) mod kw {
    /// Helper macro for parsing a keyword-argument pair from IPLD schemas.
    #[macro_export(local_inner_macros)]
    macro_rules! parse_kwarg {
        ($input:ident, $kw:ident => $type:ty) => {{
            $input.parse::<crate::kw::$kw>()?;
            $input.parse::<$type>()?
        }};
    }

    #[doc(hidden)]
    macro_rules! define_keywords {
        ($($kw:ident)*) => {
            $(::syn::custom_keyword!($kw);)*
        };
    }

    define_keywords! {
        // main
        representation advanced
        // representation data types
        null bool int u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 float f32 f64
        bytes string list map link copy
        // representation types
        listpairs stringjoin stringpairs
        tuple keyed kinded envelope inline byteprefix
        // representation type args
        field nullable optional implicit rename
        join innerDelim entryDelim
        fieldOrder discriminantKey contentKey
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! def_internal_flag {
    () => {
        #[doc(hidden)]
        #[proc_macro_attribute]
        pub fn internal_ipld_schema(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    };
}
