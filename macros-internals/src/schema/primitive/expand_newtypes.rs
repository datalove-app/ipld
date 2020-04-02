use super::{
    BoolReprDefinition, CopyReprDefinition, FloatReprDefinition, IntReprDefinition,
    StringReprDefinition,
};
use crate::dev::{schema::expand::ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Defines a newtype wrapper around an inner type already implementing
/// `Serialize`, `Deserialize`, and `Representation`.
/// TODO: manually/macro implement serialize/deserialize for these types
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! define_newtype {
    ($self:ident, $meta:ident => $type:ident) => {{
        let attrs = &$meta.attrs;
        let vis = &$meta.vis;
        let ident = &$meta.name;
        let lib = &$meta.ipld_schema_lib;

        let (try_from, serde_try_from) = if let Some(try_from_name) = &$meta.try_from {
            let ident = &$meta.try_from_name();
            (
                quote::quote! {
                    #attrs
                    #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
                    #[serde(transparent)]
                    struct #ident(#$type);
                },
                quote::quote!(#[serde(try_from = #try_from_name)]),
            )
        } else {
            (TokenStream::default(), TokenStream::default())
        };

        quote::quote! {
            #try_from

            #attrs
            #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
            #[serde(transparent)]
            #serde_try_from
            #vis struct #ident(#$type);

            impl ::std::ops::Deref for #ident {
                type Target = #$type;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    }};
}

/// Derives `Representation` for this type, delegating to the inner type.
macro_rules! derive_newtype_repr {
    ($self:ident, $meta:ident => $type:ident) => {{
        let lib = &$meta.ipld_schema_lib;

        let repr_body = quote! {
            // TODO: impl the rest
            const KIND: #lib::dev::SchemaKind = <#$type as #lib::dev::Representation<Ctx>>::KIND;
        };
        $self.impl_repr($meta, repr_body)
    }};
}

// macro_rules! expand_def {
//     ($def:ident, $inner_type:ident) => {
//         impl ExpandBasicRepresentation for $def {
//             fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
//                 define_newtype!(self, meta => $inner_type)
//             }
//             fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
//                 derive_newtype_repr!(self, meta => $inner_type)
//             }
//             fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
//                 unimplemented!()
//             }
//         }
//     };
// }

impl ExpandBasicRepresentation for BoolReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = Type::Verbatim(quote!(bool));
        define_newtype!(self, meta => inner_type)
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = Type::Verbatim(quote!(bool));
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}

impl ExpandBasicRepresentation for IntReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        define_newtype!(self, meta => inner_type)
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}

impl ExpandBasicRepresentation for FloatReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        define_newtype!(self, meta => inner_type)
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}

impl ExpandBasicRepresentation for StringReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = Type::Verbatim(quote!(String));
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}

impl ExpandBasicRepresentation for CopyReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}
