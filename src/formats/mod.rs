//! IPLD codec interfaces.

#[cfg(feature = "dag-cbor")]
pub mod dag_cbor;
#[cfg(feature = "dag-json")]
pub mod dag_json;

use crate::prelude::*;
use serde::de;
use std::error::Error as StdError;

/// An IPLD Format.
pub trait Format {
    /// Version of CID used by this codec.
    const VERSION: cid::Version;

    /// Multicodec content type that identifies this IPLD Format.
    const CODEC: cid::Codec;

    // type Encoder = E;
    // type Decoder = D;

    // fn encoder<W: Write>(&self, writer: W) -> E;
    // fn decoder<'de, R: Read>(&self, reader: R) -> D;

    // /// Given a dag, serialize it to bytes.
    // fn encode<S>(dag: &S) -> Result<Box<[u8]>, Self::Error>
    // where
    //     S: Serialize;

    // /// Given some bytes, deserialize it to a dag.
    // fn decode<'de, D>(bytes: &'de [u8]) -> Result<D, Self::Error>
    // where
    //     D: Deserialize<'de>;

    // /// Given a dag and a `Write`, serialize it to the writer.
    // fn write<S, W>(dag: &S, writer: W) -> Result<(), Self::Error>
    // where
    //     S: Serialize,
    //     W: Write;

    // /// Given a `Read`, deserialize a dag.
    // fn read<D, R>(reader: R) -> Result<D, Self::Error>
    // where
    //     D: DeserializeOwned,
    //     R: Read;
}

/// The IPLD and Serde data models do not map 1:1. As a result, Serde may
/// encounter types that require special handling when serializing (i.e. bytes
/// and links).
pub trait Encoder: Serializer + Sized {
    /// Serialize a sequence of bytes.
    ///
    /// Because some codecs are text-based rather than binary, `Codec`s may define
    /// custom default behaviour for serializing bytes.
    fn serialize_bytes(
        self,
        bytes: &[u8],
    ) -> Result<<Self as Serializer>::Ok, <Self as Serializer>::Error>;

    /// Serialize an IPLD link.
    fn serialize_link(
        self,
        cid: &Cid,
    ) -> Result<<Self as Serializer>::Ok, <Self as Serializer>::Error>;
}

/// The IPLD and Serde data models do not map 1:1. As a result, Serde may
/// encounter types that are not equivalent in IPLD (such as byte and link maps
/// in DagJSON), or types it cannot handle altogether (such as IPLD links).
pub trait Decoder<'de>: Deserializer<'de> + Sized {
    /// Deserialize any IPLD data type, mapping any encountered Serde type to the
    /// appropriate `Visitor` or `IpldVisitorExt` method.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, <Self as Deserializer<'de>>::Error>
    where
        V: IpldVisitorExt<'de>;

    /// Deserialize a sequence of bytes.
    fn deserialize_bytes<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, <Self as Deserializer<'de>>::Error>
    where
        V: IpldVisitorExt<'de>;

    /// Deserialize an IPLD link.
    fn deserialize_link<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, <Self as Deserializer<'de>>::Error>
    where
        V: IpldVisitorExt<'de>;
}

/// A helper trait for visiting special and recursive IPLD types.
///
/// Should be implemented by any types representing IPLD links and maps.
pub trait IpldVisitorExt<'de>: Visitor<'de> {
    /// The input contains a `Cid`.
    ///
    /// The default implementation fails with a type error.
    fn visit_link<E>(self, _cid: Cid) -> Result<<Self as Visitor<'de>>::Value, E>
    where
        E: de::Error,
    {
        Err(de::Error::invalid_type(de::Unexpected::Other("CID"), &self))
    }
}

mod specialization {
    use crate::prelude::*;
    use serde::de;

    /// Default (specialized) implementation for all `Deserializer`s (to avoid
    /// having to introduce an extension to `Deserialize`).
    impl<'de, D: Deserializer<'de> + Sized> Decoder<'de> for D {
        /// Default behaviour is to delegate directly to `Deserializer::deserialize_any`.
        #[inline]
        default fn deserialize_any<V>(
            self,
            visitor: V,
        ) -> Result<V::Value, <Self as Deserializer<'de>>::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            Deserializer::deserialize_any(self, visitor)
        }

        /// Default behaviour is to delegate directly to
        /// `Deserializer::deserialize_bytes`.
        #[inline]
        default fn deserialize_bytes<V>(
            self,
            visitor: V,
        ) -> Result<V::Value, <Self as Deserializer<'de>>::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            Deserializer::deserialize_bytes(self, visitor)
        }

        /// Default behaviour is to deserialize some bytes and parse them directly
        /// as a `Cid`.
        #[inline]
        default fn deserialize_link<V>(
            self,
            visitor: V,
        ) -> Result<V::Value, <Self as Deserializer<'de>>::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            let bytes = <&[u8]>::deserialize(self)?;
            let cid = ToCid::to_cid(bytes).or(Err(de::Error::custom("expected a CID")))?;
            visitor.visit_link(cid)
        }
    }

    /// Default (specialized) implementation for all `Serializer`s (to avoid having
    /// to introduce an extension to `Serialize`).
    impl<S: Serializer> Encoder for S {
        /// Default behaviour is to delegate to `Serializer::serialize_bytes`.
        #[inline]
        default fn serialize_bytes(
            self,
            bytes: &[u8],
        ) -> Result<<Self as Serializer>::Ok, <Self as Serializer>::Error> {
            Serializer::serialize_bytes(self, bytes)
        }

        /// Default behaviour is to serialize the link directly as bytes.
        #[inline]
        default fn serialize_link(
            self,
            cid: &Cid,
        ) -> Result<<Self as Serializer>::Ok, <Self as Serializer>::Error> {
            Serializer::serialize_bytes(self, cid.to_bytes().as_ref())
        }
    }
}
