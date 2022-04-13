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
            _ => Err(Error::UnknownCodec(code)),
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
}

// impl<'a> CodecExt for DagCbor {
//     // type Encoder = &'a mut CborSerializer<W>

//     fn encoder<W: Write>(writer: W) -> Result<Self::Encoder, Error> {}
// }

impl<'a, W: CborWrite> Encoder for &'a mut CborSerializer<W> {
    #[inline]
    fn serialize_link<S>(self, cid: &CidGeneric<S>) -> Result<Self::Ok, CborError>
    where
        S: MultihashSize,
    {
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
        match current_cbor_tag() {
            Some(CBOR_LINK_TAG) => {
                // TODO:
                let bytes = <&[u8]>::deserialize(self)?;
                visitor.visit_link_bytes(bytes)
            }
            Some(tag) => Err(de::Error::custom(format!(
                "unexpected CBOR tag for CID: {}",
                tag
            ))),
            _ => Err(de::Error::custom("expected a CID")),
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
