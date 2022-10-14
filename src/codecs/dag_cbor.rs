//! IPLD DagCbor codec.

use crate::dev::*;
use serde_cbor::{
    de::{IoRead, SliceRead},
    ser::IoWrite,
    tags::Tagged,
    Deserializer as CborDeserializer, Serializer as CborSerializer,
};
// use cbor4ii::{
//     core::{
//         dec::{Decode, Read as CborRead},
//         enc::{Encode, Write as CborWrite},
//         types::{Bytes as CborBytes, Tag},
//         utils::{BufWriter, IoReader, IoWriter, SliceReader},
//     },
//     serde::{Deserializer as CborDeserializer, Serializer as CborSerializer},
// };
use delegate::delegate;
use maybestd::{
    borrow::Cow,
    convert::TryFrom,
    fmt,
    io::{BufReader, Read, Write},
};

/// The [DagCBOR](https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md) codec, that delegates to `serde_cbor`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DagCbor;

impl DagCbor {
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
        // EncodableCid(cid).serialize(serializer)
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
        V: LinkVisitor<'de>,
    {
        // deserializer.deserialize_any(DagCborVisitor::<'a', _>(visitor))
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
        V: LinkVisitor<'de>,
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

        // deserializer.deserialize_bytes(DagCborVisitor::<'l', _>(visitor))
    }

