//! Types, traits and macros for the [IPLD](https://ipld.io)
//! [Data Model](https://github.com/ipld/specs/blob/master/data-model-layer/data-model.md),
//! [Schemas](https://github.com/ipld/specs/blob/master/schemas/introduction.md),
//! [Representations](https://github.com/ipld/specs/blob/master/schemas/representations.md), and
//! [Selectors](https://github.com/ipld/specs/blob/master/selectors/selectors.md)
//! [specifications](https://github.com/ipld/specs).

#![feature(specialization)]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]

#[forbid(unsafe_code)]

///
#[path = "cid.rs"]
mod _cid;
#[path = "codecs/mod.rs"]
mod _codecs;
#[path = "multihash.rs"]
mod _multihash;
mod advanced_layouts;
mod block;
mod error;
#[cfg(feature = "multicodec")]
mod multicodec;
mod representation;
mod selectors;
mod value;

#[doc(inline)]
pub use error::Error;
// pub use ipld::borrowed::Ipld as BorrowedIpld;
#[doc(inline)]
pub use specs::*;
#[doc(inline)]
pub use value::Value;

mod specs {
    use super::*;

    // codecs
    #[cfg(feature = "dag-cbor")]
    pub use crate::_codecs::dag_cbor::DagCbor;
    #[cfg(feature = "dag-json")]
    pub use crate::_codecs::dag_json::DagJson;
    #[cfg(feature = "dag-pb")]
    pub use crate::_codecs::dag_pb::DagPb;
    pub use crate::_codecs::{Codec, Decoder, Encoder};

    // multiformats
    #[cfg(feature = "multicodec")]
    pub use crate::multicodec::Multicodec;
    pub use _multihash::Multihash;
    pub use multibase::Base as Multibase;
    pub use multihash::{
        self, Code as Multihashes, Hasher as _, Multihash as DefaultMultihash,
        MultihashDigest as _, MultihashGeneric,
    };

    // cid
    pub use crate::_cid::Cid;
    pub use cid::{Cid as DefaultCid, CidGeneric, Version};

    // representation
    pub use crate::representation::Representation;

    // schema
    pub use ipld_macros::{ipld_attr, schema};

    // selectors
    pub use crate::selectors::{Context, Select, Selector};
    pub use ipld_macros::selector;
}

// pub use

/// All the exports and re-exports necessary for using `ipld`.
pub mod prelude {
    #[doc(inline)]
    pub use crate::_codecs::IpldVisitorExt;
    #[doc(inline)]
    pub use crate::advanced_layouts::*;
    #[doc(inline)]
    pub use crate::specs::*;
    #[doc(inline)]
    pub use crate::value::*;
    #[doc(inline)]
    pub use crate::{Cid, Context, Error, Representation, Select, Selector, Value};

    ///
    pub const DEFAULT_MULTIHASH_SIZE: usize = 64;

    #[doc(hidden)]
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
pub mod dev {
    pub use std::io::{Read, Write};

    // pub use crate::impl_root_select;
    pub use crate::{impl_ipld_serde, prelude::*, representation::*, selectors::*};

    // dependency re-exports for macro convenience
    pub use anyhow;
    pub use bytes;
    // pub use erased_serde::{Deserializer as ErasedDeserializer, Serializer as ErasedSerializer};
    pub use ipld_macros_internals as macros;
    pub use serde::{
        self,
        de::{
            DeserializeOwned, DeserializeSeed, EnumAccess, Error as _, IgnoredAny, MapAccess,
            SeqAccess, VariantAccess, Visitor,
        },
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use serde_repr;
}
