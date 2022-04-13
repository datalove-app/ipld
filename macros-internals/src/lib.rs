pub extern crate derive_more;
// #[macro_use]
// pub extern crate impls;
// #[macro_use]
// pub extern crate static_assertions;

pub mod common;
pub mod schema;
pub mod selector;

pub use schema::SchemaDefinition;
pub use selector::{RootSelectorDefinition, SelectorDefinition};

pub mod dev {
    pub use crate::common::*;
    pub use crate::define_newtype;
    pub use crate::impl_advanced_parse;
    pub use crate::parse_kwarg;
    pub use crate::schema::{expand::*, parse::*, *};
    pub use crate::selector::*;
    pub use crate::*;

    pub use derive_more;
    // pub use impls;
    // pub use static_assertions;
    // pub use tylift;
}