    #[doc(hidden)]
    #[inline]
    pub fn read_with_seed<Ctx, T, R>(seed: SelectorSeed<'_, Ctx, T>, reader: R) -> Result<(), Error>
    where
        Ctx: Context,
        T: Select<Ctx>,
        R: Read,
    {
        let mut de = CborDeserializer::from_reader(reader);
        T::__select_de::<{ Self::CODE }, _>(seed, &mut de).map_err(Error::decoder)
    }

    // pub(crate) fn deserializer_from_reader<R: Read>(
    //     &mut self,
    //     reader: R,
    // ) -> CborDeserializer<IoRead<R>> {
    //     CborDeserializer::from_reader(reader)
    // }

    // pub(crate) fn read_with_seed<'de, S, R>(
    //     &mut self,
    //     seed: S,
    //     reader: R,
    // ) -> Result<S::Value, Error>
    // where
    //     S: CodecDeserializeSeed<'de>,
    //     R: Read,
    // {
    //     let mut de = CborDeserializer::from_reader(reader);
    //     // let mut de = CborDeserializer::new(IoReader::new(BufReader::new(reader)));
    //     seed.deserialize::<{ Self::CODE }, _>(&mut de)
    //         .map_err(Error::decoder)
    // }

    // pub(crate) fn read_with_seed<'de, const D: bool, T, R>(
    //     &mut self,
    //     seed: T,
    //     reader: R,
    // ) -> Result<<CodecSeed<{ Self::CODE }, D, T> as DeserializeSeed<'de>>::Value, Error>
    // where
    //     // S: CodecDeserializeSeed<'de>,
    //     CodecSeed<{ Self::CODE }, D, T>: DeserializeSeed<'de>,
    //     R: Read,
    // {
    //     let mut de = CborDeserializer::from_reader(reader);
    //     // seed.deserialize::<{ Self::CODE }, _>(&mut de)
    //     // .map_err(Error::decoder)
    //     CodecSeed::<{ Self::CODE }, D, T>(seed)
    //         .deserialize(&mut de)
    //         .map_err(Error::decoder)
    // }
}

impl Codec for DagCbor {
    const NAME: &'static str = "dag-cbor";
    const CODE: u64 = 0x71;

    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write,
    {
        let mut ser = CborSerializer::new(IoWrite::new(writer));
        // let mut ser = CborSerializer::new(IoWriter::new(writer));
        Representation::serialize::<{ Self::CODE }, _>(dag, &mut ser).map_err(Error::encoder)
    }

    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        let mut de = CborDeserializer::new(SliceRead::new(bytes));
        // let mut de = CborDeserializer::new(SliceReader::new(bytes));
        Representation::deserialize::<{ Self::CODE }, _>(&mut de).map_err(Error::decoder)
    }

    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read,
    {
        let mut de = CborDeserializer::new(IoRead::new(reader));
        // let mut de = CborDeserializer::new(IoReader::new(BufReader::new(reader)));
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

// mod visitor {
//     use super::*;

//     pub struct DagCborVisitor<const T: char, V>(V);
//     impl<'de> Visitor<'de> for DagCborVisitor<'a', V>
// }

#[cfg(feature = "cbor4ii")]
use tag::*;
#[cfg(feature = "cbor4ii")]
mod tag {
    use super::*;

    // add a null byte to the cid bytes
    impl Encode for &Cid {
        fn encode<W: CborWrite>(
            &self,
            writer: &mut W,
        ) -> Result<(), cbor4ii::EncodeError<W::Error>> {
            writer.push(&[0x00])?;
            writer.push(&self.to_bytes())?;
            Ok(())
        }
    }

    pub struct EncodableCid<'a>(pub &'a Cid);
    impl<'a> Serialize for EncodableCid<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut bytes = BufWriter::new(Vec::with_capacity(Cid::SIZE + 8));
            Tag(DagCbor::LINK_TAG, self.0)
                .encode(&mut bytes)
                .map_err(S::Error::custom)?;

            serializer.serialize_bytes(bytes.buffer())
        }
    }

    pub struct DagCborVisitor<const T: char, V>(pub V);

    // visitor for any
    // impl<'de, V: IpldVisitorExt<'de>> Visitor<'de> for DagCborVisitor<'a', V> {
    //     type Value = V::Value;
    //     #[inline]
    //     fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         let tagged_bytes =
    //             Tag::<CborBytes<&[u8]>>::decode(&mut SliceReader::new(v)).map_err(E::custom)?;
    //         match tagged_bytes {
    //             // remove the first byte
    //             Tag(DagCbor::CODE, CborBytes(cid_bytes)) => {
    //                 self.0.visit_link_bytes(&cid_bytes[1..])
    //             }
    //             Tag(tag, _) => Err(E::custom(format!("unexpected CBOR tag for Cid: {}", tag))),
    //             _ => Err(E::custom("expected tagged bytes for a Cid")),
    //         }
    //     }
    //     // Some of these are not expected to be called, since the only data model
    //     // mis-match exists between Serde maps and IPLD maps.
    //     delegate! {
    //         to self.0 {
    //             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    //             fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E>;
    //             fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E>;
    //             fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E>;
    //             fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E>;
    //             fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E>;
    //             fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E>;
    //             fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E>;
    //             fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E>;
    //             fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E>;
    //             fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E>;
    //             fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E>;
    //             fn visit_f32<E: de::Error>(self, v: f32) -> Result<Self::Value, E>;
    //             fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E>;
    //             fn visit_char<E: de::Error>(self, v: char) -> Result<Self::Value, E>;
    //             fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E>;
    //             fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E>;
    //             fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E>;
    //             fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E>;
    //             fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E>;
    //             fn visit_none<E: de::Error>(self) -> Result<Self::Value, E>;
    //             fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error>;
    //             fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E>;
    //             fn visit_newtype_struct<D: Deserializer<'de>>(
    //                 self,
    //                 deserializer: D
    //             ) -> Result<Self::Value, D::Error>;
    //             fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error>;
    //             fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error>;
    //             fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error>;
    //         }
    //     }
    // }

    // visitor for links
    // TODO: does not work, as Cids are tagged differently than bytes
    impl<'de, V: LinkVisitor<'de>> Visitor<'de> for DagCborVisitor<'l', V> {
        type Value = V::Value;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "A tagged Cid")
        }
        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // TODO: use dec::Reference?
            let tagged_bytes =
                Tag::<CborBytes<&[u8]>>::decode(&mut SliceReader::new(v)).map_err(E::custom)?;
            match tagged_bytes {
                // remove the first byte
                Tag(DagCbor::CODE, CborBytes(cid_bytes)) => {
                    self.0.visit_link_bytes(&cid_bytes[1..])
                }
                Tag(tag, _) => Err(E::custom(format!("unexpected CBOR tag for Cid: {}", tag))),
                _ => Err(E::custom("expected tagged bytes for a Cid")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::prelude::*;

    #[test]
    fn test_null() {
        // let tests = &[(Null, "null")];
        // roundtrip_bytes_codec::<Null>(DagJson::CODE, tests);
    }

    #[test]
    fn test_bool() {}

    #[test]
    fn test_number() {
        // ints
        // // u8
        // let vec = to_vec(&24).unwrap();
        // assert_eq!(vec, b"\x18\x18");
        // // i8
        // let vec = to_vec(&-5).unwrap();
        // assert_eq!(vec, b"\x24");
        // // i16
        // let vec = to_vec(&-300).unwrap();
        // assert_eq!(vec, b"\x39\x01\x2b");
        // // i32
        // let vec = to_vec(&-23567997).unwrap();
        // assert_eq!(vec, b"\x3a\x01\x67\x9e\x7c");
        // // u64
        // let vec = to_vec(&::std::u64::MAX).unwrap();
        // assert_eq!(vec, b"\x1b\xff\xff\xff\xff\xff\xff\xff\xff");

        // floats
        let cases = &[(4000.5f32, b"\xfb\x40\xaf\x41\x00\x00\x00\x00\x00".as_ref())];
        roundtrip_bytes_codec::<f32>(DagCbor::CODE, cases);
        let cases = &[(12.3f64, b"\xfb@(\x99\x99\x99\x99\x99\x9a".as_ref())];
        roundtrip_bytes_codec::<Float>(DagCbor::CODE, cases);
    }

    #[test]
    fn test_string() {
        let cases = &[(IpldString::from("foobar"), b"ffoobar".as_ref())];
        roundtrip_bytes_codec::<IpldString>(DagCbor::CODE, cases);
    }

    #[test]
    fn test_bytes() {}

    #[test]
    fn test_link() {}

    #[test]
    fn test_seq() {
        let cases = &[(vec![1, 2, 3], b"\x83\x01\x02\x03".as_ref())];
        roundtrip_bytes_codec::<List<Int>>(DagCbor::CODE, cases)
    }

    #[test]
    fn test_map() {}
}
