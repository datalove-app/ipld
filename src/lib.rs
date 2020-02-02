//! An implementation of core `Ipld` types and interfaces.

mod borrowed;
mod canon;
mod codec;
mod error;

pub use borrowed::{
    Ipld as BorrowedIpld, IpldListIter as BorrowedIpldListIter, IpldMapIter as BorrowedIpldMapIter,
};
pub use codec::{Codec, CodecExt, IpldVisitor};
pub use error::Error;

#[cfg(test)]
mod test {}
