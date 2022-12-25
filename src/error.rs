use crate::dev::*;
use cid::Error as CidError;
use maybestd::{
    borrow::Cow, error::Error as StdError, fmt::Display, num::TryFromIntError,
    string::FromUtf8Error,
};
#[cfg(feature = "multiaddr")]
use multiaddr::Error as MultiaddrError;
use multibase::Error as MultibaseError;
use multihash::Error as MultihashError;
use serde::{de, ser};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "multiaddr")]
    #[error("Multiaddr error: {0}")]
    Multiaddr(#[from] MultiaddrError),

    #[error("Multibase error: {0}")]
    Multibase(#[from] MultibaseError),

    #[error("Multihash error: {0}")]
    Multihash(#[from] MultihashError),

    #[error("Cid error: {0}")]
    Cid(#[from] CidError),

    #[error("Mismatched `Any` data model type")]
    MismatchedAny,

    #[error("Failed conversion from `Any::{any_variant}` to `{type_name}`")]
    FailedAnyConversion {
        type_name: &'static str,
        any_variant: &'static str,
    },

    //////////////////////////////////////////////////////////////////////
    // codec
    //////////////////////////////////////////////////////////////////////
    ///
    #[error("Unknown multicodec code: {0}")]
    UnknownMulticodecCode(u64),

    #[error("Unknown multicodec name: {0}")]
    UnknownMulticodecName(String),

    #[error("IPLD codec encoding error: {0}")]
    Encoder(Box<dyn StdError>),

    #[error("IPLD codec decoding error: {0}")]
    Decoder(Box<dyn StdError>),

    // #[error("Value error: {0}")]
    // Value(&'static str),
    // #[error("{0}")]
    // BlockMeta(&'static str),
    #[error("Map field parse error: {0}")]
    FieldParse(Box<dyn StdError>),

    //////////////////////////////////////////////////////////////////////
    // selector
    //////////////////////////////////////////////////////////////////////
    #[error("General selection failure: {0}")]
    SelectionFailure(String),

    #[error("Selector Context error: {0}")]
    Context(#[from] anyhow::Error),

    #[error("Invalid selection params: {0}")]
    InvalidSelectionParams(&'static str),

    #[error(
        "Invalid selector: selector `{selector_name}` cannot be used to select against type `{type_name}`"
    )]
    UnsupportedSelector {
        type_name: &'static str,
        // selected_type_name: &'static str,
        selector_name: &'static str,
    },

    #[error("Invalid selector: selector `{0}` does not possess an inner selector")]
    MissingNextSelector(&'static str),

    // #[error(
    //     "Invalid selection: type `{desired_type_name}` cannot be selected from schema type `{actual_type_name}`"
    // )]
    // InvalidTypeSelection {
    //     actual_type_name: &'static str,
    //     desired_type_name: &'static str,
    // },
    #[error("ExploreIndex failure for type `{type_name}`: no node at index {index}")]
    ExploreIndexFailure {
        type_name: &'static str,
        index: usize,
    },

    #[error("ExploreRange failure for type `{type_name}`: missing node at index {index}; range {start}..{end}")]
    ExploreRangeFailure {
        type_name: &'static str,
        index: usize,
        start: Int,
        end: Int,
    },

    #[error("Failed to explore field key `{field_name}` of type `{key_type_name}`")]
    ExploreFieldKeyFailure {
        key_type_name: &'static str,
        field_name: String,
    },

    #[error("Failed to explore field value of type `{value_type_name}` for key `{key}`")]
    ExploreFieldValueFailure {
        value_type_name: &'static str,
        key: String,
    },

    #[error("Selector depth error: {0}: {1}")]
    SelectorDepth(&'static str, usize),

    #[error("Selector range error: {0}")]
    SelectorRange(&'static str),

    #[error("Selector assertion failure")]
    SelectorAssertionFailure,

    //////////////////////////////////////////////////////////////////////
    // misc
    //////////////////////////////////////////////////////////////////////
    #[error("Downcast failure for type `{type_name}`: {msg}")]
    DowncastFailure {
        type_name: &'static str,
        msg: &'static str,
    },

    #[error("{0}")]
    Custom(anyhow::Error),
}

impl Error {
    #[doc(hidden)]
    pub fn failed_any_conversion<T>(any_variant: &'static str) -> Self
    where
        T: Representation,
    {
        Self::FailedAnyConversion {
            type_name: T::NAME,
            any_variant,
        }
    }

    #[doc(hidden)]
    pub fn unsupported_selector<T>(selector: &Selector) -> Self
    where
        T: Representation,
    {
        Self::UnsupportedSelector {
            type_name: <T as Representation>::NAME,
            // selected_type_name: <U as Representation>::NAME,
            selector_name: Representation::name(selector),
        }
    }

    #[doc(hidden)]
    pub fn missing_next_selector(selector: &Selector) -> Self {
        Self::MissingNextSelector(Representation::name(selector))
    }

    #[doc(hidden)]
    pub fn explore_list_failure<E: Representation>(selector: &Selector, index: usize) -> Self {
        match selector {
            Selector::ExploreIndex(_) => Self::ExploreIndexFailure {
                type_name: E::NAME,
                index,
            },
            Selector::ExploreRange(s) => Self::ExploreRangeFailure {
                type_name: E::NAME,
                index,
                start: s.start(),
                end: s.end(),
            },
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn explore_map_failure(selector: &Selector) -> Self {
        match selector {
            // Selector::ExploreIndex(s) => Self::ExploreIndexFailure(current_index),
            // Selector::ExploreRange(s) => Self::ExploreRangeFailure(current_index, s.start, s.end),
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn explore_index_failure<E: Representation>(index: usize) -> Self {
        Self::ExploreIndexFailure {
            type_name: E::NAME,
            index,
        }
    }

    #[doc(hidden)]
    pub fn explore_key_failure<K: Representation>(field_name: Option<&Field<'static>>) -> Self {
        const ANONYMOUS: Field<'static> = Field::Key(Cow::Borrowed("anonymous key"));
        Self::ExploreFieldKeyFailure {
            key_type_name: K::NAME,
            field_name: field_name
                .or(Some(&ANONYMOUS))
                .and_then(|f| f.as_key())
                .unwrap()
                .into(),
        }
    }

    #[doc(hidden)]
    pub fn explore_value_failure<V: Representation>(field: impl Display) -> Self {
        Self::ExploreFieldValueFailure {
            value_type_name: V::NAME,
            key: field.to_string(),
        }
    }

    #[doc(hidden)]
    pub fn downcast_failure<T: Representation>(msg: &'static str) -> Self {
        Self::DowncastFailure {
            type_name: T::NAME,
            msg,
        }
    }

    // pub(crate) fn invalid_type_selection<T, U>() -> Self
    // where
    //     T: Representation,
    //     U: Representation,
    // {
    //     Self::InvalidTypeSelection {
    //         actual_type_name: <T as Representation>::NAME,
    //         desired_type_name: <U as Representation>::NAME,
    //     }
    // }

    ///
    #[inline]
    pub fn decoder<E>(err: E) -> Self
    where
        E: de::Error + 'static,
    {
        Error::Decoder(Box::new(err))
    }

    ///
    #[inline]
    pub fn encoder<E>(err: E) -> Self
    where
        E: ser::Error + 'static,
    {
        Error::Encoder(Box::new(err))
    }

    /*
    #[inline]
    pub fn de_error<E>(self) -> E
    where
        E: de::Error + 'static,
    {
        match self {
            Self::Decoder(inner) if inner.is::<E>() => *inner.downcast::<E>().unwrap(),
            err => E::custom(err),
        }
    }

    #[inline]
    pub fn ser_error<E>(self) -> E
    where
        E: ser::Error + 'static,
    {
        match self {
            Self::Encoder(inner) if inner.is::<E>() => *inner.downcast::<E>().unwrap(),
            Self::Encoder(inner) => E::custom(inner),
            _ => unreachable!(),
        }
    }
     */
}

// impl<E: de::Error> Into<E> for Error {
//     fn into(self) -> E {
//         self.de_error()
//     }
// }

// impl<E> From<E> for Error
// where
//     E: ser::Error,
// {
//     fn from(err: E) -> Error {
//         Error::Decoder(err.to_string())
//     }
// }
