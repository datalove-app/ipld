//! `schema!` macro.
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types

extern crate proc_macro;

use ipld_macros_internals::{
    schema2::DeriveRepresentation, RootSelectorDefinition, SchemaDefinition,
};
use proc_macro::TokenStream;
use syn::parse_macro_input;

///
///
/// TODO: rename attrs to ipld_macros(), add a `wrap = Box/Rc/etc` attr
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as SchemaDefinition)
        .expand()
        .into()
}

/// An attribute macro, used to help the `schema!` and `selector!` macros
/// capture relevant arguments.
#[proc_macro_attribute]
pub fn ipld_attr(_attr: TokenStream, items: TokenStream) -> TokenStream {
    items
}

///
#[proc_macro]
pub fn selector(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as RootSelectorDefinition)
        .expand()
        .into()
}

// derive API

///
#[proc_macro_derive(Representation, attributes(ipld))]
pub fn derive_representation(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as DeriveRepresentation)
        .expand()
        .into()
}

// #[proc_macro_derive(Select, attributes(ipld))]
// pub fn derive_select(input: TokenStream) -> TokenStream {
//     parse_macro_input!(input as DeriveSelect).expand().into()
// }
//
// #[proc_macro_derive(Patch, attributes(ipld))]
// pub fn derive_patch(input: TokenStream) -> TokenStream {
//     parse_macro_input!(input as DerivePatch).expand().into()
// }
//
// #[proc_macro_derive(Merge, attributes(ipld))]
// pub fn derive_merge(input: TokenStream) -> TokenStream {
//     parse_macro_input!(input as DeriveMerge).expand().into()
// }
