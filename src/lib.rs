//! Types, traits and macros for the [IPLD](https://ipld.io)
//! [Data Model](https://github.com/ipld/specs/blob/master/data-model-layer/data-model.md),
//! [Schemas](https://github.com/ipld/specs/blob/master/schemas/introduction.md),
//! [Representations](https://github.com/ipld/specs/blob/master/schemas/representations.md), and
//! [Selectors](https://github.com/ipld/specs/blob/master/selectors/selectors.md)
//! [specifications](https://github.com/ipld/specs).
//!
//! Notes, 20220108:
//! Encoder/Decoder knows how to write to block writers, read from block readers
//! Context:
//!     needs to know how to provide readers and writers
//!     for other use cases (super traits?):
//!         SigningContext: must be signer-aware
//!         Transform/Encryption: must know how
//!         ? SelectionContext
//! [new] Representation, that uses a Context to:
//!     request readers to decode block, then selects within it
//!         may be able to use a SelectorSeed to get a specific val
//!     request writers for writing itself
//!         if dirty, will first fetch writers for nested items
//!
//! I really want three things:
//! - a Representation method to merge a type at a path, and update all parent blocks
//!     ?Context
//!     ::merge(&mut self, Value, path)
//!         -> by default, acts as a register
//!     ? how do we know what blocks are interested in updated links?
//! - Verifiable val/range selectors
//!     ::resolve(&self, path) -> (Value, Proof, ?RemPath?)
//!     ::verify(path, value, proof)

// #![feature(generic_associated_types)]
#![feature(box_patterns)]
#![feature(specialization)]
// #![recursion_limit = "256"]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]

#[forbid(unsafe_code)]

///
#[path = "codecs/mod.rs"]
mod _codecs;
mod block;
mod error;

pub mod advanced_layouts;
pub mod representation;
pub mod selectors;
pub mod value;

#[doc(inline)]
pub use _codecs::{Codec, Decoder, Encoder, IpldVisitorExt};
#[doc(inline)]
pub use error::Error;
// pub use ipld::borrowed::Ipld as BorrowedIpld;
#[doc(inline)]
pub use representation::Representation;
#[doc(inline)]
pub use selectors::{Context, Select, Selector};
#[doc(inline)]
pub use value::Value;

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

    #[cfg(feature = "dag-pb")]
    pub use crate::_codecs::dag_pb::DagPb;

    #[cfg(feature = "multicodec")]
    pub use crate::_codecs::multicodec::Multicodec;
}

/// All the exports and re-exports necessary for using `ipld`.
pub mod prelude {
    pub use crate::advanced_layouts::*;
    pub use crate::block::BlockMeta;
    pub use crate::*;
    pub use codecs::*;
    pub use value::*;
    // pub use async_trait::async_trait;
    pub use cid::{self, Cid, CidGeneric};
    pub use multibase::{self, Base as Multibase};
    pub use multihash::{
        self, Code as Multihash, Multihash as DefaultMultihash, MultihashDigest,
    };
    pub use serde::{Deserialize, Serialize};
    pub use std::{
        fmt::Debug,
        io::{Read, Write},
    };

    ///
    pub const DEFAULT_MULTIHASH_SIZE: usize = 64;
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
pub mod dev {
    pub use crate::_codecs::*;
    #[doc(inline)]
    pub use crate::impl_root_select;
    pub use crate::prelude::*;
    pub use crate::representation::*;
    // pub use crate::runtime::*;
    pub use crate::selectors::*;

    // dependency re-exports for macro convenience
    // pub use async_stream::stream;
    pub use anyhow;
    pub use bytes;
    pub use erased_serde::{Deserializer as ErasedDeserializer, Serializer as ErasedSerializer};
    // pub use futures::{self, Stream, StreamExt};
    pub use impls;
    pub use ipld_macros_internals as macros;
    pub use serde::{
        de::{
            DeserializeOwned, DeserializeSeed, EnumAccess, Error as _, MapAccess, SeqAccess,
            VariantAccess, Visitor,
        },
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use serde_repr;
}
