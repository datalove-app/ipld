extern crate proc_macro;

use ipld_macros_internals::{
    // def_attributes,
    RootSelectorDefinition,
    // ValueDefinition,
    SchemaDefinition,
};
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use syn::parse_macro_input;

///
///
/// TODO: rename attrs to ipld_macros(), add a `wrap = Box/Rc/etc` attr
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    let schema = parse_macro_input!(input as SchemaDefinition);
    TokenStream::from(schema.expand())
}

#[proc_macro_attribute]
pub fn ipld_attr(_attr: TokenStream, items: TokenStream) -> TokenStream {
    items
}

///
/// TODO? possibly parse typedefs from strings in functions?
#[proc_macro_hack]
pub fn selector(input: TokenStream) -> TokenStream {
    let selector = parse_macro_input!(input as RootSelectorDefinition);
    TokenStream::from(selector.expand())
}

// ///
// #[proc_macro]
// pub fn value(input: TokenStream) -> TokenStream {
//     let value = parse_macro_input!(input as ValueDefinition);
//     TokenStream::from(value.expand())
// }

// exports attribute proc macros used with `schema!`.
// def_attributes!(
//     #[doc(hidden)]
//     ipld_macros_internal
// );
// def_attributes!(try_from);
// def_attributes!(wrap);
