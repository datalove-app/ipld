//! IPLD Schemas and Representations

// extern crate derive_more;
#[macro_use]
extern crate lazy_static;

//mod link;
mod representation;

// public internal and dependency exports
// pub use crate::representation::{context::Context, Representation};

/// External imports, re-exported for convenience and for `libipld-schema-derive`
pub mod dev {
    pub use async_trait::async_trait;
    pub use cid::{self, Cid};
    pub use serde::{
        de::{DeserializeOwned, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };

    // #[cfg(feature = "derive")]
    // #[macro_use]
    // pub use ipld_schema_derive::{advanced, schema};
}
