use super::{
    BoolReprDefinition, CopyReprDefinition, FloatReprDefinition, IntReprDefinition,
    StringReprDefinition,
};
use crate::{ExpandBasicRepresentation, SchemaMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Defines a newtype wrapper around an inner type already implementing `Representation`.
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! define_newtype {
    ($meta:ident, $type:ident) => {{
        let attrs = &$meta.attrs;
        let vis = &$meta.vis;
        let ident = &$meta.name;
        let lib = &$meta.ipld_schema;
        quote::quote! {
            #attrs
            #[derive(#lib::dev::serde::Deserialize, #lib::dev::serde::Serialize)]
            #[serde(transparent)]
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
        let lib = &$meta.ipld_schema;
        let repr_body = quote! {
            // TODO: impl the rest
            const KIND: #lib::SchemaKind = <#$type as #lib::dev::Representation<Ctx>>::KIND;
        };
        $self.impl_repr($meta, repr_body)
    }};
}

// macro_rules! expand_def {
//     ($def:ident, $inner_type:ident) => {
//         impl ExpandBasicRepresentation for $def {
//             fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
//                 define_newtype!(meta, $inner_type)
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
        define_newtype!(meta, inner_type)
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
        define_newtype!(meta, inner_type)
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
        define_newtype!(meta, inner_type)
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
        let inner_type = Type::Verbatim(quote!(String));
        define_newtype!(meta, inner_type)
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
        let inner_type = &self.0;
        define_newtype!(meta, inner_type)
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let inner_type = &self.0;
        derive_newtype_repr!(self, meta => inner_type)
    }
    fn derive_selector(&self, meta: &SchemaMeta) -> TokenStream {
        unimplemented!()
    }
}
