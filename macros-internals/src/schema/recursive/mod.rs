mod expand;
pub(crate) mod parse;

use crate::dev::SchemaMeta;
use proc_macro2::TokenStream;
use syn::{LitStr, Path, Type};

#[derive(Debug)]
pub enum ListReprDefinition {
    Basic { elem: Type, nullable: bool },
    Advanced(AdvancedListReprDefinition),
}

// #[derive(Debug)]
// pub struct AdvancedListSchemaDefinition {
//     pub meta: SchemaMeta,
//     pub repr_def: AdvancedListReprDefinition,
// }

#[derive(Debug)]
pub struct AdvancedListReprDefinition {
    pub name: Path,
    pub elem: Type,
    pub rest: TokenStream,
    pub nullable: bool,
}

impl ListReprDefinition {
    #[inline]
    pub fn elem_type(&self) -> (&Type, bool) {
        use AdvancedListReprDefinition as Adv;
        match self {
            Self::Basic { elem, nullable } => (elem, *nullable),
            Self::Advanced(Adv {
                name: _,
                elem,
                rest: _,
                nullable,
            }) => (elem, *nullable),
        }
    }
}

#[derive(Debug)]
pub enum MapReprDefinition {
    Basic {
        key: Type,
        value: Type,
        nullable: bool,
    },
    Listpairs {
        key: Type,
        value: Type,
        nullable: bool,
    },
    Stringpairs {
        key: Type,
        value: Type,
        nullable: bool,
        inner_delim: LitStr,
        entry_delim: LitStr,
    },
    Advanced(AdvancedMapReprDefinition),
}

// #[derive(Debug)]
// pub struct AdvancedMapSchemaDefinition {
//     pub meta: SchemaMeta,
//     pub repr_def: AdvancedMapReprDefinition,
// }

#[derive(Debug)]
pub struct AdvancedMapReprDefinition {
    pub name: Path,
    pub key: Type,
    pub value: Type,
    pub nullable: bool,
    pub rest: TokenStream,
}

impl MapReprDefinition {
    #[inline]
    pub fn field_type(&self) -> (&Type, &Type, bool) {
        match self {
            Self::Basic {
                key,
                value,
                nullable,
            } => (key, value, *nullable),
            Self::Stringpairs {
                key,
                value,
                nullable,
                inner_delim: _,
                entry_delim: _,
            } => (key, value, *nullable),
            Self::Listpairs {
                key,
                value,
                nullable,
            } => (key, value, *nullable),
            Self::Advanced(AdvancedMapReprDefinition {
                name: _,
                key,
                value,
                nullable,
                rest: _,
            }) => (key, value, *nullable),
        }
    }
}
