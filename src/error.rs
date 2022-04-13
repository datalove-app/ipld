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
    Encoder(Box<dyn StdError>),

    #[error("IPLD format decoding error: {0}")]
    Decoder(Box<dyn StdError>),

    #[error("Selector Context error: {0}")]
    Context(#[from] anyhow::Error),

    #[error(
        "Invalid selector: selector `{selector_name}` cannot be used to select type `{selected_type_name}` from against type `{type_name}`"
    )]
    UnsupportedSelector {
        type_name: &'static str,
        selected_type_name: &'static str,
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

    #[error("Invalid selection mode: {0}")]
    InvalidSelectionMode(&'static str),

    #[error("Dag selection send error: {0}")]
    DagSelectionStream(#[from] DagSelectionSenderError),

    #[error("Node selection send error: {0}")]
    NodeSelectionStream(#[from] NodeSelectionSenderError),

    #[error("Selector depth error: {0}: {1}")]
    SelectorDepth(&'static str, usize),

    #[error("Selector range error: {0}")]
    SelectorRange(&'static str),

    #[error("Unknown codec: {0}")]
    UnknownCodec(u64),

    #[error("{0}")]
    Custom(anyhow::Error),
}

impl Error {
    pub fn unsupported_selector<T, U>(selector: &Selector) -> Self
    where
        T: Representation,
        U: Representation,
    {
        Self::UnsupportedSelector {
            type_name: <T as Representation>::NAME,
            selected_type_name: <U as Representation>::NAME,
            selector_name: selector.name(),
        }
    }

    pub fn missing_next_selector(selector: &Selector) -> Self {
        Self::MissingNextSelector(selector.name())
    }

    pub fn invalid_type_selection<T, U>() -> Self
    where
        T: Representation,
        U: Representation,
    {
        Self::InvalidTypeSelection {
            actual_type_name: <T as Representation>::NAME,
            desired_type_name: <U as Representation>::NAME,
        }
    }

    #[inline]
    pub fn decoder<E>(err: E) -> Self
    where
        E: de::Error + 'static,
    {
        Error::Decoder(Box::new(err))
    }

    #[inline]
    pub fn encoder<E>(err: E) -> Self
    where
        E: ser::Error + 'static,
    {
        Error::Encoder(Box::new(err))
    }

    #[inline]
    pub fn de_error<E>(self) -> E
    where
        E: de::Error + 'static,
    {
        match self {
            Self::Decoder(inner) if inner.is::<E>() => *inner.downcast::<E>().unwrap(),
            Self::Decoder(inner) => E::custom(inner),
            _ => unreachable!(),
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
