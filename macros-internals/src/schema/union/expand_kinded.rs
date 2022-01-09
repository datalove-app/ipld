use super::*;
use crate::dev::{
    schema::{
        expand::{self, ExpandBasicRepresentation},
        kw,
    },
    SchemaMeta,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for KindedUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields: Vec<TokenStream> = self.iter().map(field_typedef).collect();

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
            #[serde(untagged)]
            #vis enum #ident {
                #(#fields,)*
            }
        }
    }

    // fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
    //     TokenStream::default()
    // }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let name_branches: Vec<TokenStream> = self
            .iter()
            .map(|field| {
                let field_name = field.key.field_name();
                let ty = &field.value;
                let name = if let Some(generics) = &field.generics {
                    let (_, ty_generics, _) = generics.split_for_impl();
                    let turbofish = ty_generics.as_turbofish();
                    quote!(#ty#turbofish::NAME)
                } else {
                    quote!(#ty::NAME)
                };

                quote!(Self::#field_name(_) => #name)
            })
            .collect();
        let kind_branches: Vec<TokenStream> = self
            .iter()
            .map(|field| {
                let field_name = field.key.field_name();
                let kind = field.key.kind(&lib);
                quote!(Self::#field_name(_) => #kind)
            })
            .collect();

        expand::impl_repr(
            meta,
            quote! {
                const KIND: #lib::dev::Kind =
                    #lib::dev::Kind::Union;
                // const FIELDS: _ipld::dev::Fields = _ipld::dev::Fields::Keyed(&[#(#fields,)*]);

                #[inline]
                fn name(&self) -> &'static str {
                    match self {
                        #(#name_branches,)*
                    }
                }

                #[inline]
                fn kind(&self) -> _ipld::dev::Kind {
                    match self {
                        #(#kind_branches,)*
                    }
                }
            },
        )
    }

    fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

fn field_typedef(field: &UnionField<DataModelKind>) -> TokenStream {
    let attrs = &field.attrs;
    let value = &field.value;
    let key = &field.key.field_name();
    let generics = &field.generics;

    // let implicit_attr = if let Some(implicit) = &field.implicit {
    //     quote!(#[serde(default)])
    // } else {
    //     TokenStream::default()
    // };

    // let rename_attr = if let Some(rename) = &field.rename {
    //     quote!(#[serde(rename = #rename)])
    // } else {
    //     TokenStream::default()
    // };

    quote! {
        #(#attrs)*
        #key(#value #generics)
    }
}

impl DataModelKind {
    fn field_name(&self) -> TokenStream {
        match self {
            Self::Null => quote!(null),
            Self::Boolean => quote!(boolean),
            Self::Integer => quote!(int),
            Self::Float => quote!(float),
            Self::Bytes => quote!(bytes),
            Self::String => quote!(string),
            Self::List => quote!(list),
            Self::Map => quote!(map),
            Self::Link => quote!(link),
        }
    }

    fn kind(&self, lib: &TokenStream) -> TokenStream {
        match self {
            Self::Null => quote! { #lib::dev::Kind::Null },
            Self::Boolean => quote! { #lib::dev::Kind::Boolean },
            Self::Integer => quote! { #lib::dev::Kind::Integer },
            Self::Float => quote! { #lib::dev::Kind::Float },
            Self::Bytes => quote! { #lib::dev::Kind::Bytes },
            Self::String => quote! { #lib::dev::Kind::String },
            Self::List => quote! { #lib::dev::Kind::List },
            Self::Map => quote! { #lib::dev::Kind::Map },
            Self::Link => quote! { #lib::dev::Kind::Link },
        }
    }
}
