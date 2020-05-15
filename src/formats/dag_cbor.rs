//! IPLD DagCbor codec.

use crate::dev::*;
use serde::de;
use serde_cbor::{
    de::Read as CborRead,
    from_reader,
    ser::Write as CborWrite,
    tags::{current_cbor_tag, Tagged},
    to_writer, Deserializer as CborDeserializer, Error as CborError, Serializer as CborSerializer,
};
use std::io::{Read, Write};

/// The magic tag signifying an IPLD link.
pub const CBOR_LINK_TAG: u64 = 42;

/// The DagCBOR codec, that delegates to `serde_cbor`.
pub struct DagCbor;

impl Format for DagCbor {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    // type Encoder<W: Write> = CborSerializer<W>;
    // type Decoder<R: Read> = CborDeserializer<R>;

    // fn encoder<W: Write>(writer: W) -> Self::Encoder<W> {
    //     unimplemented!()
    // }
    // fn decoder<'de, R: Read>(reader: R) -> Self::Decoder<R> {
    //     unimplemented!()
    // }

    // type Encoder = CborSerializer;
    // type Decoder = CborDeserializer;
    // type Error = CborError;

    // fn encoder<W: Write>(&self, writer: W) -> CborSerializer<W> {
    //     CborSerializer::from_writer(writer)
    // }
    // fn decoder<R: Read>(&self, reader: R) -> CborDeserializer<R> {
    //     CborSerializer::from_reader(reader)
    // }

    fn write<T, W>(dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation + Serialize,
        W: Write,
    {
        to_writer(writer, dag).map_err(|e| Error::Encoder(anyhow::Error::new(e)))
    }

    fn read<T, R>(reader: R) -> Result<T, Error>
    where
        T: Representation + for<'de> Deserialize<'de>,
        R: Read,
    {
        from_reader(reader).map_err(|e| Error::Decoder(anyhow::Error::new(e)))
    }
}

impl<'a, W: CborWrite> Encoder for &'a mut CborSerializer<W> {
    #[inline]
    fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, CborError> {
        let vec: Vec<u8> = cid.to_bytes();
        let bytes: &[u8] = vec.as_ref();
        Tagged::new(Some(CBOR_LINK_TAG), bytes).serialize(self)
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
                let bytes = <&'de [u8]>::deserialize(self)?;
                let cid = ToCid::to_cid(bytes)
                    .or::<CborError>(Err(de::Error::custom("expected a CID")))?;
                visitor.visit_link(cid)
            }
            Some(tag) => Err(de::Error::custom(format!("unexpected CBOR tag: {}", tag))),
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
