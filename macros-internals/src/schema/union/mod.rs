// TODO? impl flatten types manually (in serialize impl)
mod expand; // struct/map flattens, enum default
mod expand_byte_prefix;
mod expand_envelope; // struct/map flattens, enum tag=dk, content=ck
mod expand_inline; // struct/map flattens, enum tag=dk
mod expand_kinded; // enum untagged, but by schema kind
mod parse;

use crate::dev::{schema::SchemaKind, Fields, OuterAttributes};
use std::ops::Deref;
use syn::{parse::Parse, Generics, Ident, LitInt, LitStr, Type};

#[derive(Debug)]
pub enum UnionReprDefinition {
    Keyed(KeyedUnionReprDefinition),
    Envelope(EnvelopeUnionReprDefinition),
    Inline(InlineUnionReprDefinition),
    BytePrefix(BytePrefixUnionReprDefinition),
    Kinded(KindedUnionReprDefinition),
}

#[derive(Debug)]
pub struct KeyedUnionReprDefinition {
    fields: UnionStrFields,
}
#[derive(Debug)]
pub struct EnvelopeUnionReprDefinition {
    fields: UnionStrFields,
    discriminant_key: LitStr,
    content_key: LitStr,
}
#[derive(Debug)]
pub struct InlineUnionReprDefinition {
    fields: UnionStrFields,
    discriminant_key: LitStr,
}
#[derive(Debug)]
pub struct BytePrefixUnionReprDefinition {
    fields: UnionIntFields,
}
#[derive(Debug)]
pub struct KindedUnionReprDefinition {
    fields: UnionKindedFields,
}

macro_rules! deref {
    ($($variant:ident, $field:ty => $type:ty,)*) => {
        $(
            impl Deref for $type {
                type Target = Fields<UnionField<$field>>;
                fn deref(&self) -> &Self::Target {
                    &self.fields
                }
            }
        )*
    };
}

deref! {
    Keyed, LitStr => KeyedUnionReprDefinition,
    Envelope, LitStr => EnvelopeUnionReprDefinition,
    Inline, LitStr => InlineUnionReprDefinition,
    BytePrefix, LitInt => BytePrefixUnionReprDefinition,
    Kinded, SchemaKind => KindedUnionReprDefinition,
}

pub type UnionStrField = UnionField<LitStr>;
pub type UnionIntField = UnionField<LitInt>;
pub type UnionKindedField = UnionField<SchemaKind>;
pub type UnionStrFields = Fields<UnionStrField>;
pub type UnionIntFields = Fields<UnionIntField>;
pub type UnionKindedFields = Fields<UnionKindedField>;

#[derive(Debug)]
pub struct UnionField<T: Parse> {
    pub attrs: OuterAttributes,
    pub wrapper: Option<Type>,
    pub value: Ident,
    pub generics: Option<Generics>,
    pub key: T,
    pub linked: bool,
}
