//!
//! Representation/Node/Select/Patch/Merge edge-ish cases:
//! - unit struct       ==> Null
//! - newtypes          ==> delegate to inner (copy)
//! - structs           ==> structs
//! - enums w/ discrims ==> int/str enum
//! - enums             ==> unions

mod representation;
mod select;

pub use representation::{DeriveRepresentation, ReprKind, SchemaKind};

use crate::dev::*;
use darling::{
    ast::{self, Data as Body, GenericParam, Style},
    util::{self, SpannedValue},
    Error, FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Add,
    Attribute, Expr, Field, Generics, Ident, LitStr, Type, TypeParamBound, Variant, Visibility,
};

macro_rules! proc_macro_compat {
    ($t:ident) => {
        impl Parse for $t {
            // todo: validation
            fn parse(input: ParseStream) -> ParseResult<Self> {
                // println!("{}", &input);
                let input = syn::DeriveInput::parse(input)?;
                // // println!("{:?}", &input);
                Ok(Self::from_derive_input(&input)?)
            }
        }

        impl ToTokens for $t {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let imports = self.imports();
                let scope = self.scope();
                let derive_impl = self.derive();
                tokens.append_all(quote! {
                    // #typedef
                    const #scope: () = {
                        #imports

                        #[automatically_derived]
                        #derive_impl
                    };
                });
            }
        }
    };
}

proc_macro_compat!(DeriveRepresentation);
// proc_macro_compat!({ 'S' as u64 });

// #[derive(Clone, Debug, FromDeriveInput)]
// #[darling(
//     attributes(ipld),
//     // forward_attrs(cfg),
//     // supports(any)
// )]
// pub enum IpldDeriveInput {
//     Null,
//     Newtype,
//     // TupleStruct,
//     Struct,
//     Enum
// }

#[derive(Clone, Debug, FromDeriveInput)]
#[darling(attributes(ipld), forward_attrs(cfg, repr), supports(any))]
pub struct Ipld<const T: u64> {
    vis: Visibility,
    ident: Ident,
    // generics: ast::Generics<GenericParam>,
    generics: Generics,
    data: Body<IpldVariant, IpldField>,
    attrs: Vec<Attribute>,

    // attrs
    #[darling(default)]
    internal: bool,
    // TODO: figure out how to handle this
    #[darling(default, rename = "repr")]
    repr_kind: SpannedValue<ReprKind>,
    #[darling(default, rename = "where_ctx")]
    where_ctx: Option<Punctuated<TypeParamBound, Add>>,
    #[darling(default, rename = "merge")]
    merge: Option<Ident>,
    // include: Option<Vec<>>,
}

impl<const T: u64> Ipld<T> {
    ///
    pub fn expand(self) -> TokenStream
    where
        Self: ToTokens,
    {
        self.into_token_stream()
    }

    fn discriminant(&self) -> Option<&Expr> {
        self.data
            .as_ref()
            .take_enum()
            .and_then(|v| v[0].discriminant.as_ref())
    }

    fn fields(&self) -> impl Iterator<Item = &IpldField> {
        self.data.as_ref().take_struct().unwrap().into_iter()
    }

    fn schema_kind(&self) -> Result<SchemaKind, Error> {
        use ReprKind::*;
        use SchemaKind::*;

        let body = &self.data;
        let repr_kind = *self.repr_kind.as_ref();
        match body {
            Body::Struct(fields) => match (fields.style, repr_kind) {
                // unit == null
                (Style::Unit, Default) => Ok(Null),
                // copy == newtype
                (Style::Tuple, Default | Transparent) if fields.is_newtype() => Ok(Copy),
                // tuple def or repr == tuple struct
                (Style::Tuple, Default | Tuple) => Ok(Struct(Tuple)),
                // struct == repr
                (Style::Struct, Default | Tuple | Stringpairs | Stringjoin | Listpairs) => {
                    Ok(Struct(repr_kind))
                }
                _ => Err(Error::unsupported_shape(
                    "invalid null, transparent/newtype (copy), or struct schema definition/representation",
                )),
            },
            Body::Enum(variants) => match (variants, repr_kind) {
                // enum
                (v, Default) if v.iter().all(|v| v.discriminant.is_some()) => Ok(Enum),
                // union
                (_, Default) => Ok(Union(Keyed)),
                (_, Keyed | Kinded | Envelope | Inline | Bytesprefix | Stringprefix) => {
                    Ok(Union(repr_kind))
                }
                _ => Err(Error::unsupported_shape(
                    "invalid enum or union schema definition/representation",
                )),
            },
        }
    }
}

impl<const T: u64> Ipld<T> {
    fn imports(&self) -> TokenStream {
        SchemaMeta::imports(self.internal)
    }

    fn scope(&self) -> Ident {
        Ident::new(&format!("_{}_FOR_{}", T, &self.ident), Span::call_site())
    }
}

///
#[derive(Clone, Debug, FromField)]
#[darling(attributes(ipld))]
pub struct IpldField {
    vis: Visibility,
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<Attribute>,

    // attrs
    #[darling(default)]
    wrapper: Option<Type>,
    #[darling(default)]
    skip: bool,
    // #[darling(default)]
    // nullable: bool,
    // #[darling(default)]
    // optional: bool,
    // #[darling(default)]
    // implicit: bool,
    // #[darling(default)]
    // linked: bool,
    #[darling(default)]
    rename: Option<LitStr>,
}

#[derive(Clone, Debug, FromVariant)]
#[darling(attributes(ipld))]
pub struct IpldVariant {
    ident: Ident,
    discriminant: Option<Expr>,
    fields: ast::Fields<IpldField>,
    attrs: Vec<Attribute>,

    // attrs
    #[darling(default)]
    wrapper: Option<Type>,
}

#[derive(Clone, Debug, FromMeta)]
#[darling(and_then = "Self::validate")]
pub struct IpldFieldCardinality {
    // #[darling(default)]
    // nullable: bool,
    // #[darling(default)]
    // optional: bool,
    // #[darling(default)]
    // implicit: bool,
}

impl IpldFieldCardinality {
    fn validate(self) -> Result<Self, Error> {
        Ok(self)
    }
}
