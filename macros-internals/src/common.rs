pub use attr::{InnerAttributes, OuterAttributes};

use syn::{
    parse::{ParseStream, Result as ParseResult},
    Token,
};

pub(crate) mod attr {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};
    use std::ops::Deref;
    use syn::{
        parse::{Parse, ParseStream, Result as ParseResult},
        parse_str, Attribute, LitStr, Path, Type,
    };

    crate::define_keywords! {
        internal
    }

    pub(crate) const IPLD_CRATE_NAME: &'static str = "ipld";
    pub(crate) const ATTR: &'static str = "ipld_attr";

    pub(crate) const INTERNAL: &'static str = "internal";
    pub(crate) const TRY_FROM: &'static str = "try_from";
    pub(crate) const WRAPPER: &'static str = "wrapper";
    pub(crate) const SERDE: &'static str = "serde";

    #[doc(hidden)]
    #[macro_export(local_inner_macros)]
    macro_rules! get_attr {
        // flag
        ($kw:path, $self:ident) => {
            $self
                .iter()
                .filter(|attr| attr.path.is_ident($crate::common::attr::ATTR))
                .any(|attr| attr.parse_args::<$kw>().is_ok())
        };
        // arg
        // () => {};
        // keyword arg
        ($kw:path = $type:ty, $self:ident) => {
            $self
                .iter()
                .filter(|attr| attr.path.is_ident($crate::common::attr::ATTR))
                .filter_map(|attr| attr.parse_args::<syn::MetaNameValue>().ok())
                .filter_map(|attr| match attr {
                    syn::MetaNameValue {
                        path,
                        lit: syn::Lit::Str(lit_str),
                        ..
                    } if path.is_ident($kw) => Some(lit_str),
                    _ => None,
                })
                .try_fold(None, |res, lit_str| {
                    if res.is_none() {
                        Ok(Some(lit_str.parse::<$type>()?))
                    } else {
                        Err(syn::Error::new(
                            lit_str.span(),
                            ::std::format!("duplicate IPLD attribute `{}`", $kw),
                        ))
                    }
                })
        };
    }

    macro_rules! attr_vec {
        ($name:ident, $parse_fn:ident) => {
            /// Wrapper around a vec of `syn::Attribute`s.
            #[derive(Debug, Clone)]
            pub struct $name(Vec<Attribute>);
            impl Deref for $name {
                type Target = Vec<Attribute>;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl Parse for $name {
                fn parse(input: ParseStream) -> ParseResult<Self> {
                    Ok(Self(input.call(Attribute::$parse_fn)?))
                }
            }
            impl $name {
                // TODO: consider getting rid of this omission of additional serde attrs
                pub(crate) fn is_internal_attr(attr: &Attribute) -> bool {
                    attr.path.is_ident(SERDE) || attr.path.is_ident(ATTR)
                }
                pub(crate) fn omit_internal_attrs(self) -> Self {
                    Self(
                        self.0
                            .into_iter()
                            .filter(|attr| !Self::is_internal_attr(attr))
                            .collect::<Vec<Attribute>>(),
                    )
                }
            }
        };
    }

    attr_vec!(OuterAttributes, parse_outer);
    attr_vec!(InnerAttributes, parse_inner);

    impl OuterAttributes {
        pub(crate) fn parse_internal(&self, input: ParseStream) -> bool {
            crate::get_attr!(internal, self)
            // if let Some(path) = crate::get_attr!(CRATE = Path, self)? {
            //     Ok(path)
            // } else {
            //     let name =
            //         .or(Err(input.error("`ipld` is not present in Cargo.toml")))?;
            //     parse_str(&name)
            // }
        }

        pub(crate) fn parse_try_from(&self) -> ParseResult<Option<LitStr>> {
            crate::get_attr!(TRY_FROM = LitStr, self)
            // self.iter().try_fold(None, |mut try_from, attr| {
            //     if attr.path.is_ident(ATTR) {
            //         match attr.parse_meta()? {
            //             Meta::NameValue(MetaNameValue {
            //                 path,
            //                 lit: Lit::Str(lit_str),
            //                 ..
            //             }) if path.is_ident(TRY_FROM) => {
            //                 try_from.replace(lit_str);
            //             }
            //             _ => {}
            //         };
            //     }
            //     Ok(try_from)
            // })
        }

        pub(crate) fn parse_wrapper(&self) -> ParseResult<Option<Type>> {
            crate::get_attr!(WRAPPER = Type, self)
        }
    }

    impl InnerAttributes {
        pub(crate) fn parse_try_from(&self) -> ParseResult<Option<LitStr>> {
            crate::get_attr!(TRY_FROM = LitStr, self)
        }

        pub(crate) fn parse_wrapper(&self) -> ParseResult<Option<Type>> {
            crate::get_attr!(WRAPPER = Type, self)
        }
    }
}

/// Checks if the next token is the ending semicolon.
pub fn is_end(input: ParseStream) -> bool {
    input.peek(Token![;])
}

/// Parses the ending semicolon, asserting that the token stream is empty.
pub fn parse_end(input: ParseStream) -> ParseResult<()> {
    input.parse::<Token![;]>()?;
    if !input.is_empty() {
        Err(input.error("must end IPLD schema definitions with a semicolon"))
    } else {
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! define_keywords {
    ($($kw:ident)*) => {
        $(::syn::custom_keyword!($kw);)*
    };
}
