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

impl Ipld<CODE> {
    fn validate(&self) -> ParseResult<()> {
        Ok(())
    }

    pub(crate) fn derive(&self) -> TokenStream {
        Default::default()
    }

    pub(crate) fn expand_null(&self) -> TokenStream {
        Default::default()
    }

    pub(crate) fn expand_newtype(&self) -> TokenStream {
        Default::default()
    }
}
