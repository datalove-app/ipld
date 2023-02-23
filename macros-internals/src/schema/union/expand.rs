use super::*;
use crate::dev::{
    common,
    schema::expand::{self, ExpandBasicRepresentation},
    SchemaMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Ident, Type};

impl ExpandBasicRepresentation for UnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.define_type(meta),
            // Self::Envelope(def) => def.define_type(meta),
            // Self::Inline(def) => def.define_type(meta),
            // Self::BytePrefix(def) => def.define_type(meta),
            Self::Kinded(def) => def.define_type(meta),
            _ => unimplemented!(),
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.derive_repr(meta),
            // Self::Envelope(def) => def.derive_repr(meta),
            // Self::Inline(def) => def.derive_repr(meta),
            // Self::BytePrefix(def) => def.derive_repr(meta),
            Self::Kinded(def) => def.derive_repr(meta),
            _ => unimplemented!(),
        }
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.derive_select(meta),
            // Self::Envelope(def) => def.derive_select(meta),
            // Self::Inline(def) => def.derive_select(meta),
            // Self::BytePrefix(def) => def.derive_select(meta),
            Self::Kinded(def) => def.derive_select(meta),
            _ => unimplemented!(),
        }
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Keyed(def) => def.derive_conv(meta),
            // Self::Envelope(def) => def.derive_conv(meta),
            // Self::Inline(def) => def.derive_conv(meta),
            // Self::BytePrefix(def) => def.derive_conv(meta),
            Self::Kinded(def) => def.derive_conv(meta),
            _ => unimplemented!(),
        }
    }
}

impl ExpandBasicRepresentation for KeyedUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields = self.iter().map(UnionStrField::field_typedef);

        quote! {
            #(#attrs)*
            #vis enum #ident {
                #(#fields,)*
            }
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name_branches = self.iter().map(UnionStrField::name_branch);
        let kind_branches = self.iter().map(UnionStrField::kind_branch);
        let link_branches = self.iter().map(UnionStrField::link_branch);
        let serialize_branches = self.iter().map(UnionStrField::serialize_branch);
        // let deserialize_branches = self.iter().map(UnionStrField::deserialize_branch);
        self.impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = Kind::Map;
                const SCHEMA_KIND: Kind = Kind::Union;
                const REPR_KIND: Kind = Kind::Map;
                const REPR_STRATEGY: Strategy = Strategy::Keyed;
                // TODO
                // const FIELDS: Fields = Fields::Keyed(&[#(#fields,)*]);
            },
            quote! {
                #[inline]
                fn name(&self) -> &'static str {
                    match self {
                        #(#name_branches,)*
                    }
                }
                #[inline]
                fn has_links(&self) -> bool {
                    match self {
                        #(#link_branches,)*
                    }
                }
                #[inline]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    // match self {
                    //     #(#serialize_branches,)*
                    // }
                    unimplemented!()
                }
                #[inline]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    unimplemented!()
                }
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        // self.impl_select(meta, None)
        Default::default()
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        quote! {
            #[automatically_derived]
            impl From<#name> for SelectedNode {
                fn from(t: #name) -> Self {
                    // t.0.into()
                    unimplemented!()
                }
            }
            #[automatically_derived]
            impl Into<Any> for #name {
                fn into(self) -> Any {
                    // self.0.into()
                    unimplemented!()
                }
            }
            #[automatically_derived]
            impl TryFrom<Any> for #name {
                type Error = Error;
                fn try_from(any: Any) -> Result<Self, Self::Error> {
                    // let variant = Representation::name(&any);
                    // let inner = TryFrom::try_from(any)
                    //     .map_err(|_| Error::failed_any_conversion::<Self>(variant))?;
                    // Ok(Self(inner))
                    unimplemented!()
                }
            }
        }
    }
}

impl UnionField<LitStr> {
    // TODO: if the field is a link type, rename the field to avoid ambiguity,
    // TODO? but preserve the listed name?
    fn field_name(&self) -> Ident {
        if self.linked {
            Ident::new(&format!("{:?}Link", &self.value), Span::call_site())
        } else {
            self.value.clone()
        }
    }

    fn field_typedef(&self) -> TokenStream {
        let attrs = &self.attrs;
        let value = &self.value;
        let key = &self.key;
        let generics = &self.generics;

        let field_name: Ident = self.field_name();

        let ty: TokenStream = if let Some(wrapper_type) = &self.wrapper {
            quote!(#wrapper_type<#value #generics>)
        } else {
            quote!(#value #generics)
        };

        quote! {
            #(#attrs)*
            // #[serde(rename = #key)]
            #field_name(#ty)
        }
    }

    fn name_branch(&self) -> TokenStream {
        let field_name = self.field_name();
        quote!(Self::#field_name(inner) => Representation::name(inner))
    }

    fn kind_branch(&self) -> TokenStream {
        let field_name = self.field_name();
        quote!(Self::#field_name(inner) => Representation::kind(inner))
    }

    fn link_branch(&self) -> TokenStream {
        let field_name = self.field_name();
        quote!(Self::#field_name(inner) => Representation::has_links(inner))
    }

    fn serialize_branch(&self) -> TokenStream {
        let field_name = self.field_name();
        let repr_field_name = &self.key;
        // let ty = &self.value;
        quote!(Self::#field_name(ty) => {
            use serde::ser::SerializeTupleVariant;

            let mut tv = serializer.serialize_tuple_variant("", 0, #repr_field_name, 1)?;
            tv.serialize_field(&SerializeWrapper(&ty))?;
            tv.end()
        })
    }
}
