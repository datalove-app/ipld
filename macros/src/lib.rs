//! `schema!` macro.
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types

use ipld_macros_internals::{
    // def_attributes,
    RootSelectorDefinition,
    // ValueDefinition,
    SchemaDefinition,
};
use proc_macro::TokenStream;
use syn::parse_macro_input;

///
///
/// TODO: rename attrs to ipld_macros(), add a `wrap = Box/Rc/etc` attr
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    let schema = parse_macro_input!(input as SchemaDefinition);
    TokenStream::from(schema.expand())
}

/// A no-op macro, used to help the `schema!` and `selector!` macros capture
/// relevant arguments.
#[proc_macro_attribute]
pub fn ipld_attr(_attr: TokenStream, items: TokenStream) -> TokenStream {
    items
}

///
#[proc_macro]
pub fn selector(input: TokenStream) -> TokenStream {
    let selector = parse_macro_input!(input as RootSelectorDefinition);
    TokenStream::from(selector.expand())
}

// def_attributes!(
//     #[doc(hidden)]
//     ipld_macros_internal
// );
// def_attributes!(try_from);
// def_attributes!(wrap);
