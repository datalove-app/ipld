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
//!     - ?? refactor Matcher* logic to only select nodes

#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#[cfg_attr(not(feature = "std"), no_std)]
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
    #[cfg(feature = "dag-cbor")]
    pub use crate::codecs_::dag_cbor::DagCbor;
    #[cfg(feature = "dag-json")]
    pub use crate::codecs_::dag_json::DagJson;
    // #[cfg(feature = "dag-pb")]
    // pub use crate::codecs_::dag_pb::DagPb;
    pub use crate::multicodec::{Codec, Multicodec};

    // multiformats
    pub use multibase::Base as Multibase;
    pub use multihash::{self, Hasher as _, Multihash as DefaultMultihash, MultihashDigest as _};
    pub use multihash_::Multihash;

    // cid
    pub use crate::cid_::*;
    pub use cid::{Cid as DefaultCid, CidGeneric, Version};

    // data model, schemas and representation
    pub use crate::data_model::*;
    pub use crate::representation::{Kind, Representation, TypedKind as _};
    pub use ipld_macros::{ipld_attr, schema};

    // selectors
    pub use crate::selectors::{Context, Params, Select, Selector};
    // pub use ipld_macros::selector;
}

/// All the exports and re-exports necessary for using `ipld`.
pub mod prelude {
    #[doc(inline)]
    pub use crate::codecs_::IpldVisitorExt;
    #[doc(inline)]
    pub use crate::specs::*;
    #[doc(inline)]
    pub use crate::Error;
    #[doc(hidden)]
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
pub mod dev {
    #[doc(hidden)]
    pub use crate::maybestd;
    pub use crate::{prelude::*, representation::*, selectors::*};

    // dependency re-exports for macro convenience
    #[doc(hidden)]
    pub use anyhow;
    #[doc(hidden)]
    pub use bytes;
    /// Useful macros for aiding in providing bespoke IPLD support.
    pub mod macros {
        pub use crate::impl_selector_seed_serde;
        // pub use const_format::*;
        pub use ipld_macros_internals::dev::*;
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
    #[doc(hidden)]
    pub use ipld_macros_internals::dev::typenum;
    #[doc(hidden)]
    pub use ipld_macros_internals::dev::typenum_macro;
}

///
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub mod maybestd {
    extern crate alloc;

    pub use alloc::{boxed, collections, rc, vec::Vec};
    pub use core::{borrow, cell, cmp, convert, fmt, hash, iter, marker, ops, str, sync};
    pub use core2::{error, io};
}
#[cfg(feature = "std")]
#[doc(hidden)]
pub mod maybestd {
    pub use core2::{error, io};
    pub use std::{
        borrow, boxed, cell, cmp, collections, convert, fmt, hash, iter, marker, ops, rc, str,
        sync, vec::Vec,
    };
}
