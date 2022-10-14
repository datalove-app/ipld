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
#[cfg_attr(not(feature = "std"), no_std)]
#[forbid(unsafe_code)]
//
#[path = "cid.rs"]
mod cid_;
#[path = "codecs/mod.rs"]
mod codecs_;
mod compat;
mod data_model;
mod error;
mod multicodec;
#[path = "multihash.rs"]
mod multihash_;
mod representation;
mod select;

pub use cid;
#[doc(inline)]
pub use error::Error;
pub use multibase;
pub use multihash;

pub use specs::*;

mod specs {
    use super::*;

    // codecs
    #[cfg(feature = "dag-cbor")]
    pub use crate::codecs_::dag_cbor::DagCbor;
    #[cfg(feature = "dag-json")]
    pub use crate::codecs_::dag_json::DagJson;
    pub use crate::multicodec::{Codec, Multicodec};

    // multiformats
    pub use multibase::Base as Multibase;
    #[doc(hidden)]
    pub use multihash::{Hasher as _, Multihash as DefaultMultihash, MultihashDigest as _};
    pub use multihash_::Multihash;

    // cid
    pub use crate::cid_::Cid;
    #[doc(hidden)]
    pub use cid::{Cid as DefaultCid, CidGeneric, Version};

    // data model, schemas and representations
    pub use crate::data_model::*;
    #[doc(hidden)]
    pub use crate::representation::TypedKind as _;
    pub use crate::representation::{strategies::Strategy, Kind, Representation};
    pub use ipld_macros::{ipld_attr, schema};

    // selectors
    pub use crate::select::{Context, Params, Select, Selector};
    // pub use ipld_macros::selector;
}

/// All the exports and re-exports necessary for using `ipld`.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::representation::strategies::*;
    #[doc(inline)]
    pub use crate::{codecs_::LinkVisitor, specs::*, Error};
    #[doc(hidden)]
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
#[cfg_attr(not(feature = "dev"), doc(hidden))]
pub mod dev {
    #[doc(hidden)]
    pub use crate::maybestd;
    pub use crate::{prelude::*, representation::*, select::*};

    // dependency re-exports for macro convenience
    #[doc(hidden)]
    pub use anyhow;
    #[doc(hidden)]
    pub use bytes;
    /// Useful macros for aiding in providing bespoke IPLD support.
    pub mod macros {
        pub use crate::repr_serde;
        // pub use const_format::*;
        pub use ipld_macros_internals::dev::*;
    }

    #[doc(hidden)]
    pub use ipld_macros_internals::dev::typenum;
    #[doc(hidden)]
    pub use ipld_macros_internals::dev::typenum_macro;
    #[doc(hidden)]
    pub use serde::{
        self,
        de::{
            self, DeserializeOwned, DeserializeSeed, EnumAccess, Error as _, IgnoredAny,
            IntoDeserializer as _, MapAccess, SeqAccess, VariantAccess, Visitor,
        },
        ser::{self, Error as _},
    };
}

///
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub mod maybestd {
    extern crate alloc;

    pub use alloc::{boxed, collections, rc, vec};
    pub use core::{
        borrow, cell, cmp, convert, fmt, hash, iter, marker, num, ops, primitive, str, string, sync,
    };
    pub use core2::{error, io};
    pub use std::path;
}
#[cfg(feature = "std")]
#[doc(hidden)]
pub mod maybestd {
    pub use core2::{error, io};
    pub use std::path;
    pub use std::{
        borrow, boxed, cell, cmp, collections, convert, fmt, hash, iter, marker, num, ops,
        primitive, rc, str, string, sync, vec,
    };
}
