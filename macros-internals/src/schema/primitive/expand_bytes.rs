use super::BytesReprDefinition;
use crate::{
    define_newtype,
    dev::{
        schema::{expand, SchemaKind},
        SchemaMeta,
    },
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

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
                fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E>{
                    Ok(#name(bytes::Bytes::copy_from_slice(v)))
                }
                fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
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
        let lib = &meta.lib;
        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: #lib::dev::Kind = #lib::dev::Kind::Bytes;
            },
        )
    }
    // TODO: add support for explore range
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        // let name = &meta.name;
        // quote!(impl_root_select!(#name => Matcher);)
        TokenStream::default()
    }

    // fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
    //     let lib = &meta.lib;
    //     let ident = &meta.name;
    //
    //     quote! {
    //         impl Into<#lib::dev::Node> for #ident {
    //             fn into(self) -> #lib::dev::Node {
    //                 Node::Bytes(self.into())
    //             }
    //         }
    //
    //         impl Into<#lib::dev::Value> for #ident {
    //             fn into(self) -> #lib::dev::Value {
    //                 Value::Bytes(self.into())
    //             }
    //         }
    //     }
    // }
}
