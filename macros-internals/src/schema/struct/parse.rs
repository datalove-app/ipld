//! Struct

use super::{AdvancedStructReprDefinition, StructField, StructFields, StructReprDefinition};
use crate::dev::{
    parse_kwarg,
    schema::{kw, parse, recursive::parse::parse_stringpair_args},
    InnerAttributes,
};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
    token, Expr, Ident, LitStr, Path, Type,
};

impl Parse for StructReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let typedef_stream;
        braced!(typedef_stream in input);
        let fields: StructFields = typedef_stream.parse_terminated(StructField::parse)?;

        let struct_repr = if parse::is_end(input) {
            StructReprDefinition::Map { fields }
        } else {
            input.parse::<kw::representation>()?;
            match input {
                // map
                _ if input.peek(kw::map) => {
                    input.parse::<kw::map>()?;
                    StructReprDefinition::Map { fields }
                }
                // listpairs
                _ if input.peek(kw::listpairs) => {
                    input.parse::<kw::listpairs>()?;
                    StructReprDefinition::Listpairs { fields }
                }
                // tuple
                // TODO? assert that any fields do not have optional + implicit?
                _ if input.peek(kw::tuple) => {
                    input.parse::<kw::tuple>()?;
                    if input.is_empty() {
                        StructReprDefinition::Tuple {
                            fields,
                            field_order: None,
                        }
                    } else {
                        let args;
                        braced!(args in input);
                        let field_order = parse_kwarg!(args, fieldOrder => Expr);

                        StructReprDefinition::Tuple {
                            fields,
                            field_order: Some(field_order),
                        }
                    }
                }
                // stringpairs
                _ if input.peek(kw::stringpairs) => {
                    input.parse::<kw::stringpairs>()?;
                    let (inner_delim, entry_delim) = parse_stringpair_args(input)?;

                    StructReprDefinition::Stringpairs {
                        fields,
                        inner_delim,
                        entry_delim,
                    }
                }
                // stringjoin
                _ if input.peek(kw::stringjoin) => {
                    input.parse::<kw::stringjoin>()?;
                    let args;
                    braced!(args in input);
                    let join = parse_kwarg!(args, join => LitStr);

                    StructReprDefinition::Stringjoin { fields, join }
                }
                // advanced
                _ if input.peek(kw::advanced) => {
                    let name = parse_kwarg!(input, advanced => Path);
                    StructReprDefinition::Advanced(AdvancedStructReprDefinition {
                        name,
                        fields,
                        rest: parse::parse_rest(input)?,
                    })
                }
                _ => return Err(input.error("invalid IPLD struct representation definition")),
            }
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

        let attrs = input.parse::<InnerAttributes>()?;
        let key = input.parse::<Ident>()?;
        let value = input.parse::<Type>()?;

        try_parse_schema_directives(input, &mut optional, &mut nullable)?;
        try_parse_schema_directives(input, &mut optional, &mut nullable)?;
        if input.peek(token::Paren) {
            let repr_input;
            parenthesized!(repr_input in input);
            try_parse_repr_directives(&repr_input, &mut implicit, &mut rename)?;
            try_parse_repr_directives(&repr_input, &mut implicit, &mut rename)?;
        }

        Ok(StructField {
            attrs,
            key,
            value,
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
        implicit.replace(parse_kwarg!(input, implicit => LitStr));
        Ok(())
    } else if input.peek(kw::rename) {
        if rename.is_some() {
            return Err(input.error("invalid IPLD struct field type definition"));
        }
        rename.replace(parse_kwarg!(input, rename => LitStr));
        Ok(())
    } else {
        Ok(())
    }
}
