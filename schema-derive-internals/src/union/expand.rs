use super::*;
use crate::{ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for UnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let lib = &meta.ipld_schema;

        // match self {
        //     Self::Keyed => {

        //     }
        //     Self::Envelope => {}
        //     Self::Inline => {}
        //     Self::BytePrefix => {}
        //     Self::Kinded => {}
        // }

        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}
