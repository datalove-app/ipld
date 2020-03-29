mod expand;
mod parse;

use crate::{Fields, InnerAttributes};
use std::ops::Deref;
use syn::{parse::Parse, LitInt, LitStr, Type};

#[derive(Debug)]
pub enum UnionReprDefinition {
    Keyed {
        fields: UnionStrFields,
    },
    Envelope {
        fields: UnionStrFields,
        discriminant_key: LitStr,
        content_key: LitStr,
    },
    Inline {
        fields: UnionStrFields,
        discriminant_key: LitStr,
    },
    BytePrefix {
        fields: UnionIntFields,
    },
    Kinded {
        fields: UnionKindedFields,
    },
}

pub type UnionStrFields = Fields<UnionField<LitStr>>;
pub type UnionIntFields = Fields<UnionField<LitInt>>;
pub type UnionKindedFields = Fields<UnionField<DataModelKind>>;

#[derive(Debug)]
pub struct UnionField<T: Parse> {
    pub attrs: InnerAttributes,
    pub value: Type,
    pub key: T,
    pub linked: bool,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DataModelKind {
    Null,
    Boolean,
    Integer,
    Float,
    Bytes,
    String,
    List,
    Map,
    Link,
}
