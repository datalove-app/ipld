use crate::dev::*;
use cid::Error as CidError;
use serde::{de, ser};
use std::{
    convert::Infallible, error::Error as StdError, num::TryFromIntError, string::FromUtf8Error,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("CID error: {0}")]
    Cid(CidError),

    #[error("IPLD format encoding error: {0}")]
    Encoder(anyhow::Error),

    #[error("IPLD format decoding error: {0}")]
    Decoder(anyhow::Error),

    #[error("Selector Context error: {0}")]
    Context(#[from] anyhow::Error),

    #[error(
        "Invalid selector: type {type_name} does not support selecting against {selector_name}"
    )]
    UnsupportedSelector {
        type_name: &'static str,
        selector_name: &'static str,
    },

    #[error("Other error: {0}")]
    Other(String),
}

impl Error {
    pub fn unsupported_selector<T, S>(selector: &S) -> Self
    where
        S: ISelector,
        T: Representation,
    {
        Self::UnsupportedSelector {
            type_name: <T as Representation>::NAME,
            selector_name: selector.name(),
        }
    }
}

impl From<CidError> for Error {
    fn from(err: CidError) -> Self {
        Error::Cid(err)
    }
}
