//! Struct

use super::*;
use crate::dev::{
    common, impl_advanced_parse, parse_kwarg,
    schema::{kw, parse},
    InnerAttributes, OuterAttributes, *,
};
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
    token, Expr, Generics, Ident, LitStr, Path, Visibility,
};

impl Parse for StructReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let typedef_stream;
        braced!(typedef_stream in input);
        let fields = typedef_stream.parse::<StructFields>()?;

        // map
        if !input.peek(kw::representation) {
            return Ok(Self::Map(BasicStructReprDefinition { fields }));
        }

        // defined representation
        input.parse::<kw::representation>()?;
        let struct_repr = match input {
            // map
            _ if input.peek(kw::map) => {
                input.parse::<kw::map>()?;
                Self::Map(BasicStructReprDefinition { fields })
            }
            // listpairs
            _ if input.peek(kw::listpairs) => {
                input.parse::<kw::listpairs>()?;
                Self::Listpairs(ListpairsStructReprDefinition { fields })
            }
            // tuple
            // TODO? assert that any fields do not have optional + implicit?
            _ if input.peek(kw::tuple) => {
                input.parse::<kw::tuple>()?;

                if input.is_empty() | input.peek(Token![;]) {
                    Self::Tuple(TupleStructReprDefinition {
                        fields,
                        field_order: None,
                    })
                } else {
                    let args;
                    braced!(args in input);
                    let field_order = parse_kwarg!(args, fieldOrder => Expr);

                    Self::Tuple(TupleStructReprDefinition {
                        fields,
                        field_order: Some(field_order),
                    })
                }
            }
            // stringpairs
            _ if input.peek(kw::stringpairs) => {
                input.parse::<kw::stringpairs>()?;

                let (inner_delim, entry_delim) = parse_stringpair_args(input)?;
                Self::Stringpairs(StringpairsStructReprDefinition {
                    fields,
                    inner_delim,
                    entry_delim,
                })
            }
            // stringjoin
            _ if input.peek(kw::stringjoin) => {
                input.parse::<kw::stringjoin>()?;

                let args;
                braced!(args in input);
                let join = parse_kwarg!(args, join => LitStr);

                Self::Stringjoin(StringjoinStructReprDefinition { fields, join })
            }
            // advanced
            _ if input.peek(kw::advanced) => {
                let name = parse_kwarg!(input, advanced => Path);
                Self::Advanced(AdvancedStructReprDefinition {
                    name,
                    fields,
                    rest: parse::parse_rest(input)?,
                })
            }
            _ => return Err(input.error("invalid IPLD struct representation definition")),
        };

        Ok(struct_repr)
    }
}

impl Parse for StructField {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut optional = false;
        let mut nullable = false;
        let mut implicit = None;
        let mut rename = None;

        let attrs = input.parse::<OuterAttributes>()?;
        let vis = input.parse::<Visibility>()?;
        let key = input.parse::<Ident>()?;

        try_parse_schema_directives(input, &mut optional, &mut nullable)?;
        try_parse_schema_directives(input, &mut optional, &mut nullable)?;

        let value = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>().map_or(None, Some);

        if input.peek(token::Paren) {
            let repr_input;
            parenthesized!(repr_input in input);
            try_parse_repr_directives(&repr_input, &mut implicit, &mut rename)?;
            try_parse_repr_directives(&repr_input, &mut implicit, &mut rename)?;
        }

        if optional && implicit.is_some() {
            return Err(input.error("invalid IPLD struct representation definition: field cannot have `optional` and `implicit` directives"));
        }

        // parse optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(StructField {
            attrs,
            vis,
            key,
            value,
            generics,
            nullable,
            optional,
            implicit,
            rename,
        })
    }
}

fn try_parse_schema_directives(
    input: ParseStream,
    optional: &mut bool,
    nullable: &mut bool,
) -> ParseResult<()> {
    if input.peek(kw::optional) {
        if *optional {
            return Err(input.error("invalid IPLD struct field type definition"));
        }
        input.parse::<kw::optional>()?;
        *optional = true;
        Ok(())
    } else if input.peek(kw::nullable) {
        if *nullable {
            return Err(input.error("invalid IPLD struct field type definition"));
        }
        input.parse::<kw::nullable>()?;
        *nullable = true;
        Ok(())
    } else {
        Ok(())
    }
}

fn try_parse_repr_directives(
    input: ParseStream,
    implicit: &mut Option<LitStr>,
    rename: &mut Option<LitStr>,
) -> ParseResult<()> {
    if input.peek(kw::implicit) {
        if implicit.is_some() {
            return Err(input.error("invalid IPLD struct field type definition"));
        }
        let val = parse_kwarg!(input, implicit => LitStr);
        implicit.replace(val);
        Ok(())
    } else if input.peek(kw::rename) {
        if rename.is_some() {
            return Err(input.error("invalid IPLD struct field type definition"));
        }
        let val = parse_kwarg!(input, rename => LitStr);
        rename.replace(val);
        Ok(())
    } else {
        Ok(())
    }
}

impl_advanced_parse!(AdvancedStructSchemaDefinition => Struct, StructReprDefinition);
