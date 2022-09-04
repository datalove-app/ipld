mod expand;
mod expand_bytes;
mod expand_link;
mod parse;

use crate::dev::{SchemaKind, SchemaMeta};
use proc_macro2::TokenStream;
use syn::{Path, Type};

#[derive(Debug)]
pub struct NullReprDefinition;

#[derive(Debug)]
pub struct BoolReprDefinition;

#[derive(Debug)]
pub struct IntReprDefinition(pub Type, pub SchemaKind);

#[derive(Debug)]
pub struct FloatReprDefinition(pub Type, pub SchemaKind);

#[derive(Debug)]
pub struct StringReprDefinition;

#[derive(Debug)]
pub struct LinkReprDefinition(pub Type);

#[derive(Debug)]
pub struct CopyReprDefinition(pub Type);

#[derive(Debug)]
pub enum BytesReprDefinition {
    Basic,
    Advanced(AdvancedBytesReprDefinition),
}

#[derive(Debug)]
pub struct AdvancedBytesSchemaDefinition {
    pub meta: SchemaMeta,
    pub repr: AdvancedBytesReprDefinition,
}

#[derive(Debug)]
pub struct AdvancedBytesReprDefinition {
    pub name: Path,
    pub rest: TokenStream,
}
