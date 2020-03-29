use super::{LinkReprDefinition, NullReprDefinition};
use crate::{define_newtype, ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

impl ExpandBasicRepresentation for NullReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        quote! {
            #attrs
            #vis struct #ident;
        }
    }

    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let lib = &meta.ipld_schema;

        let ser_body = quote!(serializer.serialize_unit());
        let impl_ser = self.impl_serialize(meta, ser_body);

        let visitor_body = quote! {
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: #lib::dev::serde::de::Error,
            {
                Ok(#name)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: #lib::dev::serde::de::Error,
            {
                Ok(#name)
            }
        };
        let (visitor, impl_visitor) =
            self.impl_visitor(meta, "expected an IPLD null value", visitor_body);

        let de_body = quote!(deserializer.deserialize_unit(#visitor));
        let impl_de = self.impl_deserialize(meta, de_body);

        quote! {
            #impl_ser
            #impl_visitor
            #impl_de
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.ipld_schema;
        let repr_body = quote! {
            // TODO: impl the rest
            const KIND: #lib::SchemaKind = #lib::SchemaKind::Null;
        };
        self.impl_repr(meta, repr_body)
    }

    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}

impl ExpandBasicRepresentation for LinkReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner = &self.0;
        let inner_type = Type::Verbatim(quote!(Link<#inner>));
        define_newtype!(meta, inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner = &self.0;
        let inner_type = Type::Verbatim(quote!(Link<#inner>));

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
