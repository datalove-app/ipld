use super::{BytesReprDefinition, LinkReprDefinition, NullReprDefinition};
use crate::{
    define_newtype,
    dev::{schema::expand, SchemaMeta},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Type};

impl expand::ExpandBasicRepresentation for NullReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;

        quote! {
            #(#attrs)*
            #[derive(#lib::dev::Deserialize, #lib::dev::Serialize)]
            #vis struct #ident;
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const KIND: _ipld::dev::Kind = _ipld::dev::Kind::Null;
            },
        )
    }
}

impl expand::ExpandBasicRepresentation for BytesReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;

        quote! {
            #(#attrs)*
            #vis struct #ident(bytes::Bytes);

            #[automatically_derived]
            impl bytes::buf::Buf for #ident {
                fn remaining(&self) -> usize {
                    self.0.remaining()
                }
                fn bytes(&self) -> &[u8] {
                    self.0.bytes()
                }
                fn advance(&mut self, cnt: usize) {
                    self.0.advance(cnt)
                }
            }
        }
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;

        // TODO correctness?
        let impl_ser = expand::impl_serialize(
            meta,
            quote! {
                use bytes::buf::Buf;
                <S as Encoder>::serialize_bytes(serializer, self.bytes())
            },
        );
        let (visitor, impl_visitor) = expand::impl_visitor(
            meta,
            "expected an IPLD bytes value",
            quote! {
                fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E>{
                    Ok(#name(bytes::Bytes::copy_from_slice(v)))
                }
                fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                    Ok(#name(bytes::Bytes::from(v)))
                }
            },
        );
        let impl_visitor_ext = expand::impl_visitor_ext(meta, None);
        let impl_de = expand::impl_deserialize(
            meta,
            quote! {
                <D as Decoder>::deserialize_bytes(deserializer, #visitor)
            },
        );

        quote! {
            #impl_ser
            #impl_visitor
            #impl_visitor_ext
            #impl_de
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                const KIND: _ipld::dev::Kind = _ipld::dev::Kind::Bytes;
            },
        )
    }
    // TODO: add support for explore range
    fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
        // let name = &meta.name;
        // quote!(impl_root_select!(#name => Matcher);)
        TokenStream::default()
    }
}

impl expand::ExpandBasicRepresentation for LinkReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner = &self.0;
        let inner_type = Type::Verbatim(quote!(Link<#inner>));
        define_newtype!(self, meta => inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner = &self.0;
        let inner_type = Type::Verbatim(quote!(Link<#inner>));

        // TODO:
        let repr_def = quote! {};
        let wrapper_repr_def = quote! {};

        quote! {
            #repr_def
            #wrapper_repr_def
        }
    }
    fn derive_selects(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}
