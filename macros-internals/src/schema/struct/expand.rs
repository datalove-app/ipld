use super::*;
use crate::dev::{
    schema::expand::{self, ExpandAdvancedRepresentation, ExpandBasicRepresentation},
    SchemaKind, SchemaMeta,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Type};

impl ExpandBasicRepresentation for StructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.define_type(meta),
            Self::Listpairs(repr) => repr.define_type(meta),
            Self::Tuple(repr) => repr.define_type(meta),
            Self::Stringpairs(repr) => repr.define_type(meta),
            Self::Stringjoin(repr) => repr.define_type(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }
    fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.derive_serde(meta),
            Self::Listpairs(repr) => repr.derive_serde(meta),
            Self::Tuple(repr) => repr.derive_serde(meta),
            Self::Stringpairs(repr) => repr.derive_serde(meta),
            Self::Stringjoin(repr) => repr.derive_serde(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        // let lib = &meta.ipld_schema_lib;
        // let repr_body = quote! {
        //     const KIND: Kind = Kind::Struct;
        // };
        // expand::impl_repr(meta, repr_body)
        let repr = match self {
            Self::Map(repr) => repr.derive_repr(meta),
            Self::Listpairs(repr) => repr.derive_repr(meta),
            Self::Tuple(repr) => repr.derive_repr(meta),
            Self::Stringpairs(repr) => repr.derive_repr(meta),
            Self::Stringjoin(repr) => repr.derive_repr(meta),
            Self::Advanced(_) => unreachable!(),
        };

        // TODO:
        quote! {
            #repr
            // impl_root_select!(#name => Matcher);
        }
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.derive_select(meta),
            Self::Listpairs(repr) => repr.derive_select(meta),
            Self::Tuple(repr) => repr.derive_select(meta),
            Self::Stringpairs(repr) => repr.derive_select(meta),
            Self::Stringjoin(repr) => repr.derive_select(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        match self {
            Self::Map(repr) => repr.derive_conv(meta),
            Self::Listpairs(repr) => repr.derive_conv(meta),
            Self::Tuple(repr) => repr.derive_conv(meta),
            Self::Stringpairs(repr) => repr.derive_conv(meta),
            Self::Stringjoin(repr) => repr.derive_conv(meta),
            Self::Advanced(_) => unreachable!(),
        }
    }
}

impl ExpandBasicRepresentation for BasicStructReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let generics = &meta
            .generics
            .as_ref()
            .map(|g| quote!(#g))
            .unwrap_or(TokenStream::default());
        let fields = self.iter().map(StructField::field_def);

        quote! {
            #(#attrs)*
            // #[derive(Deserialize, Serialize)]
            #vis struct #ident #generics {
                #(#fields,)*
            }
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        impl_repr(self, meta, &Self::dm_kind())
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        TokenStream::default()
    }
    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}

impl BasicStructReprDefinition {
    pub fn dm_kind() -> Ident {
        SchemaKind::Map.data_model_kind()
    }
}

pub(super) fn impl_repr<'a, D: Deref<Target = StructFields>>(
    fields: &D,
    meta: &SchemaMeta,
    repr_kind: &Ident,
) -> TokenStream {
    let lib = &meta.lib;
    let name = &meta.name;
    // let fields = iter.map(|f| {
    //     let key = f.key.to_string();
    //     let val = &f.value;
    //     let rename = f
    //         .rename
    //         .as_ref()
    //         .map(|s| s.value())
    //         .unwrap_or_else(|| key.clone());
    //     quote! {
    //         (#key, Field::new::<#val>(#rename))
    //     }
    // });

    let repr_body = expand::impl_repr(
        meta,
        quote! {
            type ReprKind = type_kinds::#repr_kind;

            const SCHEMA: &'static str = "";
            const DATA_MODEL_KIND: Kind = Kind::Map;
            const SCHEMA_KIND: Kind = Kind::Struct;
            // const REPR_KIND: Kind = Kind::#repr_kind;
            // const FIELDS: Fields = Fields::Struct(&[#(#fields,)*]);
        },
    );
    let selector_bodies = quote! {};
    repr_body
}

impl StructField {
    pub fn default_field_def(&self) -> TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let key = &self.key;
        let value = field_value(self);
        let generics = &self.generics;

        quote! {
            #(#attrs)*
            #vis #key: #value #generics
        }
    }

    fn field_def(&self) -> TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let key = &self.key;
        let value = field_value(self);
        let generics = &self.generics;

        let implicit_attr = self.implicit.as_ref().map(|_| quote!(#[serde(default)]));
        let rename_attr = self
            .rename
            .as_ref()
            .map(|name| quote!(#[serde(rename = #name)]));

        quote! {
            #(#attrs)*
            // #implicit_attr
            // #rename_attr
            #vis #key: #value #generics
        }
    }
}

pub(super) fn field_value(field: &StructField) -> TokenStream {
    let value = &field.value;
    if field.optional || field.nullable {
        quote!(Option<#value>)
    } else {
        quote!(#value)
    }
}

impl ExpandAdvancedRepresentation for StructReprDefinition {
    fn expand_struct(repr: AdvancedStructReprDefinition) -> TokenStream {
        unimplemented!()
    }
}
