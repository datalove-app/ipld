mod expand;
mod expand_listpairs;
mod expand_stringjoin;
mod expand_stringpairs;
mod expand_tuple;
mod parse;

use crate::dev::{InnerAttributes, SchemaMeta};
use proc_macro2::TokenStream;
use std::ops::Deref;
use syn::{punctuated::Punctuated, Expr, Generics, Ident, LitStr, Path, Token, Visibility};

pub type StructFields = Punctuated<StructField, Token![,]>;

#[derive(Debug)]
pub enum StructReprDefinition {
    Map(BasicStructReprDefinition),
    Listpairs(ListpairsStructReprDefinition),
    Tuple(TupleStructReprDefinition),
    Stringpairs(StringpairsStructReprDefinition),
    Stringjoin(StringjoinStructReprDefinition),
    Advanced(AdvancedStructReprDefinition),
}

#[derive(Debug)]
pub struct BasicStructReprDefinition {
    fields: StructFields,
}

#[derive(Debug)]
pub struct ListpairsStructReprDefinition {
    fields: StructFields,
}

#[derive(Debug)]
pub struct TupleStructReprDefinition {
    fields: StructFields,
    field_order: Option<Expr>,
}

#[derive(Debug)]
pub struct StringpairsStructReprDefinition {
    fields: StructFields,
    inner_delim: LitStr,
    entry_delim: LitStr,
}

#[derive(Debug)]
pub struct StringjoinStructReprDefinition {
    fields: StructFields,
    join: LitStr,
}

macro_rules! deref {
    ($($variant:ident => $type:ty,)*) => {
        $(
            impl Deref for $type {
                type Target = StructFields;
                fn deref(&self) -> &Self::Target {
                    &self.fields
                }
            }
        )*

        impl Deref for StructReprDefinition {
            type Target = StructFields;
            fn deref(&self) -> &Self::Target {
                match &self {
                    $(
                        Self::$variant(def) => &def.fields,
                    )*
                    Self::Advanced(AdvancedStructReprDefinition { fields, .. }) => fields,
                }
            }
        }
    };
}

deref! {
    Map => BasicStructReprDefinition,
    Listpairs => ListpairsStructReprDefinition,
    Tuple => TupleStructReprDefinition,
    Stringpairs => StringpairsStructReprDefinition,
    Stringjoin => StringjoinStructReprDefinition,
}

#[derive(Debug)]
pub struct AdvancedStructSchemaDefinition {
    pub meta: SchemaMeta,
    pub repr: AdvancedStructReprDefinition,
}

#[derive(Debug)]
pub struct AdvancedStructReprDefinition {
    pub name: Path,
    pub fields: StructFields,
    pub rest: TokenStream,
}

#[derive(Debug)]
pub struct StructField {
    pub attrs: InnerAttributes,
    pub vis: Visibility,
    pub key: Ident,
    pub value: Ident,
    pub generics: Option<Generics>,
    pub nullable: bool,
    pub optional: bool,
    pub implicit: Option<LitStr>,
    pub rename: Option<LitStr>,
}
