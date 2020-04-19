use crate::dev::*;
use cid::Error as CidError;
use std::{
    convert::Infallible, error::Error as StdError, num::TryFromIntError, string::FromUtf8Error,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cid error: {0}")]
    Cid(CidError),

    #[error("IPLD format error:")]
    Format(),

    #[error("Invalid IPLD data type:")]
    Ipld(),

    #[error(
        "Invalid selector: type {type_name} does not support selecting against {selector_name}"
    )]
    UnsupportedSelector {
        type_name: &'static str,
        selector_name: &'static str,
    },

    #[error("")]
    Custom(),
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

// impl From<FromUtf8Error> for Error {
//     fn from(err: FromUtf8Error) -> Self {
//         Error::Codec(err.into())
//     }
// }

// impl From<TryFromIntError> for Error {
//     fn from(err: TryFromIntError) -> Self {
//         Error::Codec(err.into())
//     }
// }

// impl From<Infallible> for Error {
//     fn from(err: Infallible) -> Self {
//         Error::Codec(err.into())
//     }
// }
