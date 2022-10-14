use super::*;
use crate::{
    dev::{
        schema::expand::{self, ExpandAdvancedRepresentation, ExpandBasicRepresentation},
        SchemaMeta,
    },
    schema::SchemaKind,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Type};

impl ExpandBasicRepresentation for StringjoinStructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields = self.iter().map(StructField::default_field_def);

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
            #vis struct #ident {
                #(#fields)*
            }
        }
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        super::expand::impl_repr(
            self,
            meta,
            &SchemaKind::String.data_model_kind(),
            Some(Ident::new("StringJoin", Span::call_site())),
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}
