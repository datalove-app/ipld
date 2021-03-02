mod expand;
mod parse;

use crate::dev::{Fields, InnerAttributes};
use std::ops::Deref;
use syn::{parse::Parse, Ident, LitInt, LitStr, Token, Type};

#[derive(Debug)]
pub enum EnumReprDefinition {
    String { fields: EnumStrFields },
    Int { fields: EnumIntFields },
}

pub type EnumStrFields = Fields<EnumStrField>;

#[derive(Debug)]
pub struct EnumIntFields {
    repr_type: Type,
    fields: Fields<EnumIntField>,
}

#[derive(Debug)]
pub struct EnumStrField {
    pub attrs: InnerAttributes,
    pub name: Ident,
    pub alias: Option<LitStr>,
}

#[derive(Debug)]
pub struct EnumIntField {
    pub attrs: InnerAttributes,
    pub name: Ident,
    pub alias: LitInt,
}
