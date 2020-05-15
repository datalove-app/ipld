use super::{common::*, kw, *};
use crate::dev::*;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Peek, Result as ParseResult},
    token, Ident, Lit, LitInt, LitStr, Path, Token, Type,
};

impl Parse for RootSelectorDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // any attributes
        let attrs = input.parse::<OuterAttributes>()?;
        let internal = attrs.parse_internal(input);

        // the root type this selector begins selecting against
        // TODO: maybe parse optional bracketed type: `<Type>` or attr
        let root_type = input.parse::<Type>()?;
        input.parse::<Token![,]>()?;

        // the actual selector definition
        let def = input.parse::<SelectorDefinition>()?;

        // parse optional ending semicolon
        if common::is_end(input) {
            common::parse_end(input)?;
        }

        Ok(Self {
            internal,
            root_type,
            def,
        })
    }
}

impl Parse for SelectorDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        match input {
            // Matcher
            _ if input.peek(Token![match]) => {
                input.parse::<Token![match]>()?;

                if input.peek(token::Paren) {
                    // parse the optional label
                    let inner;
                    parenthesized!(inner in input);
                    inner.parse::<kw::label>()?;
                    inner.parse::<Token![=]>()?;

                    let inner_args;
                    parenthesized!(inner_args in inner);
                    let label = inner_args.parse::<Interpolated<LitStr>>()?;
                    Ok(Self::Matcher(Some(label)))
                } else {
                    // unlabeled
                    Ok(Self::Matcher(None))
                }
            }
            // ExploreAll
            _ if input.peek(kw::all) => {
                input.parse::<kw::all>()?;
                let inner;
                parenthesized!(inner in input);
                Ok(Self::ExploreAll(input.parse()?))
            }
            // ExploreFields
            _ if input.peek(kw::fields) => {
                input.parse::<kw::fields>()?;
                let inner;
                parenthesized!(inner in input);

                let mut vec = Vec::new();
                while !inner.is_empty() {
                    vec.push((input.parse()?, input.parse()?));
                }
                Ok(Self::ExploreFields(vec))
            }
            // TODO ExploreIndex
            // TODO ExploreRange
            // ExploreRecursive
            _ if input.peek(kw::recursive) => {
                input.parse::<kw::recursive>()?;
                let inner;
                parenthesized!(inner in input);

                // parse the limit
                inner.parse::<kw::limit>()?;
                inner.parse::<Token![=]>()?;
                let limit = match inner.parse::<Interpolated<Lit>>()? {
                    Interpolated::Ident(i) => Some(Interpolated::Ident(i)),
                    Interpolated::Expected(Lit::Int(int)) => Some(Interpolated::Expected(int)),
                    Interpolated::Expected(Lit::Str(s)) if s.value() == "none" => None,
                    _ => {
                        return Err(input
                            .error("ExploreRecursive selector missing required `limit` parameter"))
                    }
                };

                // parse sequence selector
                let sequence = inner.parse()?;

                // parse stopAt param
                if input.peek(kw::stopAt) {
                    input.parse::<kw::stopAt>()?;
                    // TODO
                }

                Ok(Self::ExploreRecursive { limit, sequence })
            }
            // ExploreUnion
            _ if input.peek(Token![union]) => {
                input.parse::<Token![union]>()?;
                let inner;
                parenthesized!(inner in input);

                let mut vec = Vec::new();
                while !inner.is_empty() {
                    vec.push(inner.parse()?);
                }
                Ok(Self::ExploreUnion(vec))
            }
            // TODO ExploreConditional
            // ExploreRecursiveEdge
            _ if input.peek(kw::recurse) => Ok(Self::ExploreRecursiveEdge),
            _ => Err(input.error("failed to parse selector")),
        }
    }
}

impl<T> Parse for Interpolated<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            Ok(Self::Ident(input.parse::<Ident>()?))
        } else {
            Ok(Self::Expected(input.parse::<T>()?))
        }
    }
}

// impl Parse for InterpolatedSelector {
//     fn parse(input: ParseStream) -> ParseResult<Self> {
//         if input.peek(Token![#]) {
//             input.parse::<Token![#]>()?;
//             Ok(Self::Ident(input.parse::<Ident>()?))
//         } else {
//             Ok(Self::Expected(input.parse()?))
//         }
//     }
// }
