use super::{
    BoolReprDefinition, CopyReprDefinition, FloatReprDefinition, IntReprDefinition,
    StringReprDefinition,
};
use crate::dev::{schema::expand, SchemaMeta};
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
        let lib = $meta.lib();
        let attrs = &$meta.attrs;
        let vis = &$meta.vis;
        let ident = &$meta.name;

        // let (try_from_typedef, try_from_serde_attr) = if let Some(try_from_name) = &$meta.try_from {
        //     let ident = &$meta.try_from_name();
        //     (
        //         // creates an inner type that transparently (de)serializes itself
        //         quote::quote! {
        //             #[derive(_ipld::dev::Deserialize, _ipld::dev::Serialize)]
        //             #[serde(transparent)]
        //             struct #ident(#$type);
        //         },
        //         // tells serde to delegate to a user-defined TryFrom impl
        //         quote::quote!(#[serde(try_from = #try_from_name)]),
        //     )
        // } else {
        //     (TokenStream::default(), TokenStream::default())
        // };

        quote::quote! {
            // #try_from_typedef

            #(#attrs)*
            #[derive(#lib::dev::Deserialize, #lib::dev::Serialize)]
            #[serde(transparent)]
            // #try_from_serde_attr
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
        expand::impl_repr(
            $meta,
            quote! {
                // const KIND: #lib::dev::Kind = <#$type as #lib::dev::Representation>::KIND;
            },
        )
    }};
}

// macro_rules! expand_def {
//     ($def:ident, $inner_type:ident) => {
//         impl expand::ExpandBasicRepresentation for $def {
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

impl expand::ExpandBasicRepresentation for BoolReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = Type::Verbatim(quote!(bool));
        define_newtype!(self, meta => inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = Type::Verbatim(quote!(bool));
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}

impl expand::ExpandBasicRepresentation for IntReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        define_newtype!(self, meta => inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        derive_newtype_repr!(self, meta => inner_type)
    }
    // TODO:
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

impl expand::ExpandBasicRepresentation for FloatReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        define_newtype!(self, meta => inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        derive_newtype_repr!(self, meta => inner_type)
    }
    // TODO:
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
}

impl expand::ExpandBasicRepresentation for StringReprDefinition {
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

impl expand::ExpandBasicRepresentation for CopyReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
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
