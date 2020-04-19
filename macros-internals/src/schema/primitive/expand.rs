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
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;

        quote! {
            #(#attrs)*
            #vis struct #ident;
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        expand::impl_repr(
            meta,
            quote! {
                // const KIND: #lib::dev::Kind = #lib::dev::Kind::Null;
            },
        )
    }

    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        // let lib = &meta.ipld_schema_lib;

        // TODO? correctness?
        let impl_ser = expand::impl_serialize(
            meta,
            quote! {
            serializer.serialize_unit()},
        );
        let (visitor, impl_visitor) = expand::impl_visitor(
            meta,
            "expected an IPLD null value",
            quote! {
                fn visit_none<E: de::Error>(self) -> Result<Self::Value, E>{
                    Ok(#name)
                }

                fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                    Ok(#name)
                }
            },
        );
        // TODO: is this right?
        let impl_de = expand::impl_deserialize(
            meta,
            quote! {
            deserializer.deserialize_unit(#visitor)},
        );

        quote! {
            #impl_ser
            #impl_visitor
            #impl_de
        }
    }

    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        // let impl_de_seed = expand::impl_primtive_de_seed(meta);
        quote! {
            impl_root_select!(#name => Matcher);
            // #impl_de_seed
        }
    }
}

impl expand::ExpandBasicRepresentation for BytesReprDefinition {
    // TODO:
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = meta.lib();
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
                // const KIND: #lib::dev::Kind = #lib::dev::Kind::Bytes;
            },
        )
    }
    // TODO:
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
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
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}
