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
        // S: DeserializeSeed<'de, Value = ()>,
        // BlockSelectorSeed<C, S>: DeserializeSeed<'de, Value = ()>,
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

// impl<W: CborWrite> CodecExt<{ DagCbor::CODE }> for &mut CborSerializer<W> {}
// impl<'de, R: CborRead<'de>> CodecExt<{ DagCbor::CODE }> for &mut CborDeserializer<R> {}

#[cfg(feature = "specialization")]
mod specialization {
    use super::*;

    impl<'a, W: CborWrite> Encoder for &'a mut CborSerializer<W> {
        #[inline]
        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, CborError> {
            let bytes = cid.to_bytes();
            Tagged::new(Some(DagCbor::LINK_TAG), bytes.as_slice()).serialize(self)
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
                Some(tag) => Err(<CborError as de::Error>::custom(format!(
                    "unexpected CBOR tag for CID: {}",
                    tag
                ))),
                _ => Err(<CborError as de::Error>::custom("expected a CID")),
            }
        }
    }
}

#[cfg(feature = "autoref")]
mod autoref {
    use super::*;

    pub trait DagCborEncoder {
        type Ok;
        type Error;

        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error>;

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }

    impl<'a, W> DagCborEncoder for &'a mut &'a mut Encoder<&mut CborSerializer<W>>
    where
        W: CborWrite,
    {
        type Ok = ();
        type Error = CborError;

        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
            self.0.serialize_bytes(bytes)
        }

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            let bytes = cid.to_bytes();
            Tagged::new(Some(CBOR_LINK_TAG), bytes.as_slice()).serialize(&mut *self.0)
        }
    }

    pub trait DagCborDecoder<'de> {
        type Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>;

        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>;

        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>;

        fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>;
    }

    impl<'a, 'de, R> DagCborDecoder<'de> for &'a mut &'a mut Decoder<&mut CborDeserializer<R>>
    where
        R: CborRead<'de>,
    {
        type Error = CborError;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            self.0.deserialize_any(visitor)
        }

        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.0.deserialize_bytes(visitor)
        }

        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.0.deserialize_byte_buf(visitor)
        }

        fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
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

            let Tagged { tag, value } = Tagged::<ByteVec>::deserialize(&mut *self.0)?;
            match tag {
                Some(CBOR_LINK_TAG) => visitor.visit_link_bytes(&value.0),
                Some(tag) => Err(<Self::Error as de::Error>::custom(format!(
                    "unexpected CBOR tag for CID: {}",
                    tag
                ))),
                _ => Err(<Self::Error as de::Error>::custom("expected a CID")),
            }
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
