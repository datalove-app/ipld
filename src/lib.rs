//! Types, traits and macros for the [IPLD](https://ipld.io)
//! [Data Model](https://github.com/ipld/specs/blob/master/data-model-layer/data-model.md),
//! [Schemas](https://github.com/ipld/specs/blob/master/schemas/introduction.md),
//! [Representations](https://github.com/ipld/specs/blob/master/schemas/representations.md), and
//! [Selectors](https://github.com/ipld/specs/blob/master/selectors/selectors.md)
//! [specifications](https://github.com/ipld/specs).

// #![feature(generic_associated_types)]
#![feature(specialization)]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]

#[path = "codecs/mod.rs"]
mod _codecs;
mod error;

pub mod representation;
pub mod selectors;
pub mod value;

#[doc(inline)]
pub use _codecs::{Codec, Decoder, Encoder, IpldVisitorExt};
#[doc(inline)]
pub use error::Error;
// pub use ipld::borrowed::Ipld as BorrowedIpld;
#[doc(inline)]
pub use representation::{Context, Representation, Select};
#[doc(inline)]
pub use selectors::Selector;
// #[doc(inline)]
// pub use value::Value;

#[doc(inline)]
pub use ipld_macros::{ipld_attr, schema, selector};

///
pub mod codecs {
    // #[doc(inline)]
    // pub use crate::_codecs::Error as FormatError;
    #[cfg(feature = "dag-cbor")]
    pub use crate::_codecs::dag_cbor::DagCbor;

    #[cfg(feature = "dag-json")]
    pub use crate::_codecs::dag_json::DagJson;
}

/// All the exports and re-exports necessary for using `ipld`.
pub mod prelude {
    pub use crate::*;
    pub use codecs::*;
    pub use value::*;
    // pub use async_trait::async_trait;
    pub use cid::{self, Cid, CidGeneric};
    pub use multibase::{self, Base as Multibase};
    pub use multihash::{
        self, typenum, Code as DefaultMultihash, MultihashDigest, Size as MultihashSize,
        U64 as DefaultMultihashSize,
    };
    pub use serde::{
        self,
        de::{DeserializeOwned, DeserializeSeed, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use std::{
        fmt::Debug,
        io::{Read, Write},
    };
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
pub mod dev {
    // pub use crate::_codecs::*;
    #[doc(inline)]
    pub use crate::impl_root_select;
    pub use crate::prelude::*;
    pub use crate::representation::*;
    pub use crate::selectors::*;

    // dependency re-exports for macro convenience
    // pub use async_stream::stream;
    pub use anyhow;
    pub use bytes;
    pub use futures::{self, Stream, StreamExt};
    pub use ipld_macros_internals as macros;
    pub use serde_repr;
}
