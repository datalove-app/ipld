mod expand_list;
mod expand_map;
mod parse;

use crate::dev::SchemaMeta;
use proc_macro2::TokenStream;
use syn::{LitStr, Path, Type};

#[derive(Debug)]
pub enum ListReprDefinition {
    Basic { elem: Type, nullable: bool },
    Advanced(AdvancedListReprDefinition),
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

#[derive(Debug)]
pub struct AdvancedMapReprDefinition {
    pub name: Path,
    pub key: Type,
    pub value: Type,
    pub nullable: bool,
    pub rest: TokenStream,
}

#[derive(Debug)]
pub struct AdvancedListSchemaDefinition {
    pub meta: SchemaMeta,
    pub repr: AdvancedListReprDefinition,
}

#[derive(Debug)]
pub struct AdvancedListReprDefinition {
    pub name: Path,
    pub elem: Type,
    pub rest: TokenStream,
    pub nullable: bool,
}

#[derive(Debug)]
pub struct AdvancedMapSchemaDefinition {
    pub meta: SchemaMeta,
    pub repr: AdvancedMapReprDefinition,
}
