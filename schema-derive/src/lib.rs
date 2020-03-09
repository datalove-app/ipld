//! `schema!` macro.
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types

// #[macro_use]
// extern crate impls;

#[macro_use]
mod advanced;
#[macro_use]
mod schema;
