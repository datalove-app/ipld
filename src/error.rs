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

    #[error("")]
    Custom(),
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
