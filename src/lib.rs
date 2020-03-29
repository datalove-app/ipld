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
pub use ipld::borrowed::Ipld as BorrowedIpld;

pub mod formats {
    #[cfg(feature = "dag-cbor")]
    pub use crate::_formats::dag_cbor::DagCbor;

    #[cfg(feature = "dag-json")]
    pub use crate::_formats::dag_json::DagJson;
}

pub mod dev {
    pub use crate::prelude::*;
    pub use serde::{self, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
    pub use std::{
        fmt::Debug,
        io::{Read, Write},
    };
}
pub mod prelude {
    pub use crate::*;
    pub use async_trait::async_trait;
    pub use cid::{self, Cid, ToCid};
    pub use multibase::{self, Base as Multibase};
    pub use multihash::{self, Multihash, MultihashDigest, MultihashRef};
}

#[cfg(test)]
mod test {}
