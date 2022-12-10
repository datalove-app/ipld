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

#[doc(hidden)]
mod strategies;

use crate::dev::*;
use downcast_rs::{impl_downcast, Downcast};
use macros::derive_more::From;
use maybestd::{fmt, marker::PhantomData, str::FromStr};

pub use ipld_macros_internals::schema::SchemaKind as Kind;
#[doc(hidden)]
pub use strategies::*;

// ///
// #[derive(Debug, Eq, Hash, PartialEq)]
// pub struct Field<T, I = ()> {
//     /// Name of the `Representation` type contained within this field.
//     pub type_name: &'static str,
//
//     /// `Representation::Kind` of the field type.
//     pub ty: PhantomData<T>
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
//     List,
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

#[doc(hidden)]
pub struct Label<T: Representation> {
    label: Field<'static>,
    _t: PhantomData<T>,
}

///
///
// Some types have in-memory representations distinct from their IPLD representations:
//     - Links can map to types, so they can represent both CIDs and the underling types
//     - Signed/encrypted payloads can be further resolved into native types after verifying the signature/performing decryption
//
//     - patch<T, F>(&mut self, selector, f: F, context) -> Result<()>
//         where F: Fn(&mut T, context);
//         - based on success of recursing, flags any link type as dirty
//     - flush(&self, context) -> Result<Selector>
pub trait Representation: Sized {
    /// The stringified name of the IPLD type.
    const NAME: &'static str;

    /// The stringified IPLD type definition (or equivalent, if a native type
    /// not defined by IPLD).
    const SCHEMA: &'static str;

    /// The IPLD [Data Model Kind](https://ipld.io/docs/data-model/kinds/) of
    /// the type, which denotes the access API and executable [`Selector`]s.
    const DATA_MODEL_KIND: Kind;

    /// The IPLD [Schema Kind](https://ipld.io/docs/schemas/features/typekinds/)
    /// of the type, which denotes how the type is defined by its schema.
    const SCHEMA_KIND: Kind = Self::DATA_MODEL_KIND;

    /// The IPLD [Representation Kind]() of the type, which, in combination with
    /// the [`Representation::REPR_STRATEGY`], denotes how the type is
    /// represented on-disk by any given codec.
    const REPR_KIND: Kind = Self::DATA_MODEL_KIND;

    /// The IPLD specific [Representation]() [`Strategy`] used to encode this
    /// type.
    const REPR_STRATEGY: Strategy = Strategy::Basic;

    /// Whether or not the type contains IPLD links.
    const HAS_LINKS: bool = Self::DATA_MODEL_KIND.contains(Kind::Link);

    // /// Marker type for exact `u32` value of the type's
    // /// [`Representation::DATA_MODEL_KIND`], needed for internal blanket
    // /// implementations of various traits.
    // #[doc(hidden)]
    // type DataModelKind: TypedKind;

    // /// Marker type for exact `u32` value of the type's
    // /// [`Representation::SCHEMA_KIND`], needed for internal blanket
    // /// implementations of various traits.
    // #[doc(hidden)]
    // type SchemaKind: TypedKind;

    // /// Marker type for exact `u32` value of the type's
    // /// [`Representation::REPR_KIND`], needed for internal blanket
    // /// implementations of various traits.
    // #[doc(hidden)]
    // type ReprKind: TypedKind;

