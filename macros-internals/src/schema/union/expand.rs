use super::*;
use crate::dev::{
    common,
    schema::expand::{self, ExpandBasicRepresentation},
    SchemaMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Ident, Type};

impl ExpandBasicRepresentation for KeyedUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = meta.lib();
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields: Vec<TokenStream> = self.iter().map(field_typedef).collect();

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::Deserialize, #lib::dev::Serialize)]
            #vis enum #ident {
                #(#fields,)*
            }
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                // const KIND: _ipld::dev::Kind =
                //     _ipld::dev::Kind::Union;
                // const FIELDS: _ipld::dev::Fields = _ipld::dev::Fields::Keyed(&[#(#fields,)*]);
            },
        )
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        // let lib = &meta.ipld_schema_lib;
        quote!(impl_root_select!(#name => Matcher);)
    }
}

fn field_typedef(field: &UnionField<LitStr>) -> TokenStream {
    let attrs = &field.attrs;
    let value = &field.value;
    let key = &field.key;

    let value_tokens: TokenStream = if let Some(wrapper_type) = &field.wrapper {
        quote!(#wrapper_type<#value>)
    } else {
        quote!(#value)
    };

    let rename_attr = quote!(#[serde(rename = #key)]);
    // TODO: if the field is a link type, rename the field to avoid ambiguity,
    // TODO? but preserve the listed name?
    let field_name: Ident = if field.linked {
        Ident::new(&format!("{:?}Link", value), Span::call_site())
    } else {
        field.value.clone()
    };

    quote! {
        #(#attrs)*
        #rename_attr
        #field_name(#value_tokens)
    }
}

impl ExpandBasicRepresentation for UnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.define_type(meta),
            // Self::Envelope(def) => def.define_type(meta),
            // Self::Inline(def) => def.define_type(meta),
            // Self::BytePrefix(def) => def.define_type(meta),
            // Self::Kinded(def) => def.define_type(meta),
            _ => unimplemented!(),
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.derive_repr(meta),
            // Self::Envelope(def) => def.derive_repr(meta),
            // Self::Inline(def) => def.derive_repr(meta),
            // Self::BytePrefix(def) => def.derive_repr(meta),
            // Self::Kinded(def) => def.derive_repr(meta),
            _ => unimplemented!(),
        }
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.derive_selector(meta),
            // Self::Envelope(def) => def.derive_selector(meta),
            // Self::Inline(def) => def.derive_selector(meta),
            // Self::BytePrefix(def) => def.derive_selector(meta),
            // Self::Kinded(def) => def.derive_selector(meta),
            _ => unimplemented!(),
        }
    }
}
