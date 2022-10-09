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

mod strategies;

use crate::dev::*;
use downcast_rs::{impl_downcast, Downcast};
use macros::derive_more::From;
use maybestd::{fmt, str::FromStr};

pub use ipld_macros_internals::schema::{type_kinds, SchemaKind as Kind, TypedKind};

#[doc(hidden)]
pub use strategies::*;

#[cfg(feature = "old")]
mod kind {
    // use bitflags::bitflags;

    // bitflags! {
    //     /// Enum of possible [Data Model](), [Schema]() and [Representation]() kinds.
    //     ///
    //     pub struct Kind: u16 {
    //         // data model kinds

    //         ///
    //         const Null = 0b0000_0000_0000_0001;
    //         ///
    //         const Bool = 0b0000_0000_0000_0010;
    //         ///
    //         const Int = 0b0000_0000_0000_0100;
    //         ///
    //         const Float = 0b0000_0000_0000_1000;
    //         ///
    //         const String = 0b0000_0000_0001_0000;
    //         ///
    //         const Bytes = 0b0000_0000_0010_0000;
    //         ///
    //         const List = 0b0000_0000_0100_0000;
    //         ///
    //         const Map = 0b0000_0000_1000_0000;
    //         ///
    //         const Link = 0b0000_0001_0000_0000;

    //         // schema kinds

    //         ///
    //         const Struct = 0b0000_0010_0000_0000;
    //         ///
    //         const Enum = 0b0000_0100_0000_0000;
    //         ///
    //         const Union = 0b0000_1000_0000_0000;

    //         // any

    //         ///
    //         const Any = Self::Null.bits
    //             | Self::Bool.bits
    //             | Self::Int.bits
    //             | Self::Float.bits
    //             | Self::String.bits
    //             | Self::Bytes.bits
    //             | Self::List.bits
    //             | Self::Map.bits
    //             | Self::Link.bits;
    //     }
    // }

    // impl Kind {
    //     /// Const function for determining equality between [`Kind`]s.
    //     pub const fn eq(&self, other: &Self) -> bool {
    //         match (*self, *other) {
    //             (Self::Null, Self::Null)
    //             | (Self::Bool, Self::Bool)
    //             | (Self::Int, Self::Int)
    //             | (Self::Float, Self::Float)
    //             | (Self::String, Self::String)
    //             | (Self::Bytes, Self::Bytes)
    //             | (Self::List, Self::List)
    //             | (Self::Map, Self::Map)
    //             | (Self::Link, Self::Link)
    //             | (Self::Struct, Self::Struct)
    //             | (Self::Enum, Self::Enum)
    //             | (Self::Union, Self::Union) => true,
    //             _ => false,
    //         }
    //     }
    // }

    // ///
    // #[derive(Copy, Clone, Debug)]
    // pub enum SchemaKind {
    //     Null = Kind::Null.bits as isize,
    //     Bool = Kind::Bool.bits as isize,
    //     Int = Kind::Int.bits as isize,
    //     Float = Kind::Float.bits as isize,
    //     String = Kind::String.bits as isize,
    //     Bytes = Kind::Bytes.bits as isize,
    //     List = Kind::List.bits as isize,
    //     Map = Kind::Map.bits as isize,
    //     Link = Kind::Link.bits as isize,
    //     Struct = Kind::Struct.bits as isize,
    //     Enum = Kind::Enum.bits as isize,
    //     Union = Kind::Union.bits as isize,
    // }
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

// trait IsAny<const K: u16> {}
// impl IsAny<{ Kind::Any.bits() }> for Kind {}
// impl IsAny<{ Kind::Null.bits() }> for Kind {}
// impl IsAny<{ Kind::Bool.bits() }> for Kind {}
// impl IsAny<{ Kind::Int.bits() }> for Kind {}
// impl IsAny<{ Kind::Float.bits() }> for Kind {}
// impl IsAny<{ Kind::String.bits() }> for Kind {}
// impl IsAny<{ Kind::Bytes.bits() }> for Kind {}
// impl IsAny<{ Kind::List.bits() }> for Kind {}
// impl IsAny<{ Kind::Map.bits() }> for Kind {}
// impl IsAny<{ Kind::Link.bits() }> for Kind {}

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
pub trait Representation: Sized {
    // /// Marker type for exact `u32` value of the type's [`Representation::DATA_MODEL_KIND`], needed for internal blanket implementations of various traits.
    // #[doc(hidden)]
    // type DATA_MODEL_KIND: TypedKind;
    // /// Marker type for exact `u32` value of the type's [`Representation::SCHEMA_KIND`], needed for internal blanket implementations of various traits.
    // #[doc(hidden)]
    // type SCHEMA_KIND: TypedKind;
    // /// Marker type for exact `u32` value of the type's [`Representation::REPR_KIND`], needed for internal blanket implementations of various traits.
    // #[doc(hidden)]
    // type REPR_KIND: TypedKind;

