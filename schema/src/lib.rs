//! IPLD Schemas and Representations

// extern crate derive_more;
#[macro_use]
extern crate lazy_static;

//mod link;
mod executor;
mod representation;
mod selectors;

// public internal and dependency exports
pub use executor::*;
pub use representation::*;
pub use selectors::Selector;

// /// External imports, re-exported for convenience and for `libipld-schema-derive`
pub mod prelude {
    // pub use crate::{advanced, derive::*, schema};
    // pub use crate::representation::
    // pub use ipld::prelude::*;
}

// TODO: impl and export "std" types: Null, Bool, Int, Float
// TODO: impl and export `Value` type

pub mod dev {
    pub use crate::*;
    pub use serde::{
        self,
        de::{DeserializeSeed, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub use serde_repr;
}
