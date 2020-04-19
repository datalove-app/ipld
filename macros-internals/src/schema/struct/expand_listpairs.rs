use super::*;
use crate::dev::{
    schema::expand::{self, ExpandAdvancedRepresentation, ExpandBasicRepresentation},
    SchemaMeta,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Type};

impl ExpandBasicRepresentation for ListpairsStructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = meta.lib();
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields: Vec<TokenStream> = self
            .iter()
            .map(super::expand::default_field_typdef)
            .collect();

        quote! {
            #(#attrs)*
            #vis struct #ident {
                #(#fields)*
            }
        }
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        super::expand::impl_repr(self.iter(), meta)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}
