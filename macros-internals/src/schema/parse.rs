use super::*;
use crate::dev::*;
use proc_macro2::TokenStream;
use std::fmt;
use syn::{
    braced,
    parse::{Error as ParseError, Parse, ParseStream, Peek, Result as ParseResult},
    parse_quote, parse_str, token, Generics, Ident, ItemFn, Lit, LitStr, Meta, MetaNameValue, Path,
    Token, Type, Visibility,
};

impl Parse for SchemaMeta {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        /* TODO: typedef str
         * get span as multiline str
         * trim leading whitespace from first line, then trim that amt from each line
         * trim all doc comments
         * ?? add some pragma/docstring, specific to the ipld + ipld-macros version?
         * then output the raw-codec CID of the of the typedef str
         */

        // parse attributes and any flags
        let attrs = input.parse::<OuterAttributes>()?;
        let internal = attrs.parse_internal(input);
        let try_from = attrs.parse_try_from()?;

        // type info
        let vis = input.parse::<Visibility>()?;
        input.parse::<Token![type]>()?;
        let name = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>().map_or(None, Some);

        Ok(Self {
            // TODO: fix this
            typedef_str: String::default(),
            internal,
            try_from,
            attrs: attrs.omit_internal_attrs(),
            vis,
            name,
            generics,
        })
    }
}

impl Parse for SchemaDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        //
        let meta = input.parse::<SchemaMeta>()?;
        let repr = input.parse::<ReprDefinition>()?;

        if meta.try_from.is_some() && !repr.supports_try_from() {
            Err(input.error(format!("`{}` attribute only supported for Int, Float, String, and basic Bytes representations", attr::TRY_FROM)))
        } else {
            let mut schema_def = SchemaDefinition { meta, repr };
            // TODO: complete this
            // schema_def.meta.typedef_str = format!("{}", &schema_def);

            // parse ending semicolon
            parse_end(input)?;

            Ok(schema_def)
        }
    }
}

