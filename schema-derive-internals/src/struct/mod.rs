mod expand;
mod parse;

use crate::{InnerAttributes, SchemaMeta};
use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, Expr, Ident, LitStr, Path, Token, Type};

pub type StructFields = Punctuated<StructField, Token![,]>;

#[derive(Debug)]
pub enum StructReprDefinition {
    Map {
        fields: StructFields,
    },
    Listpairs {
        fields: StructFields,
    },
    Tuple {
        fields: StructFields,
        field_order: Option<Expr>,
    },
    Stringpairs {
        fields: StructFields,
        inner_delim: LitStr,
        entry_delim: LitStr,
    },
    Stringjoin {
        fields: StructFields,
        join: LitStr,
    },
    Advanced(AdvancedStructReprDefinition),
}

impl StructReprDefinition {
    #[inline]
    pub fn fields(&self) -> &StructFields {
        match self {
            Self::Map { fields } => fields,
            Self::Listpairs { fields } => fields,
            Self::Tuple {
                fields,
                field_order: _,
            } => fields,
            Self::Stringpairs {
                fields,
                inner_delim: _,
                entry_delim: _,
            } => fields,
            Self::Stringjoin { fields, join: _ } => fields,
            Self::Advanced(AdvancedStructReprDefinition {
                name: _,
                fields,
                rest: _,
            }) => fields,
        }
    }
}

#[derive(Debug)]
pub struct AdvancedStructSchema {
    meta: SchemaMeta,
    repr: AdvancedStructReprDefinition,
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
    pub key: Ident,
    pub value: Type,
    pub nullable: bool,
    pub optional: bool,
    pub implicit: Option<LitStr>,
    pub rename: Option<LitStr>,
}
