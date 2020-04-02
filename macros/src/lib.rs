//! `schema!` macro.
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types

extern crate proc_macro;

use ipld_macros_internals::{
    def_attributes,
    SchemaDefinition,
    SelectorDefinition,
};
use proc_macro::TokenStream;
use syn::parse_macro_input;

///
/// TODO? proc_macro_hack? to possibly parse typedefs from strings in functions?
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    let schema = parse_macro_input!(input as SchemaDefinition);
    TokenStream::from(schema.expand())
}

///
#[proc_macro]
pub fn selector(input: TokenStream) -> TokenStream {
    let selector = parse_macro_input!(input as SelectorDefinition);
    TokenStream::from(selector.expand())
}

// ///
// #[proc_macro]
// pub fn value(input: TokenStream) -> TokenStream {
//     let value = parse_macro_input!(input as ValueDefinition);
//     TokenStream::from(value.expand())
// }

// exports attribute proc macros used with `schema!`.
def_attributes!();
