//! `schema!` macro.
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types

extern crate proc_macro;

use ipld_schema_derive_internals::{def_internal_flag, SchemaDefinition};
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{custom_keyword, parse_macro_input};

// exports an `internal_ipld_schema` attribute proc macro
def_internal_flag!();

// TODO? proc_macro_hack? to possibly parse typedefs from strings in functions?
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    let schema = parse_macro_input!(input as SchemaDefinition);
    TokenStream::from(schema.expand())
}
