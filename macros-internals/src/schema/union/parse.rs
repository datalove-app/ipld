//! Union

use super::{
    DataModelKind, UnionField, UnionIntFields, UnionKindedFields, UnionReprDefinition,
    UnionStrFields,
};
use crate::dev::{parse_kwarg, schema::kw, InnerAttributes};
use std::collections::HashSet;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    LitInt, LitStr, Token, Type,
};

impl Parse for UnionReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let typedef_stream;
        braced!(typedef_stream in input);

        input.parse::<kw::representation>()?;
        let union_repr = match input {
            // keyed
            _ if input.peek(kw::keyed) => {
                let fields = typedef_stream.parse::<UnionStrFields>()?;
                UnionReprDefinition::Keyed { fields }
            }
            // envelope
            _ if input.peek(kw::envelope) => {
                let fields = typedef_stream.parse::<UnionStrFields>()?;
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

                UnionReprDefinition::Envelope {
                    fields,
                    discriminant_key,
                    content_key,
                }
            }
            // inline
            _ if input.peek(kw::inline) => {
                let fields = typedef_stream.parse::<UnionStrFields>()?;
                let args;
                braced!(args in input);
                let discriminant_key = parse_kwarg!(args, discriminantKey => LitStr);

                UnionReprDefinition::Inline {
                    fields,
                    discriminant_key,
                }
            }
            // byteprefix
            _ if input.peek(kw::byteprefix) => {
                let fields = typedef_stream.parse::<UnionIntFields>()?;
                UnionReprDefinition::BytePrefix { fields }
            }
            // kinded
            _ if input.peek(kw::kinded) => {
                let fields = typedef_stream.parse::<UnionKindedFields>()?;
                let unique_kinds = {
                    let set = &fields
                        .iter()
                        .map(|field| &field.key)
                        .collect::<HashSet<&DataModelKind>>();
                    fields.len() == set.len()
                };
                if !unique_kinds {
                    return Err(input.error(
                        "invalid IPLD union kinded representation defintion: duplicate kinds",
                    ));
                }

                UnionReprDefinition::Kinded { fields }
            }
            _ => return Err(input.error("invalid IPLD union representation definition")),
        };

        Ok(union_repr)
    }
}

impl<T: Parse> Parse for UnionField<T> {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let attrs = input.parse::<InnerAttributes>()?;
        input.parse::<Token![|]>()?;

        let mut linked = false;
        if input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            linked = true;
        }
        let value = input.parse::<Type>()?;
        let key = input.parse::<T>()?;

        // parse optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(UnionField {
            attrs,
            value,
            key,
            linked,
        })
    }
}

impl Parse for DataModelKind {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        macro_rules! parse_kw {
            ($input:expr, $kw:path => $variant:ident) => {{
                $input.parse::<$kw>()?;
                Ok(DataModelKind::$variant)
            }};
        }

        match input {
            _ if input.peek(kw::null) => parse_kw!(input, kw::null => Null),
            _ if input.peek(kw::bool) => parse_kw!(input, kw::bool => Boolean),
            _ if input.peek(kw::int) => parse_kw!(input, kw::int => Integer),
            _ if input.peek(kw::float) => parse_kw!(input, kw::float => Float),
            _ if input.peek(kw::bytes) => parse_kw!(input, kw::bytes => Bytes),
            _ if input.peek(kw::string) => parse_kw!(input, kw::string => String),
            _ if input.peek(kw::list) => parse_kw!(input, kw::list => List),
            _ if input.peek(kw::map) => parse_kw!(input, kw::map => Map),
            _ if input.peek(kw::link) => parse_kw!(input, kw::link => Link),
            _ => Err(input.error(
                "invalid IPLD union kinded representation definition: invalid data model kind",
            )),
        }
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
