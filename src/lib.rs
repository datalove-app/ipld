//! Types, traits and macros for the [IPLD](https://ipld.io)
//! [Data Model](https://github.com/ipld/specs/blob/master/data-model-layer/data-model.md),
//! [Schemas](https://github.com/ipld/specs/blob/master/schemas/introduction.md),
//! [Representations](https://github.com/ipld/specs/blob/master/schemas/representations.md), and
//! [Selectors](https://github.com/ipld/specs/blob/master/selectors/selectors.md)
//! [specifications](https://github.com/ipld/specs).
//!
//! TODO:
//!     - utf-8 string handling/normalization
//!     - replace boxed callbacks with a ref

#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#[forbid(unsafe_code)]
//
#[path = "cid.rs"]
mod cid_;
#[path = "codecs/mod.rs"]
mod codecs_;
mod data_model;
mod error;
mod multicodec;
#[path = "multihash.rs"]
mod multihash_;
mod representation;
mod selectors;

#[doc(inline)]
pub use error::Error;
#[doc(inline)]
pub use specs::*;

mod specs {
    use super::*;

    // codecs
    pub use crate::multicodec::Codec;

    #[cfg(feature = "dag-cbor")]
    pub use crate::codecs_::dag_cbor::DagCbor;
    #[cfg(feature = "dag-json")]
    pub use crate::codecs_::dag_json::DagJson;
    // #[cfg(feature = "dag-pb")]
    // pub use crate::codecs_::dag_pb::DagPb;
    #[cfg(feature = "multicodec")]
    pub use crate::multicodec::Multicodec;

    // multiformats
    pub use multibase::Base as Multibase;
    pub use multihash::{
        self, Code as Multihashes, Hasher as _, Multihash as DefaultMultihash,
        MultihashDigest as _, MultihashGeneric,
    };
    pub use multihash_::Multihash;

    // cid
    pub use crate::cid_::*;
    pub use cid::{Cid as DefaultCid, CidGeneric, Version};

    // data model, schemas and representation
    pub use crate::data_model::*;
    pub use crate::representation::Representation;
    pub use ipld_macros::{ipld_attr, schema};

    // selectors
    pub use crate::selectors::{Context, Params, Select, Selector};
    pub use ipld_macros::selector;
}

/// All the exports and re-exports necessary for using `ipld`.
pub mod prelude {
    #[doc(inline)]
    pub use crate::codecs_::IpldVisitorExt;
    #[doc(inline)]
    pub use crate::specs::*;
    #[doc(inline)]
    pub use crate::{Any, Cid, Context, Error, Representation, Select, Selector};

    #[doc(hidden)]
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
pub mod dev {
    pub use crate::{prelude::*, representation::*, selectors::*};

    // dependency re-exports for macro convenience
    #[doc(hidden)]
    pub use anyhow;
    #[doc(hidden)]
    pub use bytes;
    // pub use erased_serde::{Deserializer as ErasedDeserializer, Serializer as ErasedSerializer};
    /// Useful macros for aiding in providing bespoke IPLD support.
    pub mod macros {
        pub use crate::impl_selector_seed_serde;
        // pub use const_format::*;
        pub use cfg_if::cfg_if;
        pub use ipld_macros_internals::*;
    }

    #[doc(hidden)]
    pub use serde::{
        self,
        de::{
            self, DeserializeOwned, DeserializeSeed, EnumAccess, Error as _, IgnoredAny,
            IntoDeserializer as _, MapAccess, SeqAccess, VariantAccess, Visitor,
        },
        ser::{self, Error as _},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    // #[doc(hidden)]
    // pub use serde_repr;
}
