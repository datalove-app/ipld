use cid::Error as CidError;
use failure::Fail;
use std::{convert::Infallible, num::TryFromIntError, string::FromUtf8Error};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Cid error: {}", _0)]
    Cid(CidError),

    #[fail(display = "IPLD Codec error: {}", _0)]
    Codec(failure::Error),

    #[fail(display = "Invalid data received from context: {}", _0)]
    Context(failure::Error),

    #[fail(display = "Invalid IPLD data type:: {}", _0)]
    Ipld(failure::Error),
}

impl From<CidError> for Error {
    fn from(err: CidError) -> Self {
        Error::Cid(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::Codec(err.into())
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Codec(err.into())
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        Error::Codec(err.into())
    }
}
