use super::{representation::ReprKind, Ipld};
use crate::dev::*;
use darling::{
    ast::{self, Data as Body, GenericParam},
    util, Error, FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    punctuated::Punctuated,
    token::Add,
    Attribute, Expr, Field, Generics, Ident, LitStr, Type, TypeParamBound, Variant, Visibility,
};

pub const CODE: u64 = 'S' as u64;

pub type DeriveSelect = Ipld<CODE>;

impl DeriveSelect {
    fn validate(&self) -> ParseResult<()> {
        Ok(())
    }

    pub(crate) fn derive(&self) -> TokenStream {
        let name = &self.ident;

        let select = {
            let body = self.body();
            let (impl_generics, ty_generics, where_clause) = {
                let (_, ty_generics, where_clause) = self.generics.split_for_impl();
                let impl_generics = quote! {
                    <__Ctx: Context>
                };
                (impl_generics, ty_generics, where_clause)
            };
            quote! {
                impl #impl_generics Select<__Ctx> for #name #ty_generics
                #where_clause {
                    type Walker<'__a, const MC: u64> = AstWalk<'__a, MC, __Ctx, Self>;
                    #body
                }
            }
        };

        let visitor: TokenStream = {
            match self.schema_kind().unwrap() {
                // SchemaKind::Null => self.expand_null_visitor(),
                // SchemaKind::Struct(ref repr) => unimplemented!(),
                // SchemaKind::Enum => self.expand_enum(),
                // SchemaKind::Union(ref repr) => self.expand_union(repr),
                // SchemaKind::Copy => self.expand_newtype(),
                _ => Default::default(),
            }
        };

        quote! {
            #visitor
            #select
        }
    }

    pub(crate) fn expand_null(&self) -> TokenStream {
        quote! {}
    }

    pub(crate) fn expand_null_visitor(&self) -> TokenStream {
        quote! {}
    }

    pub(crate) fn expand_newtype(&self) -> TokenStream {
        Default::default()
    }

    pub(crate) fn expand_struct(&self, repr: &ReprKind) -> TokenStream {
        Default::default()
    }

    pub(crate) fn expand_enum(&self) -> TokenStream {
        Default::default()
    }

    pub(crate) fn expand_union(&self, repr: &ReprKind) -> TokenStream {
        let name = &self.ident;

        quote! {}
    }
}
