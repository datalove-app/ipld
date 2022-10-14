pub mod common;
pub mod schema;
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

    pub use derive_more;
    pub use typenum;
    pub use typenum_macro;
}