    // TODO: use this seed in Representation::deserialize by default
    // type Seed<'de, const C: u64>: Default + DeserializeSeed<'de, Value = Self>;

    /// The stringified name of the IPLD type.
    const NAME: &'static str;

    /// The stringified IPLD type definition (or equivalent, if a native type
    /// not defined by IPLD).
    /// TODO: we cant concat generic consts, only concrete ones - so refactor this to a function
    const SCHEMA: &'static str = unimplemented!();

    /// The IPLD [Data Model Kind](https://ipld.io/docs/data-model/kinds/) of
    /// the type, which would inform a user of its access patterns.
    const DATA_MODEL_KIND: Kind; // Self::DATA_MODEL_KIND::KIND

    /// The IPLD [Schema
    /// Kind](https://ipld.io/docs/schemas/features/typekinds/#schema-kinds) of
    /// the type.
    const SCHEMA_KIND: Kind = Self::DATA_MODEL_KIND;

    /// The IPLD [Representation Kind]() of the type, which would inform a user of how the type is represented when encoded.
    const REPR_KIND: Kind = Self::DATA_MODEL_KIND; // K

    // const IS_ADL: bool = false;

    /// Marker for types that can have any representation *and* should be
    /// ignored entirely during selection/deserialization.
    #[doc(hidden)]
    const __IGNORED: bool = false;

    ///
    /// todo deprecate
    const IS_LINK: bool = Self::DATA_MODEL_KIND.is_link();

    /// todo deprecate
    const HAS_LINKS: bool = Self::IS_LINK;

    /// The type's `Select`able static field names and their IPLD Schema kinds.
    const FIELDS: &'static [(Field<'static>, Kind)] = &[];

    ///
    /// for unions, this ?should delegate to the variant's type name'
    /// TODO what is this really for
    fn name(&self) -> &'static str {
        Self::NAME
    }

    ///
    fn data_model_kind(&self) -> Kind {
        Self::DATA_MODEL_KIND
    }

    ///
    fn schema_kind(&self) -> Kind {
        Self::SCHEMA_KIND
    }

    ///
    fn repr_kind(&self) -> Kind {
        Self::REPR_KIND
    }

    ///
    fn is_link(&self) -> bool {
        Self::IS_LINK
    }

    ///
    fn has_links(&self) -> bool {
        Self::HAS_LINKS
    }

    ///
    fn as_field(&self) -> Option<Field<'_>> {
        None
    }

    fn to_selected_node(&self) -> SelectedNode {
        unimplemented!()
    }

    /// Replacement method for [`serde::Serialize::serialize`] that allows us
    /// switch serialization behaviour based on the provided [`Codec`].
    ///
    /// Defaults to the type's underlying [`serde::Serialize::serialize`]
    /// implementation.
    /// TODO: remove the default impl, then remove the trait bounds
    /// TODO: rename to encode? as in, just encode this type (up to links)
    #[inline]
    #[doc(hidden)]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // <Self as Serialize>::serialize(self, serializer)
        unimplemented!()
    }

    /// Replacement method for [`serde::Deserialize::deserialize`] that allows
    /// us switch deserialization behaviour based on the provided [`CodecExt`].
    ///
    /// Defaults to the type's underlying [`serde::Deserialize::deserialize`]
    /// implementation.
    /// TODO: remove the default impl, then remove the trait bounds
    /// TODO: rename to decode? as in, just decode this type (up to links)
    #[inline]
    #[doc(hidden)]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // <Self as Deserialize<'de>>::deserialize(deserializer)
        unimplemented!()
    }

    // ///
    // #[inline]
    // #[doc(hidden)]
    // fn deserialize_seed<'de, const C: u64, S, D>(
    //     seed: S,
    //     deserializer: D,
    // ) -> Result<S::Value, D::Error>
    // where
    //     S: CodecDeserializeSeed<'de>,
    //     D: Deserializer<'de>,
    // {
    //     seed.deserialize(deserializer)
    // }

    // ///
    // #[inline]
    // #[doc(hidden)]
    // fn deserialize_with_visitor<'de, const C: u64, D, V>(
    //     deserializer: D,
    //     visitor: V,
    // ) -> Result<V::Value, D::Error>
    // where
    //     D: Deserializer<'de>,
    //     V: Visitor<'de>;

    // #[inline]
    // #[doc(hidden)]
    // fn deserialize<'de, const C: u64, D, V>(
    //     deserializer: D,
    //     visitor: V,
    // ) -> Result<V::Value, D::Error>
    // where
    //     D: Deserializer<'de>,
    //     V: Visitor<'de>;

    // ///
    // #[inline]
    // fn deserialize_seed<'de, const C: u64, D, S>(
    //     deserializer: D,
    //     seed: S,
    // ) -> Result<S::Value, D::Error>
    // where
    //     D: Deserializer<'de> + CodecExt<C>,
    //     S: DeserializeSeed<'de>,
    // {
    //     seed.deserialize(deserializer)
    // }

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

