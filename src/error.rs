use crate::dev::*;
use cid::Error as CidError;
use multihash::Error as MultihashError;
use serde::{de, ser};
use std::{
    convert::Infallible, error::Error as StdError, num::TryFromIntError, string::FromUtf8Error,
    sync::mpsc::SendError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("CID error: {0}")]
    Cid(#[from] CidError),

    #[error("Multihash error: {0}")]
    Multihash(#[from] MultihashError),

    #[error("Value error: {0}")]
    Value(&'static str),

    #[error("{0}")]
    BlockMeta(&'static str),

    #[error("IPLD format encoding error: {0}")]
    Encoder(Box<dyn StdError + Send + Sync + 'static>),

    #[error("IPLD format decoding error: {0}")]
    Decoder(Box<dyn StdError + Send + Sync + 'static>),

    #[error("Selector Context error: {0}")]
    Context(#[from] anyhow::Error),

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

    #[error(
        "Invalid selection: type `{desired_type_name}` cannot be selected from schema type `{actual_type_name}`"
    )]
    InvalidTypeSelection {
        actual_type_name: &'static str,
        desired_type_name: &'static str,
    },

    #[error("ExploreIndex failure: no node at index {0}")]
    ExploreIndexFailure(usize),

    #[error("ExploreRange failure: missing node at index {0}; range {1}..{2}")]
    ExploreRangeFailure(usize, Int, Int),

    #[error("Selector depth error: {0}: {1}")]
    SelectorDepth(&'static str, usize),

    #[error("Selector range error: {0}")]
    SelectorRange(&'static str),

    #[error("Unknown multicodec: {0}")]
    UnknownMulticodec(u64),

    #[error("{0}")]
    Custom(anyhow::Error),
}

impl Error {
    pub(crate) fn unsupported_selector<T>(selector: &Selector) -> Self
    where
        T: Representation,
    {
        Self::UnsupportedSelector {
            type_name: <T as Representation>::NAME,
            // selected_type_name: <U as Representation>::NAME,
            selector_name: selector.name(),
        }
    }

    pub(crate) fn missing_next_selector(selector: &Selector) -> Self {
        Self::MissingNextSelector(selector.name())
    }

    pub(crate) fn explore_list_failure(selector: &Selector, current_index: usize) -> Self {
        match selector {
            Selector::ExploreIndex(s) => Self::ExploreIndexFailure(current_index),
            Selector::ExploreRange(s) => Self::ExploreRangeFailure(current_index, s.start, s.end),
            _ => unreachable!(),
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
        E: de::Error + Send + Sync + 'static,
    {
        Error::Decoder(Box::new(err))
    }

    ///
    #[inline]
    pub fn encoder<E>(err: E) -> Self
    where
        E: ser::Error + Send + Sync + 'static,
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
