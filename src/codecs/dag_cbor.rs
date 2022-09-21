//! IPLD DagCbor codec.

use crate::dev::*;
use serde::de;
use serde_cbor::{
    de::{IoRead, Read as CborRead, SliceRead},
    ser::{IoWrite, Write as CborWrite},
    tags::Tagged,
    Deserializer as CborDeserializer, Error as CborError, Serializer as CborSerializer,
};
use std::{
    convert::TryFrom,
    fmt,
    io::{Read, Write},
};

/// The [DagCBOR](https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md) codec, that delegates to `serde_cbor`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DagCbor;

impl DagCbor {
    /// The multicodec code that identifies this IPLD Codec.
    pub const CODE: u64 = 0x71;

    /// The special tag signifying an IPLD link.
    pub const LINK_TAG: u64 = 42;

    #[doc(hidden)]
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    ///
    #[inline]
    pub(crate) fn serialize_link<S: Serializer>(
        cid: &Cid,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        Tagged::new(Some(Self::LINK_TAG), &cid.to_bytes()).serialize(serializer)
    }

    ///
    /// TODO:
    #[inline]
    pub(crate) fn deserialize_any<'de, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: IpldVisitorExt<'de>,
    {
        // deserializer.deserialize_any(visitor::DagJsonVisitor::<'a', _>(visitor))
        unimplemented!()
    }

    ///
    #[inline]
    pub(crate) fn deserialize_link<'de, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: IpldVisitorExt<'de>,
    {
        match Tagged::<&'de [u8]>::deserialize(deserializer)? {
            Tagged {
                tag: Some(Self::LINK_TAG),
                value,
            } => visitor.visit_link_borrowed_bytes(value),
            Tagged { tag: Some(tag), .. } => Err(D::Error::custom(format!(
                "unexpected CBOR tag for Cid: {}",
                tag
            ))),
            _ => Err(D::Error::custom("expected a Cid")),
        }
    }

    pub(crate) fn read_with_seed<'de, S, R>(&mut self, seed: S, reader: R) -> Result<(), Error>
    where
        S: CodecDeserializeSeed<'de>,
        R: Read,
    {
        let mut de = CborDeserializer::from_reader(reader);
        seed.deserialize::<{ Self::CODE }, _>(&mut de)
            .map_err(Error::decoder)
    }
}

impl Codec for DagCbor {
    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write,
    {
        let mut ser = CborSerializer::new(IoWrite::new(writer));
        Representation::serialize::<{ Self::CODE }, _>(dag, &mut ser).map_err(Error::encoder)
    }

    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        let mut de = CborDeserializer::new(SliceRead::new(bytes));
        Representation::deserialize::<{ Self::CODE }, _>(&mut de).map_err(Error::decoder)
    }

    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read,
    {
        let mut de = CborDeserializer::new(IoRead::new(reader));
        Representation::deserialize::<{ Self::CODE }, _>(&mut de).map_err(Error::decoder)
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
