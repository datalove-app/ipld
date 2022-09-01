use super::*;
use crate::dev::{
    common,
    schema::expand::{self, ExpandBasicRepresentation},
    SchemaMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Ident, Type};

impl ExpandBasicRepresentation for KeyedUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields = self.iter().map(UnionField::<LitStr>::field_typedef);

        quote! {
            #(#attrs)*
            #[derive(Deserialize, Serialize)]
            #vis enum #ident {
                #(#fields,)*
            }
        }
    }
    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let name_branches = self.iter().map(UnionField::<LitStr>::name_branch);
        let kind_branches = self.iter().map(UnionField::<LitStr>::kind_branch);
        let link_branches = self.iter().map(UnionField::<LitStr>::link_branch);
        expand::impl_repr(
            meta,
            quote! {
                const KIND: Kind = Kind::Union;
                // const FIELDS: Fields = Fields::Keyed(&[#(#fields,)*]);

                #[inline]
                fn name(&self) -> &'static str {
                    match self {
                        #(#name_branches,)*
                    }
                }

                #[inline]
                fn kind(&self) -> Kind {
                    match self {
                        #(#kind_branches,)*
                    }
                }

                #[inline]
                fn has_links(&self) -> bool {
                    match self {
                        #(#link_branches,)*
                    }
                }
            },
        )
    }
    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        // let name = &meta.name;
        // let lib = &meta.ipld_schema_lib;
        // quote!(impl_root_select!(#name => Matcher);)
        TokenStream::default()
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

        let rename_attr = quote!(#[serde(rename = #key)]);
        let field_name: Ident = self.field_name();

        let ty: TokenStream = if let Some(wrapper_type) = &self.wrapper {
            quote!(#wrapper_type<#value #generics>)
        } else {
            quote!(#value #generics)
        };

        quote! {
            #(#attrs)*
            #rename_attr
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
}

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
            // Self::Envelope(def) => def.derive_selects(meta),
            // Self::Inline(def) => def.derive_selects(meta),
            // Self::BytePrefix(def) => def.derive_selects(meta),
            Self::Kinded(def) => def.derive_select(meta),
            _ => unimplemented!(),
        }
    }
}
