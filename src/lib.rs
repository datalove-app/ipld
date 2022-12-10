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
//!     - TryFrom delegation, if custom types want value-checking

#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#[cfg_attr(not(feature = "std"), no_std)]
// #[cfg(feature = "std")] extern crate std;
#[doc(hidden)]
extern crate self as __ipld;

// #[forbid(unsafe_code)]
mod advanced;
#[path = "cid.rs"]
mod cid_;
#[path = "codecs/mod.rs"]
pub(crate) mod codecs_;
mod compat;
mod data_model;
mod error;
// todo support fmt::Display / writing to a fmt::Formatter for multibase/hash/addr
mod multicodec;
#[path = "multihash.rs"]
mod multihash_;
mod representation;
mod select;

pub use error::Error;
#[cfg(feature = "multiaddr")]
pub use multiaddr::{self, Multiaddr};

// specification implementations
pub use {
    // cid
    cid::{self, Cid as DefaultCid, CidGeneric, Version},
    cid_::Cid,

    // codecs
    codecs_::*,

    // data model, schemas and representations
    data_model::*,
    ipld_macros::{
        ipld_attr,
        schema,
        // selector
        Representation,
        // Select, Patch
    },

    // multiformats
    // multiaddr::Multiaddr,
    multibase::{self, Base as Multibase},
    multicodec::Multicodec,
    multihash::{self, Hasher as _, Multihash as DefaultMultihash, MultihashDigest as _},
    multihash_::Multihasher,

    // representations
    representation::{Kind, Representation, Strategy},

    // selectors
    select::{Context, Params, Select, Selector},
};

/// Useful macros for implementing IPLD traits.
mod macros {
    pub use crate::repr_serde;
    // pub use derive_builder::Builder;
    // pub use const_format::*;
    pub use ipld_macros_internals::dev::*;
}

/// All exports from `ipld::prelude`, plus re-exports of first- and third-party
/// dependencies to aid developers wanting to implement or extend `ipld` behaviour.
#[cfg_attr(not(feature = "dev"), doc(hidden))]
pub mod dev {
    pub use crate::maybestd;
    pub use crate::{cid_::*, macros::*, representation::*, select::*, *};

    // dependency re-exports for macro convenience
    #[doc(hidden)]
    pub use anyhow;
    #[doc(hidden)]
    pub use bytes;

    #[doc(hidden)]
    pub use serde::{
        self,
        de::{
            self, DeserializeOwned, DeserializeSeed, Deserializer, EnumAccess, Error as _,
            IgnoredAny, IntoDeserializer as _, MapAccess, SeqAccess, VariantAccess, Visitor,
        },
        ser::{self, Error as _, Serializer},
        Deserialize, Serialize,
    };
}

///
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub mod maybestd {
    #[cfg(feature = "alloc")]
    extern crate alloc;
    #[cfg(feature = "alloc")]
    pub use alloc::{boxed, collections, rc, vec};

    pub use core::{error as _, io as _, *};
    pub use core2::{error, io};
    // todo: replace this
    pub use std::path;
}
#[cfg(feature = "std")]
#[doc(hidden)]
pub mod maybestd {
    pub use core2::{error, io};
    pub use std::{error as _, io as _, *};
}
