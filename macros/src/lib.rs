//! `schema!` macro.
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types

use proc_macro_hack::proc_macro_hack;

// pub use ipld_macros_internals::{};

/// todo: docs
pub use ipld_macros_hack::schema;

/// todo: docs
#[proc_macro_hack]
pub use ipld_macros_hack::selector;

/// todo: docs
pub use ipld_macros_hack::ipld_attr;

// def_attributes!(
//     #[doc(hidden)]
//     ipld_macros_internal
// );
// def_attributes!(try_from);
// def_attributes!(wrap);
