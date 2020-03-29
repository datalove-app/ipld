use super::{
    kw, Fields, InnerAttributes, OuterAttributes, ReprDefinition, SchemaDefinition, SchemaMeta,
};
use crate::{
    primitive::{
        BoolReprDefinition, BytesReprDefinition, CopyReprDefinition, FloatReprDefinition,
        IntReprDefinition, LinkReprDefinition, NullReprDefinition, StringReprDefinition,
    },
    r#enum::EnumReprDefinition,
    r#struct::StructReprDefinition,
    recursive::{ListReprDefinition, MapReprDefinition},
    union::UnionReprDefinition,
};
use proc_macro2::TokenStream;
use proc_macro_crate::crate_name;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_quote, parse_str, token, Attribute, Ident, Path, Token, Type, Visibility,
};

impl Parse for SchemaDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let SchemaHeader { attrs, ipld_schema } = input.parse()?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![type]>()?;

        let name: Ident = input.parse()?;
        let repr: ReprDefinition = input.parse()?;
        let meta = SchemaMeta {
            attrs,
            ipld_schema,
            vis,
            name,
        };

        Ok(SchemaDefinition { meta, repr })
    }
}

impl Parse for ReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        macro_rules! parse_kw {
            ($input:expr, $kw:path => $variant:ident $repr_def:ident) => {{
                $input.parse::<$kw>()?;
                parse_end($input)?;
                Ok(ReprDefinition::$variant($repr_def))
            }};
            ($input:expr, $kw:path => $variant:ident $repr_def:ident $type:ty) => {{
                $input.parse::<$kw>()?;
                parse_end($input)?;
                Ok(ReprDefinition::$variant($repr_def(parse_quote!($type))))
            }};
        }

        match input {
            // null
            _ if input.peek(kw::null) => parse_kw!(input, kw::null => Null NullReprDefinition),
            // bool
            _ if input.peek(kw::bool) => parse_kw!(input, kw::bool => Bool BoolReprDefinition),
            // ints
            _ if input.peek(kw::int) => parse_kw!(input, kw::int => Int IntReprDefinition i32),
            _ if input.peek(kw::u8) => parse_kw!(input, kw::u8 => Int IntReprDefinition u8),
            _ if input.peek(kw::u16) => parse_kw!(input, kw::u16 => Int IntReprDefinition u16),
            _ if input.peek(kw::u32) => parse_kw!(input, kw::u32 => Int IntReprDefinition u32),
            _ if input.peek(kw::u64) => parse_kw!(input, kw::u64 => Int IntReprDefinition u64),
            _ if input.peek(kw::u128) => parse_kw!(input, kw::u128 => Int IntReprDefinition u128),
            _ if input.peek(kw::i8) => parse_kw!(input, kw::i8 => Int IntReprDefinition i8),
            _ if input.peek(kw::i16) => parse_kw!(input, kw::i16 => Int IntReprDefinition i16),
            _ if input.peek(kw::i32) => parse_kw!(input, kw::i32 => Int IntReprDefinition i32),
            _ if input.peek(kw::i64) => parse_kw!(input, kw::i64 => Int IntReprDefinition i64),
            _ if input.peek(kw::i128) => parse_kw!(input, kw::i128 => Int IntReprDefinition i128),
            // floats
            _ if input.peek(kw::float) => {
                parse_kw!(input, kw::float => Float FloatReprDefinition f64)
            }
            _ if input.peek(kw::f32) => parse_kw!(input, kw::f32 => Float FloatReprDefinition f32),
            _ if input.peek(kw::f64) => parse_kw!(input, kw::f64 => Float FloatReprDefinition f64),
            // string
            _ if input.peek(kw::string) => {
                parse_kw!(input, kw::string => String StringReprDefinition)
            }
            // bytes
            _ if input.peek(kw::bytes) => {
                input.parse::<kw::bytes>()?;
                Ok(ReprDefinition::Bytes(input.parse::<BytesReprDefinition>()?))
            }
            // link
            _ if input.peek(Token![&]) => {
                input.parse::<Token![&]>()?;
                let ty = input.parse::<Type>()?;
                parse_end(input)?;
                Ok(ReprDefinition::Link(LinkReprDefinition(ty)))
            }
            // copy
            _ if input.peek(Token![=]) => {
                input.parse::<Token![=]>()?;
                let ty = input.parse::<Type>()?;
                parse_end(input)?;
                Ok(ReprDefinition::Copy(CopyReprDefinition(ty)))
            }
            // list
            _ if input.peek(token::Bracket) => {
                Ok(ReprDefinition::List(input.parse::<ListReprDefinition>()?))
            }
            // map
            _ if input.peek(token::Brace) => {
                Ok(ReprDefinition::Map(input.parse::<MapReprDefinition>()?))
            }
            // struct
            _ if input.peek(Token![struct]) => {
                input.parse::<Token![struct]>()?;
                Ok(ReprDefinition::Struct(
                    input.parse::<StructReprDefinition>()?,
                ))
            }
            // enum
            _ if input.peek(Token![enum]) => {
                input.parse::<Token![enum]>()?;
                let ty = input.parse::<EnumReprDefinition>()?;
                parse_end(input)?;
                Ok(ReprDefinition::Enum(ty))
            }
            // union
            _ if input.peek(Token![union]) => {
                input.parse::<Token![union]>()?;
                let ty = input.parse::<UnionReprDefinition>()?;
                parse_end(input)?;
                Ok(ReprDefinition::Union(ty))
            }
            _ => Err(input.error("invalid IPLD schema definition")),
        }
    }
}

struct SchemaHeader {
    attrs: OuterAttributes,
    ipld_schema: Path,
}

impl Parse for SchemaHeader {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let internal_flag: Path = parse_quote!(internal_ipld_schema);
        let attrs = input.parse::<OuterAttributes>()?;

        let is_internal: bool = attrs.iter().any(|a| a.path == internal_flag);
        let ipld_schema: Path = if is_internal {
            // macro is used w/in `ipld_schema` => expand to `crate`
            parse_quote!(crate)
        } else {
            // macro is used in foreign crate => expand to dependency
            let name = crate_name("ipld-schema").or(Err(
                input.error("`ipld-schema` is not present in Cargo.toml")
            ))?;
            parse_str(&name)?
        };

        let attrs: OuterAttributes = attrs
            .into_iter()
            .filter(|a| a.path == internal_flag)
            .collect::<Vec<Attribute>>()
            .into();

        Ok(Self { attrs, ipld_schema })
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

// TODO: impl `fn rest(input)`, which grabs the rest of the tokens, but asserts that it ends with a semicolon

///
pub fn is_end(input: ParseStream) -> bool {
    input.peek(Token![;])
}

///
pub fn parse_end(input: ParseStream) -> ParseResult<()> {
    input.parse::<Token![;]>()?;
    if !input.is_empty() {
        Err(input.error("must end IPLD schema definitions with a semicolon"))
    } else {
        Ok(())
    }
}

///
pub fn parse_rest(input: ParseStream) -> ParseResult<TokenStream> {
    let args;
    braced!(args in input);
    parse_end(input)?;
    Ok(args.parse::<TokenStream>()?)
}
