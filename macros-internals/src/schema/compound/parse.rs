use super::*;
use crate::dev::{
    common, impl_advanced_parse, parse_kwarg,
    schema::{kw, parse},
};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result as ParseResult},
    LitStr, Path, Token, Type,
};

impl Parse for ListReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // parse list typedef
        let mut nullable = false;
        let typedef_stream;
        bracketed!(typedef_stream in input);
        if typedef_stream.peek(kw::nullable) {
            typedef_stream.parse::<kw::nullable>()?;
            nullable = true;
        }
        let elem = typedef_stream.parse::<Type>()?;
        if !typedef_stream.is_empty() {
            return Err(input.error("invalid IPLD list type definition"));
        }

        if !input.peek(kw::representation) {
            return Ok(Self::Basic { elem, nullable });
        }

        // parse list representation
        input.parse::<kw::representation>()?;
        let name = parse_kwarg!(input, advanced => Path);
        Ok(Self::Advanced(AdvancedListReprDefinition {
            name,
            elem,
            nullable,
            rest: parse::parse_rest(input)?,
        }))
    }
}

impl Parse for MapReprDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // parse map typedef
        let mut nullable = false;
        let typedef_stream;
        braced!(typedef_stream in input);
        let key = typedef_stream.parse::<Type>()?;
        typedef_stream.parse::<Token![:]>()?;
        if typedef_stream.peek(kw::nullable) {
            typedef_stream.parse::<kw::nullable>()?;
            nullable = true;
        }
        let value = typedef_stream.parse::<Type>()?;
        if !typedef_stream.is_empty() {
            return Err(input.error("invalid IPLD map type definition"));
        }

        if !input.peek(kw::representation) {
            return Ok(Self::Basic {
                key,
                value,
                nullable,
            });
        }

        // parse map representation
        input.parse::<kw::representation>()?;
        let map_repr = match input {
            // basic
            _ if input.peek(kw::map) => {
                input.parse::<kw::map>()?;
                Self::Basic {
                    key,
                    value,
                    nullable,
                }
            }
            // listpairs
            _ if input.peek(kw::listpairs) => {
                input.parse::<kw::listpairs>()?;
                Self::Listpairs {
                    key,
                    value,
                    nullable,
                }
            }
            // stringpairs
            _ if input.peek(kw::stringpairs) => {
                input.parse::<kw::stringpairs>()?;
                let (inner_delim, entry_delim) = parse_stringpair_args(input)?;

                Self::Stringpairs {
                    key,
                    value,
                    nullable,
                    inner_delim,
                    entry_delim,
                }
            }
            // advanced
            _ if input.peek(kw::advanced) => {
                let name = parse_kwarg!(input, advanced => Path);
                Self::Advanced(AdvancedMapReprDefinition {
                    name,
                    key,
                    value,
                    nullable,
                    rest: parse::parse_rest(input)?,
                })
            }
            _ => return Err(input.error("invalid IPLD map representation definition")),
        };

        Ok(map_repr)
    }
}

pub(crate) fn parse_stringpair_args(input: ParseStream) -> ParseResult<(LitStr, LitStr)> {
    let args;
    braced!(args in input);

    let mut inner_delim = None;
    let mut entry_delim = None;
    try_parse_stringpair_args(&args, &mut inner_delim, &mut entry_delim)?;
    try_parse_stringpair_args(&args, &mut inner_delim, &mut entry_delim)?;

    let inner_delim = inner_delim.ok_or(
        args.error("invalid IPLD map stringpairs representation definition: missing `innerDelim`"),
    )?;
    let entry_delim = entry_delim.ok_or(
        args.error("invalid IPLD map stringpairs representation definition: missing `entryDelim`"),
    )?;

    Ok((inner_delim, entry_delim))
}

