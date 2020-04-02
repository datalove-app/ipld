use super::*;
use proc_macro2::TokenStream;
use proc_macro_crate::crate_name;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_quote, parse_str, token, Ident, ItemFn, Lit, LitStr, Meta, MetaNameValue, Path, Token,
    Type, Visibility,
};

impl Parse for SchemaMeta {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let typedef_str = input.to_string();

        // parse attributes and any flags
        let attrs = input.parse::<OuterAttributes>()?;
        let ipld_schema_lib: Path = attrs.parse_lib(input)?;
        let try_from = attrs.parse_try_from()?;

        // type info
        let vis = input.parse::<Visibility>()?;
        input.parse::<Token![type]>()?;
        let name = input.parse::<Ident>()?;

        Ok(Self {
            typedef_str,
            ipld_schema_lib,
            try_from,
            attrs: attrs.omit_internal_attrs(),
            vis,
            name,
        })
    }
}

impl Parse for SchemaDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let meta = input.parse::<SchemaMeta>()?;
        let repr = input.parse::<ReprDefinition>()?;

        if meta.try_from.is_some() && !repr.supports_try_from() {
            Err(input.error(format!("`{}` attribute only supported for Int, Float, String, and basic Bytes representations", TRY_FROM)))
        } else {
            Ok(SchemaDefinition { meta, repr })
        }
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
            // floats
            _ if input.peek(kw::float) => {
                parse_kw!(input, kw::float => Float FloatReprDefinition f64)
            }
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

impl OuterAttributes {
    fn parse_lib(&self, input: ParseStream) -> ParseResult<Path> {
        if self.iter().any(|a| a.path.is_ident(INTERNAL)) {
            // macro is used w/in `ipld_schema` => expand to `crate`
            Ok(parse_quote!(crate))
        } else {
            // macro is used in foreign crate => expand to dependency
            let name = crate_name(CRATE_NAME).or(Err(
                input.error("`ipld-schema` is not present in Cargo.toml")
            ))?;
            parse_str(&name)
        }
    }

    fn parse_try_from(&self) -> ParseResult<Option<LitStr>> {
        let mut try_from = None;

        for attr in self.iter() {
            if attr.path.is_ident(TRY_FROM) {
                if let Meta::NameValue(MetaNameValue {
                    path: _,
                    eq_token: _,
                    lit: Lit::Str(lit_str),
                }) = attr.parse_meta()?
                {
                    try_from.replace(lit_str);
                };
            }
        }

        Ok(try_from)
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

/// Checks if the next token is the ending semicolon.
pub fn is_end(input: ParseStream) -> bool {
    input.peek(Token![;])
}

/// Parses the ending semicolon, asserting that the
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

impl Parse for super::Methods {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut vec = Vec::new();
        while !input.is_empty() {
            vec.push(input.parse::<ItemFn>()?);
        }
        Ok(Self(vec))
    }
}