///
pub trait StringRepresentation
where
    Self: Representation + Clone + fmt::Display + FromStr<Err = Error> + Ord,
{
}
impl<T> StringRepresentation for T where
    Self: Representation + Clone + fmt::Display + FromStr<Err = Error> + Ord
{
}

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

    ///
    fn has_links(&self) -> bool;
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

    fn has_links(&self) -> bool {
        T::HAS_LINKS
    }
}

///
#[derive(From)]
pub struct AnyRepresentation {
    erased: Box<dyn ErasedRepresentation>,
    // is_partial: bool,
}

impl AnyRepresentation {
    ///
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: Representation + 'static,
    {
        (*self.erased).as_any().is::<T>()
    }

    ///
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Representation + 'static,
    {
        (*self.erased).as_any().downcast_ref()
    }

    ///
    #[inline]
    pub fn downcast<T>(self) -> Result<T, Error>
    where
        T: Representation + 'static,
    {
        // if self.is_partial {
        //     Err(Error::downcast_failure::<T>(
        //         "cannot downcast a partially-loaded dag",
        //     ))
        // } else {
        let dag = self
            .erased
            .downcast()
            .map_err(|_| Error::downcast_failure::<T>("incorrect type"))?;
        Ok(*dag)
        // }
    }

    // pub fn roundtrip<T, U>(self)

    // ///
    // #[inline]
    // pub fn cast_between<T, U>(self) -> Result<Self, Error>
    // where
    //     T: Representation + 'static,
    //     U: Representation + From<T> + 'static,
    // {
    //     let dag = self.downcast::<T>()?;
    //     Ok(U::from(dag).into())
    // }
}

impl<T: Representation + 'static> From<T> for AnyRepresentation {
    fn from(dag: T) -> Self {
        Self {
            erased: Box::new(dag),
            // is_partial: false,
        }
    }
}

// ///
// #[doc(hidden)]
// pub trait IsKind<const DMK: u16, const SK: u16, const RK: u16, T> {}
// impl<const DMK: u16, const SK: u16, const RK: u16, T> IsTrue<T> for () where T: Representation {}

// mod type_eq {
//     #[doc(hidden)]
//     ///
//     pub trait TypeEq<const EQ: bool, U: ?Sized> {}
//     // Default implementation.
//     default impl<T: ?Sized, U: ?Sized> TypeEq<false, U> for T {}
//     impl<T: ?Sized> TypeEq<true, T> for T {}

//     pub const fn cmp<const EQ: bool, T: ?Sized, U: ?Sized>() -> bool {
//         EQ
//     }
// }

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

// #[doc(hidden)]
// mod typeeq {
//     pub trait TypeEq<const EQ: bool, U: ?Sized> {
//         const EQ: bool = EQ; // impls!( Self: TypeEq<true, U> )
//     }

//     // // Default implementation.
//     impl<T: ?Sized, U: ?Sized> TypeEq<false, U> for T {}

//     // Specialization for `T == U`.
//     impl<T: ?Sized> TypeEq<true, Self> for T {}

//     #[doc(hidden)]
//     pub const fn type_eq<const EQ: bool, T: ?Sized, U: ?Sized>() -> bool
// // where
//     //     T: TypeEq<EQ, U>,
//     {
//         // impls!( Self: TypeEq<true, U> )
//         <T as TypeEq<EQ, U>>::EQ
//     }
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
 */

/// Helper fn for constraining and safely transmuting a generic selection output
pub(crate) fn type_cast_selection<T: Sized + 'static, U: Sized + 'static, E, F>(
    inner: F,
) -> Result<Option<U>, E>
where
    F: FnOnce() -> Result<Option<T>, E>,
{
    // if !type_eq::<T, U>() {
    //     unreachable!("should only do this for types known to be identical")
    // }

    let mut inner = inner()?;
    let outer = (&mut inner as &mut dyn std::any::Any)
        .downcast_mut::<Option<U>>()
        .unwrap()
        .take();
    Ok(outer)
}
