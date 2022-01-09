//! IPLD codec interfaces.

#[cfg(feature = "dag-cbor")]
pub mod dag_cbor;
#[cfg(feature = "dag-json")]
pub mod dag_json;

use crate::dev::*;
use serde::{de, ser};
use std::{convert::TryFrom, error::Error as StdError};

/// An IPLD
/// [Codec](https://github.com/ipld/specs/blob/master/block-layer/codecs/README.md)
/// for reading and writing blocks.
/// TODO const generic over CODE?
pub trait Codec: Into<u64> + TryFrom<u64> + Copy {
    /// The multicodec code that identifies this IPLD Codec.
    const CODE: u64;

    /// Given a dag and a `Write`, encode it to the writer.
    fn write<T, W>(dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write;

    /// Given some bytes, deserialize a dag.
    fn decode<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation;

    /// Given a `Read`, deserialize a dag.
    fn read<T, R>(reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read;
}

// ///
// #[derive(Debug, thiserror::Error)]
// pub enum Error {
//     #[error(transparent)]
//     Encode(anyhow::Error),
//     #[error(transparent)]
//     Decode(anyhow::Error),
// }

/// The IPLD and Serde data models do not map 1:1. As a result, Serde may
/// encounter types that require special handling when serializing (i.e. bytes
/// and links).
pub trait Encoder: Serializer {
    /// Serialize a sequence of bytes.
    ///
    /// Because some codecs are text-based rather than binary, `Codec`s may define
    /// custom default behaviour for serializing bytes.
    fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error>;

    /// Serialize an IPLD link.
    /// TODO: cid_bytes?
    fn serialize_link<S>(self, cid: &CidGeneric<S>) -> Result<Self::Ok, Self::Error>
    where
        S: MultihashSize;
    // fn serialize_link(self, cid_bytes: Box<[u8]>) -> Result<Self::Ok, Self::Error>;
}

/// The IPLD and Serde data models do not map 1:1. As a result, Serde may
/// encounter types that are not equivalent in IPLD (such as byte and link maps
/// in DagJSON), or types it cannot handle altogether (such as IPLD links).
pub trait Decoder<'de>: Deserializer<'de> {
    /// Deserialize any IPLD data type, mapping any encountered Serde type to the
    /// appropriate `Visitor` or `IpldVisitorExt` method.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: IpldVisitorExt<'de>;

    /// Deserialize a sequence of borrowed bytes.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        // TODO: get rid of this extra trait bound
        V: IpldVisitorExt<'de>;

    /// Deserialize a sequence of bytes.
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        // TODO: get rid of this extra trait bound
        V: IpldVisitorExt<'de>;

    /// Deserialize an IPLD link.
    fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: IpldVisitorExt<'de>;
}

/// A helper trait for visiting special and recursive IPLD types.
///
/// Should be implemented by any types representing IPLD links and maps.
pub trait IpldVisitorExt<'de>: Visitor<'de> {
    /// The input contains the bytes of a `Cid`.
    ///
    /// The default implementation fails with a type error.
    /// TODO: vec or box?
    fn visit_link<E>(self, cid_bytes: Box<[u8]>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(de::Error::invalid_type(de::Unexpected::Other("CID"), &self))
    }
}

///
/// TODO: potentially get rid of this, in order to support raw JSON and CBOR codecs
mod specialization {
    use crate::dev::*;
    use serde::de;

    /// Default (specialized) implementation for all `Serializer`s (to avoid having
    /// to introduce an extension to `Serialize`).
    impl<S: Serializer> Encoder for S {
        /// Default behaviour is to delegate to `Serializer::serialize_bytes`.
        #[inline]
        default fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
            Serializer::serialize_bytes(self, bytes)
        }

        /// Default behaviour is to serialize the link directly as bytes.
        #[inline]
        default fn serialize_link<Si>(self, cid: &CidGeneric<Si>) -> Result<Self::Ok, Self::Error>
        where
            Si: MultihashSize,
        {
            Serializer::serialize_bytes(self, cid.to_bytes().as_ref())
        }
    }

    /// Default (specialized) implementation for all `Deserializer`s (to avoid
    /// having to introduce an extension to `Deserialize`).
    impl<'de, D: Deserializer<'de> + Sized> Decoder<'de> for D {
        /// Default behaviour is to delegate directly to `Deserializer::deserialize_any`.
        #[inline]
        default fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            Deserializer::deserialize_any(self, visitor)
        }

        /// Default behaviour is to delegate directly to
        /// `Deserializer::deserialize_bytes`.
        #[inline]
        default fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            Deserializer::deserialize_bytes(self, visitor)
        }

        /// Default behaviour is to delegate directly to
        /// `Deserializer::deserialize_byte_buf`.
        #[inline]
        default fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            Deserializer::deserialize_byte_buf(self, visitor)
        }

        /// Default behaviour is to deserialize some bytes and parse them directly
        /// as a `Cid`.
        #[inline]
        default fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            let bytes = <&'de [u8]>::deserialize(self)?;
            visitor.visit_link(bytes.into())
        }
    }
}

pub(crate) mod test_utils {
    use crate::dev::*;
    use std::{fmt::Debug, io::Read, string::ToString};

    pub fn roundtrip_bytes_codec<'de, C, T>(cases: &[(T, &'de [u8])])
    where
        C: Codec,
        T: PartialEq + Debug + Representation,
    {
        for (ref dag, expected) in cases {
            // writing
            let bytes = write_to_bytes::<C, T>(dag).expect(&format!(
                "Failed to encode `{}` {:?} into {:?}",
                dag.name(),
                dag,
                expected,
            ));
            assert_eq!(expected, &bytes.as_slice(), "Writing failure");

            // decoding
            let v = decode_from_bytes::<C, T>(expected).expect(&format!(
                "Failed to decode `{}` from {:?}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Decoding failure");

            // reading
            let v = C::read(*expected).expect(&format!(
                "Failed to read `{}` from {:?}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Reading failure");
        }
    }

    pub fn roundtrip_str_codec<'de, C, T>(cases: &[(T, &'de str)])
    where
        C: Codec,
        T: PartialEq + Debug + Representation,
    {
        for (ref dag, expected) in cases {
            // writing
            let string = write_to_str::<C, T>(dag).expect(&format!(
                "Failed to encode `{}` {:?} into {}",
                dag.name(),
                dag,
                expected,
            ));
            assert_eq!(*expected, string.as_str(), "Writing failure");

            // decoding
            let v = decode_from_str::<C, T>(expected).expect(&format!(
                "Failed to decode `{}` from {}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Decoding failure");

            // reading
            let v = C::read(expected.as_bytes()).expect(&format!(
                "Failed to read `{}` from {}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Reading failure");
        }
    }

    fn write_to_bytes<C, T>(dag: &T) -> Result<Vec<u8>, Error>
    where
        C: Codec,
        T: Representation,
    {
        let mut bytes = Vec::new();
        C::write(dag, &mut bytes)?;
        Ok(bytes)
    }

    fn write_to_str<C, T>(dag: &T) -> Result<String, Error>
    where
        C: Codec,
        T: Representation,
    {
        let bytes = write_to_bytes::<C, T>(dag)?;
        Ok(String::from_utf8(bytes).unwrap())
    }

    fn decode_from_bytes<'de, C, T>(bytes: &'de [u8]) -> Result<T, Error>
    where
        C: Codec,
        T: Representation,
    {
        C::decode(bytes)
    }

    fn decode_from_str<'de, C, T>(s: &'de str) -> Result<T, Error>
    where
        C: Codec,
        T: Representation,
    {
        C::decode(s.as_bytes())
    }
}
