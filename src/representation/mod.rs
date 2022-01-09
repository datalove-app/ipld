//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.

mod context;
// mod executor;
mod impls;

pub use context::*;
// pub use executor::*;

use crate::dev::*;
// use downcast_rs::{impl_downcast, Downcast};
// use crate::selectors::args as Args;
use std::{rc::Rc, sync::Arc};

///
/// TODO: represents Schema or Representation kind?
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Bytes,
    List,
    Map,
    Link,
    Struct,
    Enum,
    Union,
    Copy,
}

// #[derive(Debug, Eq, Hash, PartialEq)]
// pub struct Field<A> {
//     /// Name of the `Representation` type contained within this field.
//     pub type_name: &'static str,
//
//     /// `Representation::Kind` of the field type.
//     pub value: Kind,
//
//     /// The serialized field name of this type.
//     pub alias: A,
// }

// impl<A> Field<A> {
//     // pub const fn new<T: Representation>(alias: A) -> Self {
//     //     Field {
//     //         type_name: T::NAME,
//     //         value: T::KIND,
//     //         alias,
//     //     }
//     // }
// }

// ///
// #[derive(Debug, Eq, Hash, PartialEq)]
// pub enum Fields {
//     None,
//     List(Field<()>),
//     Map {
//         key: Field<()>,
//         value: Field<()>,
//     },
//     Struct(&'static [(&'static str, Field<&'static str>)]),
//     Enum {
//         kind: Kind,
//         fields: &'static [Field<()>],
//     },
//     // Union representations
//     Envelope {
//         discriminant_key: &'static str,
//         fields: &'static [Field<&'static str>],
//     },
//     Inline {
//         discriminant_key: &'static str,
//         fields: &'static [Field<&'static str>],
//     },
//     Keyed(&'static [Field<&'static str>]),
//     Kinded(&'static [Field<&'static str>]),
//     Byteprefix(&'static [Field<&'static [u8; 1]>]),
// }

///
///
/// Some types have in-memory representations distinct from their IPLD representations:
///     - Links can map to types, so they can represent both CIDs and the underling types
///     - Signed/encrypted payloads can be further resolved into native types after verifying the signature/performing decryption
///
///
/// TODO? == what are the requirements?
///     - serialize a type to a block
///         - serialize an ipld as this type to a block
///     - deserialize a type from a block
///         -
///         - deserialize an ipld as this type from a block
///     - focus a type to a value (typed or ipld?) based on a selector
///         - ? call a closure on the selected type?
///     - transform a value within a type based on a selector (? and a closure ?)
///         - return a typed value or ipld?
///
/// TODO? selection can only happen *accurately* against fully-resolved types and blocks
///
/// TODO: what to impl?
///     - focus<T>(&self, selector, context) -> Result<T>
///     - patch<T, F>(&mut self, selector, f: F, context) -> Result<()>
///         where F: Fn(&mut T, context);
///         - based on success of recursing, flags any link type as dirty
///     - flush(&self, context) -> Result<Selector>
///     TODO? << other impls >>
///     - validate_selector(selector)
///         - TODO: ? returns a stateful Visitor + DeserializeSeed?
///     - derive Serialize
///     - in focus<T>(...), impl Deserialize
///         - TODO: ? stateful visitor derived from selector + type?
///         - TODO: ? impl DeserializeSeed for selector?
///         - TODO: ? Representation::visitor(selector: &Selector)
#[async_trait::async_trait]
pub trait Representation<C: Context = MemoryContext>
where
    Self: Serialize + for<'de> Deserialize<'de> + DeserializeOwned,
{
    /// The stringified name of the IPLD type.
    const NAME: &'static str;

    /// The stringified IPLD typedef.
    const SCHEMA: &'static str = unimplemented!();

    /// The IPLD Schema kind of the type.
    const KIND: Kind;

    // /// The type's `Select`able field names and their IPLD Schema kinds, if a recursive type.
    // const FIELDS: Fields = Fields::None;
    // ///
    // const HAS_LINKS: bool = false;

    ///
    /// for unions, this delegates to the variant's type name
    fn name(&self) -> &'static str {
        Self::NAME
    }

    ///
    fn kind(&self) -> Kind {
        Self::KIND
    }

    // ///
    // fn to_owned(&self) -> Self;

    // /// Returns true if any nested links have been resolved to blocks and subsequently mutated, and thus
    // /// need to be serialized first.
    // fn is_dirty(&self) -> bool {
    //     false
    // }

    // async fn resolve<C: Context>(&self, path: &Path, ctx: &mut C) -> Result<, Error> {
    //     unimplemented!()
    // }

    // async fn merge<C: Context>(&self, path: &Path, dag:  ctx: &mut C) -> Result<(), Error> {}

    // async fn write<Si: MultihashSize>(&self, ctx: C, block_meta: B) -> Result<Link<Self, Si>, Error> {
    //     unimplemented!()
    // }

    // async fn read<Si: MultihashSize>(link: Link<Self, Si>) -> Result<Self, Error> {
    //     unimplemented!()
    // }

    // fn links<R: Read + Seek>(c: Codec, reader: &mut R, )
}

impl<T> Representation for Box<T>
where
    T: Representation,
{
    const NAME: &'static str = T::NAME;
    const KIND: Kind = T::KIND;
    const SCHEMA: &'static str = T::SCHEMA;

    ///
    /// for unions, this delegates to the variant's type name
    #[inline]
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }

    ///
    #[inline]
    fn kind(&self) -> Kind {
        self.as_ref().kind()
    }
}

impl<T> Representation for Rc<T>
where
    T: Representation,
{
    const NAME: &'static str = T::NAME;
    const KIND: Kind = T::KIND;
    const SCHEMA: &'static str = T::SCHEMA;

    ///
    /// for unions, this delegates to the variant's type name
    #[inline]
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }

    ///
    #[inline]
    fn kind(&self) -> Kind {
        self.as_ref().kind()
    }
}

impl<T> Representation for Arc<T>
where
    T: Representation,
{
    const NAME: &'static str = T::NAME;
    const KIND: Kind = T::KIND;
    const SCHEMA: &'static str = T::SCHEMA;

    ///
    /// for unions, this delegates to the variant's type name
    #[inline]
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }

    ///
    #[inline]
    fn kind(&self) -> Kind {
        self.as_ref().kind()
    }
}

// impl<'a, T> Representation for &'a T
// where
//     T: Representation,
// {
//     const NAME: &'static str = T::NAME;
// }

// impl<'a, T> Representation for &'a mut T
// where
//     T: Representation,
// {
//     const NAME: &'static str = T::NAME;
// }

// // #[async_trait]
// pub trait RepresentationExt<T: Representation>: Representation {
//     // fn resolve(self)
// }

// ///
// /// TODO: possibly look at erased-serde to complete this "hack"
// #[doc(hidden)]
// pub trait ObjectSafeRepresentation: Downcast {}
// impl<T: Representation + 'static> ObjectSafeRepresentation for T {}
// impl_downcast!(ObjectSafeRepresentation);
