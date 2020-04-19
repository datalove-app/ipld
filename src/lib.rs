//! The [types](), [traits]() and [macros]() for [`IPLD`]() [Data Model](), [Schemas](), [Representations](), and [Selectors]().

#![feature(specialization)]
// #![deny(missing_docs)]

#[path = "formats/mod.rs"]
mod _formats;
mod block;
mod error;
// mod ipld;
// #[doc(inline)]
pub mod representation;
pub mod selectors;
pub mod value;

pub use _formats::{Decoder, Encoder, Format};
pub use error::Error;
// pub use ipld::borrowed::Ipld as BorrowedIpld;
#[doc(inline)]
pub use representation::{
    Context, Executor, Representation, Select, Selection, SelectionResult, SelectionStream,
};
#[doc(inline)]
pub use selectors::Selector;
// #[doc(inline)]
// pub use value::Value;

///
pub mod formats {
    #[cfg(feature = "dag-cbor")]
    pub use crate::_formats::dag_cbor::DagCbor;

    #[cfg(all(feature = "dag-json", not(feature = "simd")))]
    pub use crate::_formats::dag_json::DagJson;
    // #[cfg(all(feature = "dag-json", feature = "simd"))]
    // pub use crate::_formats::dag_json_simd::DagJson;
}

#[cfg(feature = "macros")]
#[doc(inline)]
pub use ipld_macros::{ipld_attr, schema, selector};

///
pub mod dev {
    pub use crate::_formats::*;
    pub use crate::formats::*;
    #[doc(inline)]
    pub use crate::impl_root_select;
    pub use crate::prelude::*;
    pub use crate::representation::*;
    pub use crate::selectors::*;
    pub use crate::value::*;

    // pub use async_stream::stream;
    pub use futures::{self, Stream, StreamExt};
    pub use ipld_macros_internals as macros;
    pub use serde::{
        self,
        de::{DeserializeOwned, DeserializeSeed, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use serde_repr;
}

///
pub mod prelude {
    pub use crate::*;
    pub use formats::*;
    pub use cid::{self, Cid, ToCid};
    pub use multibase::{self, Base as Multibase};
    pub use multihash::{self, Multihash, MultihashDigest, MultihashRef};
}

#[cfg(test)]
mod test {}