impl Parse for ReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        macro_rules! parse_kw {
            ($input:expr, $kw:path => $variant:ident $repr_def:ident) => {{
                $input.parse::<$kw>()?;
                Self::$variant($repr_def)
            }};
            ($input:expr, $kw:path => $variant:ident $repr_def:ident $type:ty) => {{
                $input.parse::<$kw>()?;
                Self::$variant($repr_def(parse_quote!($type)))
            }};
        }

        let repr_def = match input {
            // null
            _ if input.peek(kw::null) => parse_kw!(input, kw::null => Null NullReprDefinition),
            // bool
            _ if input.peek(kw::bool) => parse_kw!(input, kw::bool => Bool BoolReprDefinition),
            // ints
            _ if input.peek(kw::int) => parse_kw!(input, kw::int => Int IntReprDefinition i32),
            _ if input.peek(kw::uint8) => parse_kw!(input, kw::uint8 => Int IntReprDefinition u8),
            _ if input.peek(kw::uint16) => {
                parse_kw!(input, kw::uint16 => Int IntReprDefinition u16)
            }
            _ if input.peek(kw::uint32) => {
                parse_kw!(input, kw::uint32 => Int IntReprDefinition u32)
            }
            _ if input.peek(kw::uint64) => {
                parse_kw!(input, kw::uint64 => Int IntReprDefinition u64)
            }
            _ if input.peek(kw::uint128) => {
                parse_kw!(input, kw::uint128 => Int IntReprDefinition u128)
            }
            _ if input.peek(kw::int8) => parse_kw!(input, kw::int8 => Int IntReprDefinition i8),
            _ if input.peek(kw::int16) => parse_kw!(input, kw::int16 => Int IntReprDefinition i16),
            _ if input.peek(kw::int32) => parse_kw!(input, kw::int32 => Int IntReprDefinition i32),
            _ if input.peek(kw::int64) => parse_kw!(input, kw::int64 => Int IntReprDefinition i64),
            _ if input.peek(kw::int128) => {
                parse_kw!(input, kw::int128 => Int IntReprDefinition i128)
            }
            // floats
            _ if input.peek(kw::float) => {
                parse_kw!(input, kw::float => Float FloatReprDefinition f64)
            }
            _ if input.peek(kw::float32) => {
                parse_kw!(input, kw::float32 => Float FloatReprDefinition f32)
            }
            _ if input.peek(kw::float64) => {
                parse_kw!(input, kw::float64 => Float FloatReprDefinition f64)
            }
            // string
            _ if input.peek(kw::string) => {
                parse_kw!(input, kw::string => String StringReprDefinition)
            }
            // bytes
            _ if input.peek(kw::bytes) => {
                input.parse::<kw::bytes>()?;
                Self::Bytes(input.parse::<BytesReprDefinition>()?)
            }
            // link
            _ if input.peek(Token![&]) => {
                input.parse::<Token![&]>()?;
                Self::Link(LinkReprDefinition(input.parse::<Type>()?))
            }
            // copy
            _ if input.peek(Token![=]) => {
                input.parse::<Token![=]>()?;
                Self::Copy(CopyReprDefinition(input.parse::<Type>()?))
            }
            // list
            _ if input.peek(token::Bracket) => Self::List(input.parse()?),
            // map
            _ if input.peek(token::Brace) => Self::Map(input.parse()?),
            // struct
            _ if input.peek(Token![struct]) => {
                input.parse::<Token![struct]>()?;
                Self::Struct(input.parse::<StructReprDefinition>()?)
            }
            // enum
            _ if input.peek(Token![enum]) => {
                input.parse::<Token![enum]>()?;
                Self::Enum(input.parse::<EnumReprDefinition>()?)
            }
            // union
            _ if input.peek(Token![union]) => {
                input.parse::<Token![union]>()?;
                Self::Union(input.parse::<UnionReprDefinition>()?)
            }
            _ => {
                return Err(ParseError::new(
                    input.span(),
                    "invalid IPLD schema definition",
                ))
            }
        };

        Ok(repr_def)
    }
}

impl<T: Parse> Parse for Fields<T> {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut vec = Vec::new();
        while !input.is_empty() {
            vec.push(input.parse::<T>()?);
        }
        Ok(Self(vec))
    }
}

// impl<K: Parse + Peek + Default, T: Parse> Parse for kw::Directive<K, T> {
//     fn parse(input: ParseStream) -> ParseResult<Self> {
//         use std::marker::PhantomData;

//         if input.peek::<K>(K::default()) {
//             input.parse::<K>()?;
//             Ok(Self(Some(input.parse::<T>()?), PhantomData::<K>))
//         } else {
//             Ok(Self(None, PhantomData::<K>))
//         }
//     }
// }

///
/// TODO: impl `fn rest(input)`, which grabs the rest of the tokens, but asserts that it ends with a semicolon
///
pub(crate) fn parse_rest(input: ParseStream) -> ParseResult<TokenStream> {
    let args;
    braced!(args in input);
    Ok(args.parse::<TokenStream>()?)
}

// impl Parse for super::Methods {
//     fn parse(input: ParseStream) -> ParseResult<Self> {
//         let mut vec = Vec::new();
//         while !input.is_empty() {
//             vec.push(input.parse::<ItemFn>()?);
//         }
//         Ok(Self(vec))
//     }
// }

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! impl_advanced_parse {
    ($def:ident => $type:ident, $repr_variant:ident) => {
        impl syn::parse::Parse for $def {
            fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
                use crate::schema::{ReprDefinition, SchemaDefinition};
                let SchemaDefinition { meta, repr } = input.parse()?;
                match repr {
                    ReprDefinition::$type($repr_variant::Advanced(repr)) => Ok(Self { meta, repr }),
                    _ => Err(input.error(&::std::format!(
                        "invalid IPLD {} advanced representation",
                        ::std::stringify!($type)
                    ))),
                }
            }
        }
    };
}
