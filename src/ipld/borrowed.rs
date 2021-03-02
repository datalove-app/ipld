//! An IPLD type that can borrow most of it's contents from an underlying type.
//!
//! TODO: implement mutation methods, getters, etc - you know, fun stuff

// TODO: edit this
/// IPLD introduces two fundamental concepts, Data Types and Schemas/Representations.
/// In short, Data Types are a small set of common types (lists, maps, links, etc)
/// necessary for generally modelling Linked Data and can be serialized/deserialized
/// by an IPLD CodecExt. However, more advanced types might benefit from having schemas,
/// alternate serialization/deserialization behaviour, or runtime dependencies to
/// aid in verification when encoding/decoding the type to/from raw blocks.
///
/// This type fulfills the role of a low-allocation mapping between IPLD Schemas
/// /Representations and underlying Codecs by actually providing two mappings:
///     - one from IPLD Data type <-> IPLD CodecExt (via Serde data model)
///     - one from Rust types, schemas & representations <-> IPLD (via `TryFrom`)
/// In the first case, Serde only copies/borrows from the type on serialization,
/// and whenever possible provides borrowed types on deserialization. Likewise,
/// `TryFrom<Ipld>` and `TryInto<Ipld>` are implemented to borrow many of their
/// fields or lazily iterate over them.
//

///
#[cfg(feature = "graphql")]
#[path = "./juniper.rs"]
mod _juniper;

use crate::dev::*;
use serde::{
    de::{self, Visitor},
    serde_if_integer128, Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt};

/// An Ipld type that borrows most of its contents from an underlying native
/// type or `Deserializer`.
#[derive(Clone, Debug)]
pub enum Ipld<'a> {
    /// Represents the absence of a value or the value undefined.
    Null,
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an i8.
    Int8(i8),
    /// Represents an i16.
    Int16(i16),
    /// Represents an i32.
    Int32(i32),
    /// Represents an i64.
    Int64(i64),
    /// Represents an i128.
    Int128(i128),
    /// Represents an u8.
    Uint8(u8),
    /// Represents an u16.
    Uint16(u16),
    /// Represents an u32.
    Uint32(u32),
    /// Represents an u64.
    Uint64(u64),
    /// Represents an u128.
    Uint128(u128),
    /// Represents an f32.
    Float32(f32),
    /// Represents an f64.
    Float64(f64),
    /// Represents a borrowed UTF-8 string.
    Str(&'a str),
    /// Represents an allocated UTF-8 string.
    String(String),
    /// Represents a borrowed sequence of bytes.
    Bytes(&'a [u8]),
    /// Represents an allocated sequence of bytes.
    BytesBuf(Vec<u8>),
    /// Represents a list.
    List(Vec<Ipld<'a>>),
    /// Represents a map.
    Map(BTreeMap<&'a str, Ipld<'a>>),
    /// Represents a link to an Ipld node.
    Link(Cid),
}

/// `Serialize` implementation that delegates to an `Encoder` for bytes and links,
/// otherwise directly calls methods on the `Serializer`.
impl<'a> Serialize for Ipld<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Ipld::Null => serializer.serialize_unit(),
            Ipld::Bool(b) => serializer.serialize_bool(*b),
            Ipld::Int8(n) => serializer.serialize_i8(*n),
            Ipld::Int16(n) => serializer.serialize_i16(*n),
            Ipld::Int32(n) => serializer.serialize_i32(*n),
            Ipld::Int64(n) => serializer.serialize_i64(*n),
            Ipld::Int128(n) => serializer.serialize_i128(*n),
            Ipld::Uint8(n) => serializer.serialize_u8(*n),
            Ipld::Uint16(n) => serializer.serialize_u16(*n),
            Ipld::Uint32(n) => serializer.serialize_u32(*n),
            Ipld::Uint64(n) => serializer.serialize_u64(*n),
            Ipld::Uint128(n) => serializer.serialize_u128(*n),
            Ipld::Float32(n) => serializer.serialize_f32(*n),
            Ipld::Float64(n) => serializer.serialize_f64(*n),
            Ipld::Str(s) => serializer.serialize_str(s),
            Ipld::String(s) => serializer.serialize_str(&s),
            Ipld::Bytes(b) => <S as Encoder>::serialize_bytes(serializer, *b),
            Ipld::BytesBuf(b) => <S as Encoder>::serialize_bytes(serializer, &b),
            Ipld::Link(cid) => <S as Encoder>::serialize_link(serializer, cid),
            Ipld::List(vec) => serializer.collect_seq(vec),
            Ipld::Map(map) => serializer.collect_map(map),
        }
    }
}

/// `Deserialize` implementation that uses an `IpldVisitor` to visit Serde and
/// IPLD data types.
impl<'de> Deserialize<'de> for Ipld<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <D as Decoder>::deserialize_any(deserializer, IpldVisitor)
    }
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! visit_primitive {
    ($type:ty : $visit_fn:ident $member:ident) => {
        #[inline]
        fn $visit_fn<E>(self, value: $type) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ipld::$member(value))
        }
    };
    ($type:ty : $visit_fn:ident $member:ident $method:ident) => {
        #[inline]
        fn $visit_fn<E>(self, value: $type) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ipld::$member(value.$method()))
        }
    };
}

/// `Visitor` for `Deserialize`ing an `Ipld`.
struct IpldVisitor;

///
///
/// Because this type is deserialized with `deserialize_any`, the input data drives
/// deserialization. Ergo, we need to accommodate `Codec`s whose serialized IPLD
/// does not map 1:1 to the Serde data model cleanly (e.g. DagJSON links appear as
/// one-key maps). We do this by implementing `IpldVisitorExt` methods.
impl<'de> Visitor<'de> for IpldVisitor {
    type Value = Ipld<'de>;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid IPLD data type")
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Null)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_none()
    }

    visit_primitive!(bool : visit_bool Bool);
    visit_primitive!(i8 : visit_i8 Int8);
    visit_primitive!(i16 : visit_i16 Int16);
    visit_primitive!(i32 : visit_i32 Int32);
    visit_primitive!(i64 : visit_i64 Int64);
    visit_primitive!(u8 : visit_u8 Uint8);
    visit_primitive!(u16 : visit_u16 Uint16);
    visit_primitive!(u32 : visit_u32 Uint32);
    visit_primitive!(u64 : visit_u64 Uint64);
    visit_primitive!(f32 : visit_f32 Float32);
    visit_primitive!(f64 : visit_f64 Float64);

    serde_if_integer128! {
        visit_primitive!(i128 : visit_i128 Int128);
        visit_primitive!(u128 : visit_u128 Uint128);
    }

    visit_primitive!(&'de str : visit_borrowed_str Str);
    visit_primitive!(&str : visit_str String to_string);
    visit_primitive!(&'de [u8] : visit_borrowed_bytes Bytes);
    visit_primitive!(&[u8] : visit_bytes BytesBuf into);

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_link(self)
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec: Vec<Ipld<'de>> = if let Some(len) = seq.size_hint() {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        };

        while let Some(ipld) = seq.next_element()? {
            vec.push(ipld);
        }

        Ok(Ipld::List(vec))
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut btree: BTreeMap<&'de str, Ipld<'de>> = BTreeMap::new();

        while let Some((key, value)) = map.next_entry()? {
            btree.insert(key, value);
        }

        Ok(Ipld::Map(btree))
    }
}

impl<'de> IpldVisitorExt<'de> for IpldVisitor {
    #[inline]
    fn visit_link<E>(self, cid: Cid) -> Result<<Self as Visitor<'de>>::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Link(cid))
    }
}