    /// The type's `Select`able static field names and their IPLD Schema kinds.
    #[doc(hidden)]
    const FIELDS: &'static [&'static str] = &[];

    ///
    #[doc(hidden)]
    const __SERDE_SCHEMA_KIND: Kind = match Self::SCHEMA_KIND.is_copy() {
        Some(raw_sk) => raw_sk,
        None => Self::SCHEMA_KIND,
    };

    ///
    #[doc(hidden)]
    const __SERDE_REPR_KIND: Kind = match Self::REPR_KIND {
        Kind::Int => Int::REPR_KIND,
        Kind::Float => Float::REPR_KIND,
        rk => rk,
    };

    ///
    /// ? The name of the reified type?
    /// for unions, this ?should delegate to the variant's type name'
    fn name(&self) -> &'static str {
        Self::NAME
    }

    ///
    fn has_links(&self) -> bool {
        Self::HAS_LINKS
    }

    ///
    fn as_field(&self) -> Option<Field<'_>> {
        None
    }

    ///
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
    fn serialize<const MC: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
    fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // <Self as Deserialize<'de>>::deserialize(deserializer)
        // <Self as Select>::__select_de::<MC, D>(EmptySeed, deserializer)

        // deserialize_with_visitor::<MC, D, _, Self>(
        //     deserializer,
        //     Self::Walker::<'de, '_, MC>::default(),
        // )

        unimplemented!()
    }

    // AstWalk<'a, Ctx, Self>
    // ///
    // #[doc(hidden)]
    // type Walker<'de, 'a: 'de, const MC: u64>: Walk<'de, 'a, MC, Self>;

    #[inline]
    #[doc(hidden)]
    fn deserialize_with_visitor<'de, const MC: u64, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: LinkVisitor<'de, MC>,
    {
        deserialize_with_visitor::<MC, D, V, Self>(deserializer, visitor)
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

#[inline]
#[doc(hidden)]
fn deserialize_with_visitor<'de, const MC: u64, D, V, T>(
    deserializer: D,
    visitor: V,
) -> Result<V::Value, D::Error>
where
    D: Deserializer<'de>,
    V: LinkVisitor<'de, MC>,
    T: Representation,
{
    match (T::__SERDE_SCHEMA_KIND, T::__SERDE_REPR_KIND) {
        _ if T::REPR_STRATEGY.is_ignored() => deserializer.deserialize_ignored_any(visitor),
        _ if T::DATA_MODEL_KIND.is_option() => deserializer.deserialize_option(visitor),

        // union keyed (ref), envelope (tag + ref), inline (tag + inline)
        (Kind::Union, Kind::Map) => deserializer.deserialize_enum(T::NAME, T::FIELDS, visitor),
        // union (kinded => any, stringprefix => str, bytesprefix => bytes)

        // struct map
        (Kind::Struct, Kind::Map) => deserializer.deserialize_struct(T::NAME, T::FIELDS, visitor),
        // struct tuple, listpairs
        (Kind::Struct, Kind::List) => {
            deserializer.deserialize_tuple_struct(T::NAME, T::FIELDS.len(), visitor)
        }
        // structs stringpair, stringjoin => Kind::String

        // enum
        (Kind::Enum, _) => deserializer.deserialize_identifier(visitor),

        // basic
        (_, Kind::Null) => deserializer.deserialize_unit(visitor),
        (_, Kind::Bool) => deserializer.deserialize_bool(visitor),
        (_, Kind::Int8) => deserializer.deserialize_i8(visitor),
        (_, Kind::Int16) => deserializer.deserialize_i16(visitor),
        (_, Kind::Int32) => deserializer.deserialize_i32(visitor),
        (_, Kind::Int64) => deserializer.deserialize_i64(visitor),
        (_, Kind::Int128) => deserializer.deserialize_i128(visitor),
        (_, Kind::Uint8) => deserializer.deserialize_u8(visitor),
        (_, Kind::Uint16) => deserializer.deserialize_u16(visitor),
        (_, Kind::Uint32) => deserializer.deserialize_u32(visitor),
        (_, Kind::Uint64) => deserializer.deserialize_u64(visitor),
        (_, Kind::Uint128) => deserializer.deserialize_u128(visitor),
        (_, Kind::Float32) => deserializer.deserialize_f32(visitor),
        (_, Kind::Float64) => deserializer.deserialize_f64(visitor),
        (_, Kind::String) => deserializer.deserialize_str(visitor),
        (_, Kind::Bytes) => Multicodec::deserialize_bytes::<MC, _, _>(deserializer, visitor),
        (_, Kind::List) => deserializer.deserialize_seq(visitor),
        (_, Kind::Map) => deserializer.deserialize_map(visitor),
        (_, Kind::Link) => Multicodec::deserialize_link::<MC, _, _>(deserializer, visitor),
        // anything else
        _ => Multicodec::deserialize_any::<MC, _, _>(deserializer, visitor),
    }
}

///
pub trait StringRepresentation
where
    // Self: Representation<ReprKind = type_kinds::String>,
    Self: Representation,
    Self: Clone + FromStr + fmt::Display + Ord,
    <Self as FromStr>::Err: fmt::Display,
{
}
impl<T> StringRepresentation for T
where
    // T: Representation<ReprKind = type_kinds::String>,
    Self: Representation,
    T: Clone + FromStr + fmt::Display + Ord,
    <T as FromStr>::Err: fmt::Display,
{
}

///
pub trait BytesRepresentation
where
    // Self: Representation<ReprKind = type_kinds::Bytes>
    Self: Representation,
{
}

///
pub trait AdvancedRepresentation<Ctx: Context>: Representation + Select<Ctx> {}

///
/// TODO: possibly look at erased-serde to complete this "hack"
pub(crate) trait ErasedRepresentation: Downcast {
    // /// The underlying [`Representation`] type this type will downcast to.
    // type Representation: Representation = Self;

    ///
    fn name(&self) -> &'static str;

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
        Representation::name(self)
    }

    fn has_links(&self) -> bool {
        Representation::has_links(self)
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

// /// Helper fn for constraining and safely transmuting a generic selection output
// pub(crate) fn type_cast_selection<T: Sized + 'static, U: Sized + 'static, E, F>(
//     inner: F,
// ) -> Result<Option<U>, E>
// where
//     F: FnOnce() -> Result<Option<T>, E>,
// {
//     // if !type_eq::<T, U>() {
//     //     unreachable!("should only do this for types known to be identical")
//     // }
//
//     let mut inner = inner()?;
//     let outer = (&mut inner as &mut dyn std::any::Any)
//         .downcast_mut::<Option<U>>()
//         .unwrap()
//         .take();
//     Ok(outer)
// }
