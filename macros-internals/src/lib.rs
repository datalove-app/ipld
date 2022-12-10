pub mod common;
pub mod schema;
pub mod schema2;
pub mod selector;

pub use schema::SchemaDefinition;
pub use selector::{RootSelectorDefinition, SelectorDefinition};

/// Convenient re-exports for development.
pub mod dev {
    pub use crate::common::*;
    pub use crate::derive_newtype;
    pub use crate::impl_advanced_parse;
    pub use crate::parse_kwarg;
    pub use crate::schema::{expand::*, parse::*, *};
    pub use crate::selector::*;
    pub use crate::*;

    pub use darling;
    pub use derive_more;
}
