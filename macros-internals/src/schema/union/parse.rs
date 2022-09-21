//! Union

use super::*;
use crate::dev::{
    common, parse_kwarg,
    schema::{kw, SchemaKind},
    OuterAttributes,
};
use std::collections::HashSet;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    Generics, Ident, LitInt, LitStr, Token,
};

impl Parse for UnionReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let field_input;
        braced!(field_input in input);

        input.parse::<kw::representation>()?;
        let union_repr = match input {
            // keyed
            _ if input.peek(kw::keyed) => {
                input.parse::<kw::keyed>()?;
                let fields = field_input.parse::<UnionStrFields>()?;
                Self::Keyed(KeyedUnionReprDefinition { fields })
            }
            // envelope
            _ if input.peek(kw::envelope) => {
                input.parse::<kw::envelope>()?;
                let fields = field_input.parse::<UnionStrFields>()?;
                let args;
                braced!(args in input);

                let mut discriminant_key = None;
                let mut content_key = None;
                try_parse_envelope_args(&args, &mut discriminant_key, &mut content_key)?;
                try_parse_envelope_args(&args, &mut discriminant_key, &mut content_key)?;
                let discriminant_key = discriminant_key.ok_or(args.error("invalid IPLD union envelope representation definition: missing `discriminantKey`"))?;
                let content_key = content_key.ok_or(args.error(
                    "invalid IPLD union envelope representation definition: missing `contentKey`",
                ))?;
                Self::Envelope(EnvelopeUnionReprDefinition {
                    fields,
                    discriminant_key,
                    content_key,
                })
            }
            // inline
            _ if input.peek(kw::inline) => {
                input.parse::<kw::inline>()?;
                let fields = field_input.parse::<UnionStrFields>()?;
                let args;
                braced!(args in input);
                let discriminant_key = parse_kwarg!(args, discriminantKey => LitStr);
                Self::Inline(InlineUnionReprDefinition {
                    fields,
                    discriminant_key,
                })
            }
            // byteprefix
            _ if input.peek(kw::byteprefix) => {
                input.parse::<kw::byteprefix>()?;
                let fields = field_input.parse::<UnionIntFields>()?;
                Self::BytePrefix(BytePrefixUnionReprDefinition { fields })
            }
            // kinded
            _ if input.peek(kw::kinded) => {
                input.parse::<kw::kinded>()?;
                let fields = field_input.parse::<UnionKindedFields>()?;
                let all_unique_kinds = {
                    let set = &fields
                        .iter()
                        .map(|field| &field.key)
                        .collect::<HashSet<&SchemaKind>>();
                    fields.len() == set.len()
                };
                if !all_unique_kinds {
                    return Err(input.error(
                        "invalid IPLD union kinded representation defintion: duplicate kinds",
                    ));
                }
                Self::Kinded(KindedUnionReprDefinition { fields })
            }
            _ => return Err(input.error("invalid IPLD union representation definition")),
        };

        Ok(union_repr)
    }
}

impl<T: Parse> Parse for UnionField<T> {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let attrs = input.parse::<OuterAttributes>()?;
        input.parse::<Token![|]>()?;

        let linked = input.peek(Token![&]);
        if linked {
            input.parse::<Token![&]>()?;
        }
        let value = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>().ok();
        let key = input.parse::<T>()?;

        // parse optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(UnionField {
            wrapper: attrs.parse_wrapper()?,
            attrs: attrs.omit_internal_attrs(),
            value,
            generics,
            key,
            linked,
        })
    }
}

pub(crate) fn try_parse_envelope_args(
    input: ParseStream,
    discriminant_key: &mut Option<LitStr>,
    content_key: &mut Option<LitStr>,
) -> ParseResult<()> {
    if input.peek(kw::discriminantKey) {
        if discriminant_key.is_some() {
            return Err(input.error(
                "invalid IPLD union envelope representation defintion: duplicate `discriminantKey`",
            ));
        }
        *discriminant_key = Some(parse_kwarg!(input, discriminantKey => LitStr));
        Ok(())
    } else if input.peek(kw::contentKey) {
        if content_key.is_some() {
            return Err(input.error(
                "invalid IPLD union envelope representation defintion: duplicate `contentKey`",
            ));
        }
        *content_key = Some(parse_kwarg!(input, contentKey => LitStr));
        Ok(())
    } else {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use quote::quote;
//     use syn::{parse2, LitStr};

//     #[test]
//     fn it_works() {
//         let sample_quote = quote! {
//             | String "string"
//             | u8 "uint8"
//         };

//         let fields = parse2::<UnionFields<LitStr>>(sample_quote).unwrap();
//     }
// }

// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! typedef_union {
//     // TODO: union keyed representation
//     (@keyed $name:ident { $($member:ident $alias:expr,)* }) => {
//         #[derive(Debug, Deserialize, Serialize)]
//         pub enum $name {
//             $(
//                 #[serde(rename = $alias)]
//                 pub(crate) $member($member),
//             )*
//         }
//     };
//     // TODO: union kinded representation
//     (@kinded $name:ident { $($member:ident,)* }) => {
//         unimplemented!("kinded unions")
//     };
//     // TODO: union envelope representation
//     (@envelope $name:ident { $($member:ident $alias:expr,)* } $discriminant:expr, $content:expr) => {
//         // #[derive(Debug, Deserialize, Serialize)]
//         // pub enum $name {
//         //     $(
//         //         #[serde(rename = $alias)]
//         //         $member($member),
//         //     )*
//         // }
//     };
//     // TODO: union inline representation
//     (@inline $name:ident { $($member:ident $alias:expr,)* } $discriminant:expr) => {
//         // #[derive(Debug, Deserialize, Serialize)]
//         // pub enum $name {
//         //     $(
//         //         #[serde(rename = $alias)]
//         //         $member($member),
//         //     )*
//         // }
//     };
//     // TODO: union byteprefix representation
//     (@byteprefix $name:ident { $($member:ident $prefix:expr,)* }) => {
//         unimplemented!("byteprefixed unions")
//     };
// }
