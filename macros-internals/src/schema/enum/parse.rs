//! Enum

use super::{EnumIntField, EnumIntFields, EnumReprDefinition, EnumStrField, EnumStrFields};
use crate::dev::{common, parse, schema::kw, Fields, InnerAttributes};
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
    token, Ident, LitInt, LitStr, Token, Type,
};

impl Parse for EnumReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let typedef_stream;
        braced!(typedef_stream in input);

        if !input.peek(kw::representation) {
            let fields = typedef_stream.parse::<EnumStrFields>()?;
            return Ok(Self::String { fields });
        }

        input.parse::<kw::representation>()?;
        let enum_repr = match input {
            // string
            _ if input.peek(kw::string) => {
                input.parse::<kw::string>()?;
                let fields = typedef_stream.parse::<EnumStrFields>()?;
                EnumReprDefinition::String { fields }
            }
            // int
            _ if input.peek(kw::int) => {
                input.parse::<kw::int>()?;
                let fields = typedef_stream.parse::<EnumIntFields>()?;
                EnumReprDefinition::Int { fields }
            }
            _ => return Err(input.error("invalid IPLD enum representation definition")),
        };

        Ok(enum_repr)
    }
}

impl Parse for EnumStrField {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let attrs = input.parse::<InnerAttributes>()?;
        input.parse::<Token![|]>()?;

        let name = input.parse::<Ident>()?;
        let alias = if input.peek(token::Paren) {
            let alias_stream;
            parenthesized!(alias_stream in input);
            Some(alias_stream.parse::<LitStr>()?)
        } else {
            None
        };

        // parse optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(EnumStrField { attrs, name, alias })
    }
}

impl Parse for EnumIntFields {
    // TODO: determine int type for enum `repr(int_ty)`
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let repr_type = Type::Verbatim(quote!(i32));
        let fields = input.parse::<Fields<EnumIntField>>()?;
        Ok(Self { repr_type, fields })
    }
}

impl Parse for EnumIntField {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let attrs = input.parse::<InnerAttributes>()?;
        input.parse::<Token![|]>()?;

        let name = input.parse::<Ident>()?;
        let alias_stream;
        parenthesized!(alias_stream in input);
        let alias = alias_stream.parse::<LitStr>()?.parse::<LitInt>()?;

        // parse optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(EnumIntField { attrs, name, alias })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse2, LitStr};

    #[test]
    fn it_works() {
        let sample_quote = quote! {
            | Yes
            | No
        };

        let fields = parse2::<EnumStrFields>(sample_quote).unwrap();
        println!("enum fields: {:?}", fields);

        let fields = quote!(#fields);
        println!("quoted enum fields: {:?}", fields);
    }
}
