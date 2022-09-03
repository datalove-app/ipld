//! While all types and their IPLD representations ultimately dictate how the
//! type is resolved from/writen to blocks of bytes, *how* those bytes may be
//! provided (as well any additional requirements unique to the representation,
//! such as additional blocks, encryption keys, etc) can vary on how and where
//! the type is being used (e.g, in WASM, making partial/range queries,
//! querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a
//! type from/to bytes, as well query and mutate a type, while specifically
//! defining for the type it's `Context` requirements for these operations.

use crate::dev::*;
use downcast_rs::{impl_downcast, Downcast};
use std::{rc::Rc, sync::Arc};

/// Enum of possible [Data Model]() and [Schema]() kinds.
///
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    ///
    Null,
    ///
    Bool,
    // ///
    // Int8,
    // ///
    // Int16,
    ///
    Int,
    // ///
    // Int64,
    // ///
    // Int128,
    // ///
    // Uint8,
    // ///
    // Uint16,
    // ///
    // Uint32,
    // ///
    // Uint64,
    // ///
    // Uint128,
    // ///
    // Float32,
    ///
    Float,
    ///
    String,
    ///
    Bytes,
    ///
    List,
    ///
    Map,
    ///
    Link,
    ///
    Struct,
    ///
    Enum,
    ///
    Union,
    ///
    Copy,
}

// ///
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
//
// impl<A> Field<A> {
//     // pub const fn new<T: Representation>(alias: A) -> Self {
//     //     Field {
//     //         type_name: T::NAME,
//     //         value: T::KIND,
//     //         alias,
//     //     }
//     // }
// }
//
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
pub trait Representation
where
    Self: Serialize + DeserializeOwned,
{
    /// The stringified name of the IPLD type.
    const NAME: &'static str;

    /// The stringified IPLD type definition (or equivalent, if a native type
    /// not defined by IPLD).
    const SCHEMA: &'static str = unimplemented!();

    /// The IPLD [Data Model Kind](https://ipld.io/docs/data-model/kinds/) of
    /// the type.
    const DATA_MODEL_KIND: Kind;

    /// The IPLD [Schema
    /// Kind](https://ipld.io/docs/schemas/features/typekinds/#schema-kinds) of
    /// the type.
    const SCHEMA_KIND: Kind = Self::DATA_MODEL_KIND;

    /// The IPLD [Representation Kind]() of the type.
    const REPR_KIND: Kind = Self::DATA_MODEL_KIND;

    ///
    const IS_LINK: bool = false;

    ///
    const HAS_LINKS: bool = Self::IS_LINK;

    // /// The type's `Select`able field names and their IPLD Schema kinds, if a recursive type.
    // const FIELDS: Fields = Fields::None;

    ///
    /// for unions, this ?should delegate to the variant's type name
    fn name(&self) -> &'static str {
        Self::NAME
    }

    // ///
    // fn kind(&self) -> Kind {
    //     Self::KIND
    // }

    ///
    fn has_links(&self) -> bool {
        Self::HAS_LINKS
    }

    // fn r#match<'de, 'a, C, D>(
    //     seed: ContextSeed<'a, C, Self, Self>,
    //     deserializer: D,
    // ) -> Result<Self, D::Error>
    // where
    //     C: Context,
    //     D: Deserializer<'de>;
    //
    // // TODO: do we even need this
    // fn try_deserialize_path<'de, T, D>(
    //     selector: PathSeed<T>,
    //     deserializer: D,
    // ) -> Result<(Option<PathSeed<T>>, Option<T>), Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     unimplemented!()
    // }
    //
    // /// Defines how the type can be selectively deserialized, based on its kind
    // /// and serialized representation.
    // /// TODO: do we even need this
    // fn deserialize_selector<'de, D>(
    //     seed: SelectorSeed,
    //     deserializer: D,
    // ) -> Result<SelectorSeed, Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     unimplemented!()
    // }
    //
    // ///
    // fn to_owned(&self) -> Self;
    //
    // /// Returns true if any nested links have been resolved to blocks and subsequently mutated, and thus
    // /// need to be serialized first.
    // fn is_dirty(&self) -> bool {
    //     false
    // }

    // fn links<R: Read + Seek>(c: Codec, reader: &mut R, )
}

impl<T> Representation for Option<T>
where
    T: Representation,
{
    const NAME: &'static str = "Option";
    const SCHEMA: &'static str = concat!("nullable ", stringify!(T::NAME));
    // TODO
    const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
    const SCHEMA_KIND: Kind = T::DATA_MODEL_KIND;
    const HAS_LINKS: bool = T::HAS_LINKS;

    fn name(&self) -> &'static str {
        match self {
            Self::None => Null::NAME,
            Self::Some(t) => t.name(),
        }
    }

    // fn kind(&self) -> Kind {
    //     match self {
    //         Self::None => Null::KIND,
    //         Self::Some(t) => t.kind(),
    //     }
    // }

    fn has_links(&self) -> bool {
        match self {
            Self::None => false,
            Self::Some(t) => t.has_links(),
        }
    }
}

macro_rules! impl_wrapper {
    ($wrapper:ident) => {
        impl<T> Representation for $wrapper<T>
        where
            T: Representation,
        {
            const NAME: &'static str = T::NAME;
            const SCHEMA: &'static str = T::SCHEMA;
            const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
            const REPR_KIND: Kind = T::REPR_KIND;

            #[inline]
            fn name(&self) -> &'static str {
                self.as_ref().name()
            }

            #[inline]
            fn has_links(&self) -> bool {
                self.as_ref().has_links()
            }
        }
    };
    (@dyn $wrapper:ident) => {
        impl Representation for $wrapper<dyn ErasedRepresentation> {
            const NAME: &'static str = T::NAME;
            const SCHEMA: &'static str = T::SCHEMA;
            const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
            const REPR_KIND: Kind = T::REPR_KIND;

            #[inline]
            fn name(&self) -> &'static str {
                self.as_ref().name()
            }

            #[inline]
            fn has_links(&self) -> bool {
                self.as_ref().has_links()
            }
        }
    };
}

