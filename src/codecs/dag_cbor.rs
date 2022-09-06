//! IPLD DagCbor codec.

use crate::dev::*;
use serde::de;
use serde_cbor::{
    de::Read as CborRead,
    from_reader, from_slice,
    ser::Write as CborWrite,
    tags::{current_cbor_tag, Tagged},
    to_writer, Deserializer as CborDeserializer, Error as CborError, Serializer as CborSerializer,
};
use std::{
    convert::TryFrom,
    fmt,
    io::{Read, Write},
};

/// The magic tag signifying an IPLD link.
pub const CBOR_LINK_TAG: u64 = 42;

/// The [DagCBOR](https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md) codec, that delegates to `serde_cbor`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DagCbor;

impl DagCbor {
    /// The multicodec code that identifies this IPLD Codec.
    pub const CODE: u64 = 0x71;

    #[doc(hidden)]
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Into<u64> for DagCbor {
    fn into(self) -> u64 {
        Self::CODE
    }
}

impl TryFrom<u64> for DagCbor {
    type Error = Error;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            Self::CODE => Ok(Self),
            _ => Err(Error::UnknownMulticodecCode(code)),
        }
    }
}

impl Codec for DagCbor {
    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write,
    {
        to_writer(writer, dag).map_err(Error::encoder)
    }

    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        from_slice(bytes).map_err(Error::decoder)
    }

    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read,
    {
        from_reader(reader).map_err(Error::decoder)
    }

    /// Given a `Read`, deserialize a dag.
    fn read_with_seed<'de, S, R>(
        &mut self,
        seed: S,
        reader: R,
    ) -> Result<<S as DeserializeSeed<'de>>::Value, Error>
    where
        S: DeserializeSeed<'de>,
        R: Read,
    {
        let mut de = CborDeserializer::from_reader(reader);
        seed.deserialize(&mut de).map_err(Error::decoder)
    }
}

// impl<'a> CodecExt for DagCbor {
//     // type Encoder = &'a mut CborSerializer<W>

//     fn encoder<W: Write>(writer: W) -> Result<Self::Encoder, Error> {}
// }

impl<'a, W: CborWrite> Encoder for &'a mut CborSerializer<W> {
    #[inline]
    fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, CborError> {
        let bytes = cid.to_bytes();
        Tagged::new(Some(CBOR_LINK_TAG), bytes.as_slice()).serialize(self)
    }
}

impl<'de, 'a, R: CborRead<'de>> Decoder<'de> for &'a mut CborDeserializer<R> {
    #[inline]
    fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, CborError>
    where
        V: IpldVisitorExt<'de>,
    {
        struct ByteVec(Vec<u8>);
        struct ByteVecVisitor;
        impl<'de> Visitor<'de> for ByteVecVisitor {
            type Value = ByteVec;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A slice of bytes representing a Cid")
            }
            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ByteVec(v.into()))
            }
        }
        impl<'de> Deserialize<'de> for ByteVec {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_bytes(ByteVecVisitor)
            }
        }

        let Tagged { tag, value } = Tagged::<ByteVec>::deserialize(self)?;
        match tag {
            Some(CBOR_LINK_TAG) => visitor.visit_link_bytes(&value.0),
            Some(tag) => Err(CborError::custom(format!(
                "unexpected CBOR tag for CID: {}",
                tag
            ))),
            _ => Err(CborError::custom("expected a CID")),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_null() {}

    #[test]
    fn test_bool() {}

    #[test]
    fn test_number() {}

    #[test]
    fn test_string() {}

    #[test]
    fn test_bytes() {}

    #[test]
    fn test_link() {}

    #[test]
    fn test_seq() {}

    #[test]
    fn test_map() {}
}
