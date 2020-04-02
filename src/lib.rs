//! An implementation of core `IPLD` types and interfaces.

#![feature(const_generics)]
#![feature(specialization)]
// #![deny(missing_docs)]

#[path = "formats/mod.rs"]
mod _formats;
mod block;
mod error;
// mod ipld;
mod representation;
mod selectors;

// pub use error::Error;
pub use _formats::{Decoder, Encoder, Format};
// pub use ipld::borrowed::Ipld as BorrowedIpld;
pub use representation::{Context, Executor, Representation};
pub use selectors::{Select, Selector};

pub mod formats {
    #[cfg(feature = "dag-cbor")]
    pub use crate::_formats::dag_cbor::DagCbor;

    #[cfg(feature = "dag-json")]
    pub use crate::_formats::dag_json::DagJson;
}

pub mod dev {
    pub use crate::_formats::*;
    pub use crate::prelude::*;
    pub use crate::representation::*;
    pub use crate::selectors::*;

    pub use serde::{
        self,
        de::{DeserializeSeed, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use serde_repr;
    pub use std::{
        fmt::Debug,
        io::{Read, Write},
    };
}
pub mod prelude {
    pub use crate::*;
    pub use formats::*;
    pub use cid::{self, Cid, ToCid};
    pub use multibase::{self, Base as Multibase};
    pub use multihash::{self, Multihash, MultihashDigest, MultihashRef};
}

#[cfg(test)]
mod test {}