impl_wrapper!(Box);
impl_wrapper!(Rc);
impl_wrapper!(Arc);

///
/// TODO: possibly look at erased-serde to complete this "hack"
pub(crate) trait ErasedRepresentation: Downcast {
    // /// The underlying [`Representation`] type this type will downcast to.
    // type Representation: Representation = Self;

    ///
    fn name(&self) -> &'static str;

    ///
    fn data_model_kind(&self) -> Kind;

    ///
    fn schema_kind(&self) -> Kind;

    ///
    fn repr_kind(&self) -> Kind;
}

// impl_downcast!(sync ErasedRepresentation assoc Representation
//    where Representation: crate::Representation);
impl_downcast!(ErasedRepresentation);

impl<T> ErasedRepresentation for T
where
    T: Representation + 'static,
{
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn data_model_kind(&self) -> Kind {
        T::DATA_MODEL_KIND
    }

    fn schema_kind(&self) -> Kind {
        T::SCHEMA_KIND
    }

    fn repr_kind(&self) -> Kind {
        T::REPR_KIND
    }
}

///
// #[derive(Debug)]
pub struct AnyRepresentation(Box<dyn ErasedRepresentation>);

impl AnyRepresentation {
    ///
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: Representation + 'static,
    {
        (*self.0).as_any().is::<T>()
    }

    ///
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Representation + 'static,
    {
        (*self.0).as_any().downcast_ref()
    }

    ///
    #[inline]
    pub fn downcast<T>(self) -> Result<T, Error>
    where
        T: Representation + 'static,
    {
        let dag = self
            .0
            .downcast()
            .map_err(|_| Error::DowncastFailure(T::NAME))?;
        Ok(*dag)
    }
}

impl<T: Representation + 'static> From<T> for AnyRepresentation {
    fn from(dag: T) -> Self {
        Self(Box::new(dag))
    }
}

// impl<T: Representation + 'static> TryFrom<AnyRepresentation> for T {
//     type Error = Error;
//     // fn try_into(self) -> Result<T, Self::Error> {
//     fn try_from(any: AnyRepresentation) -> Result<Self, Self::Error> {
//         any.0.downcast().map_err(Error::DowncastFailure)
//     }
// }

// impl Debug for dyn ErasedRepresentation {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // let dag = Self::downcast_as(self).unrwap();
//         // dag.fmt(f)
//         // self.downcast_as().fmt(f)
//         unimplemented!()
//     }
// }

// pub trait ErasedRepresentationExt: ErasedRepresentation {
//     ///
//     #[inline]
//     fn is(&self) -> bool {
//         (*self)
//             .as_any()
//             .is::<<Self as ErasedRepresentation>::Representation>()
//     }

//     ///
//     #[inline]
//     fn downcast_as(&self) -> Option<&<Self as ErasedRepresentation>::Representation> {
//         (*self).as_any().downcast_ref()
//     }
// }

// impl ErasedRepresentationExt for dyn ErasedRepresentation
// // where
// //     T: Representation + ErasedRepresentation + 'static,
// {
//     type Representation = Self;
// }

/*

/// Helper trait. `VALUE` is false, except for the specialization of the
/// case where `T == U`.
pub(crate) trait TypeEq<U: ?Sized> {
    const EQ: bool;
}

// Default implementation.
impl<T: ?Sized, U: ?Sized> TypeEq<U> for T {
    default const EQ: bool = false;
}

// Specialization for `T == U`.
impl<T: ?Sized> TypeEq<T> for T {
    const EQ: bool = true;
}

#[doc(hidden)]
pub const fn type_eq<T: ?Sized, U: ?Sized>() -> bool {
    <T as TypeEq<U>>::EQ
}

/// Helper fn for constraining and safely transmuting a generic selection output
pub(crate) fn type_cast_selection<T: Sized + 'static, U: Sized + 'static, E, F>(
    inner: F,
) -> Result<Option<U>, E>
where
    F: FnOnce() -> Result<Option<T>, E>,
{
    if !type_eq::<T, U>() {
        unreachable!("should only do this for types known to be identical")
    }

    let mut inner = inner()?;
    let outer = (&mut inner as &mut dyn std::any::Any)
        .downcast_mut::<Option<U>>()
        .unwrap()
        .take();
    Ok(outer)
}

pub(crate) fn type_cast_mut<T: Sized + 'static, U: Sized + 'static, E, F>(inner: &mut T) -> &mut U {
    if !type_eq::<T, U>() {
        unreachable!("should only do this for types known to be identical")
    }

    (inner as &mut dyn std::any::Any)
        .downcast_mut::<Option<&mut U>>()
        .unwrap()
        .take()
        .unwrap()
}

pub(crate) trait TypeEq2<const EQ: bool, U: ?Sized> {}
// Default implementation.
default impl<T: ?Sized, U: ?Sized> TypeEq2<false, U> for T {}
impl<T: ?Sized> TypeEq2<true, T> for T {}

pub const fn type_eq2<const EQ: bool, T: ?Sized, U: ?Sized>() -> bool {
    EQ
}
 */
