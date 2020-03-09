//! An implementation of core `IPLD` types and interfaces.

#![feature(specialization)]
// #![deny(missing_docs)]

#[path = "formats/mod.rs"]
mod _formats;
mod block;
mod error;
mod ipld;

// pub use error::Error;
pub use _formats::{Decoder, Encoder, Format, IpldVisitorExt};
// pub use ipld::borrowed::{
//     Ipld as BorrowedIpld, IpldListIter as BorrowedIpldListIter, IpldMapIter as BorrowedIpldMapIter,
// };

pub mod formats {
    #[cfg(feature = "dag-cbor")]
    pub use crate::_formats::dag_cbor::DagCbor;

    #[cfg(feature = "dag-json")]
    pub use crate::_formats::dag_json::DagJson;
}

pub mod prelude {
    pub use crate::*;
    pub use async_trait::async_trait;
    pub use cid::{self, Cid, ToCid};
    pub use multibase::{self, Base as Multibase};
    pub use multihash::{self, Multihash, MultihashDigest, MultihashRef};
    pub use serde::{
        self,
        de::{DeserializeOwned, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use std::{
        fmt::Debug,
        io::{Read, Write},
    };
}

#[cfg(test)]
mod test {}