fn try_parse_stringpair_args(
    input: ParseStream,
    inner_delim: &mut Option<LitStr>,
    entry_delim: &mut Option<LitStr>,
) -> ParseResult<()> {
    if input.peek(kw::innerDelim) {
        if inner_delim.is_some() {
            return Err(input.error(
                "invalid IPLD stringpairs representation defintion: duplicate `innerDelim`",
            ));
        }
        inner_delim.replace(parse_kwarg!(input, innerDelim => LitStr));
        Ok(())
    } else if input.peek(kw::entryDelim) {
        if entry_delim.is_some() {
            return Err(input.error(
                "invalid IPLD stringpairs representation defintion: duplicate `entryDelim`",
            ));
        }
        entry_delim.replace(parse_kwarg!(input, entryDelim => LitStr));
        Ok(())
    } else {
        Ok(())
    }
}

impl_advanced_parse!(AdvancedListSchemaDefinition => List, ListReprDefinition);
impl_advanced_parse!(AdvancedMapSchemaDefinition => Map, MapReprDefinition);

// //////////////////////////////////////////////////////////////////////////
// // Link
// //////////////////////////////////////////////////////////////////////////
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! typedef_link {
//     ($name:ident $type:ty) => {
//         struct $name(::libipld_schema::Link<$type>);
//         // type $name = Link<$type>;
//     };
// }

// //////////////////////////////////////////////////////////////////////////
// // List
// //////////////////////////////////////////////////////////////////////////
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! typedef_list {
//     // ($name:ident : $type:ty) => {
//     //     typedef_list!($name $type);
//     // };
//     ($name:tt $value:ty) => {
//         struct $name(::std::vec::Vec<$value>);
//         // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
//         delegate_repr_impl!($name: (::std::vec::Vec<$value>));
//     };
// }

// //////////////////////////////////////////////////////////////////////////
// // Map
// //////////////////////////////////////////////////////////////////////////
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! typedef_map {
//     // basic map representation
//     ($name:ident { $key:ty : $value:ty }) => {
//         struct $name(::std::collections::BTreeMap<$key, $value>);
//         delegate_repr_impl!($name: (::$std::collections::BTreeMap<$key, $value>));
//     };
//     // TODO: map stringpairs representation
//     (@stringpairs $name:ident { $key:ty : $value:ty } $inner:expr, $entry:expr) => {
//         struct $name(::std::collections::BTreeMap<$key, $value>);
//         // repr_map_impl_stringpairs!($name { $key : $value } { $inner, $entry });
//     };
//     // TODO: map listpairs representation
//     (@listpairs $name:ident { $key:ty : $value:ty }) => {
//         struct $name(::std::collections::BTreeMap<$key, $value>);
//         // repr_map_impl_listpairs!($name { $key : $value });
//     };
// }

// // Delegate representation
// // delegates to the inner type's `Representation` implementation
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! delegate_recursive_repr_impl {
//     // delegation impl
//     ($name:tt : $type:tt) => {
//         #[::libipld_schema::prelude::async_trait]
//         impl<R, W> ::libipld_schema::Representation<R, W> for $name
//         where
//             R: ::libipld_schema::prelude::Read + Unpin + Send,
//             W: ::libipld_schema::prelude::Write + Unpin + Send,
//         {
//             #[inline]
//             async fn read<C>(ctx: &mut C) -> Result<Self, ::libipld_schema::Error>
//             where
//                 R: 'async_trait,
//                 W: 'async_trait,
//                 C: ::libipld_schema::Context<R, W> + Send,
//             {
//                 Ok($name(<$type>::read(ctx).await?))
//             }

//             #[inline]
//             async fn write<C>(&self, ctx: &mut C) -> Result<(), ::libipld_schema::Error>
//             where
//                 R: 'async_trait,
//                 W: 'async_trait,
//                 C: ::libipld_schema::Context<R, W> + Send,
//             {
//                 <$type>::write(&self.0, ctx).await
//             }
//         }
//     };
// }
