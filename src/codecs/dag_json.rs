//! IPLD DagJson codec.

use crate::dev::*;
use delegate::delegate;
use serde::{de, ser};
#[cfg(not(feature = "simd"))]
use serde_json::{
    de::Read as JsonRead, from_reader, from_slice, to_writer, Deserializer as JsonDeserializer,
    Error as JsonError, Serializer as JsonSerializer,
};
// #[cfg(feature = "simd")]
// use simd_json::{};
use std::convert::TryFrom;

/// All bytes are encoded with the multibase `base64` w/o padding, resulting
/// in the prefix `"m"`.
pub const DEFAULT_MB: Multibase = Multibase::Base64;

// TODO: add support for simd-json
#[cfg(not(feature = "simd"))]
/// The [DagJSON](https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-json.md) codec, that delegates to `serde_json`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DagJson;

impl DagJson {
    /// The multicodec code that identifies this IPLD Codec.
    pub const CODE: u64 = 0x0129;
}

impl Into<u64> for DagJson {
    fn into(self) -> u64 {
        Self::CODE
    }
}

impl TryFrom<u64> for DagJson {
    type Error = Error;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            Self::CODE => Ok(Self),
            _ => Err(Error::UnknownMulticodec(code)),
        }
    }
}

impl Codec for DagJson {
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
        let mut de = JsonDeserializer::from_reader(reader);
        seed.deserialize(&mut de).map_err(Error::decoder)
    }
}

impl<'a, W: Write> Encoder for &'a mut JsonSerializer<W> {
    /// Serializes bytes as a struct variant, e.g.
    /// `{ "/": { "base64": <some base64-encoded string> } }`.
    #[inline]
    fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, JsonError> {
        use ser::SerializeStructVariant;

        let mut sv = self.serialize_struct_variant("", 0, "/", 1)?;
        sv.serialize_field("bytes", &multibase::encode(DEFAULT_MB, bytes))?;
        sv.end()
    }

    /// Serializes links as a newtype variant, e.g.  `{ "/": "Qm..." }`.
    #[inline]
    fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, JsonError> {
        self.serialize_newtype_variant("", 0, "/", &cid.to_string())
    }
}

impl<'de, 'a, R: JsonRead<'de>> Decoder<'de> for &'a mut JsonDeserializer<R> {
    /// In the DagJSON IPLD codec, three IPLD types map to the map as represented
    /// in the Serde data model:
    ///     - maps
    ///     - Base58Btc- or Base32Lower-encoded links, e.g. `{ "/": "Qm..." }`
    ///     - Base64-encoded byte sequences, e.g. `{ "/": { "bytes": "m..." } }`
    ///
    /// This method wraps the provided `Visitor`, delegating the visiting of all
    /// types found in the input data to the provided `Visitor` (except for maps,
    /// which are handled separately as they may be IPLD bytes, links or actual maps.
    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Deserializer::deserialize_any(self, visitor::JsonVisitor(visitor))
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Deserializer::deserialize_map(self, visitor::JsonVisitor(visitor))
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Deserializer::deserialize_map(self, visitor::JsonVisitor(visitor))
    }

    #[inline]
    fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Deserializer::deserialize_map(self, visitor::JsonVisitor(visitor))
    }
}

mod visitor {
    use super::*;

    /// `JsonVisitor` wraps an "any" type `Visitor` in order to enhance how maps are
    /// deserialized.
    pub struct JsonVisitor<V>(pub V);

    impl<'de, V: IpldVisitorExt<'de>> Visitor<'de> for JsonVisitor<V> {
        type Value = V::Value;

        #[inline]
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a JSON map, link object or byte object")
        }

        /// Called when a map is found in the input data.
        /// TODO: test that this works with links union-inlined into structs/maps
        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let first_key: Option<String> = map.next_key()?;
            if Some("/") == first_key.as_deref() {
                match map.next_value::<MapLikeVisitor<'de>>() {
                    Ok(MapLikeVisitor::Bytes(b)) => self.0.visit_byte_buf(b),
                    Ok(MapLikeVisitor::Cid(s)) => self.0.visit_link_str(s),
                    Ok(MapLikeVisitor::CidStr(s)) => self.0.visit_link_str(&s),
                    err => Err(de::Error::custom(format!(
                        "expected a CID or byte string: {:?}",
                        err
                    ))),
                }
            } else {
                self.0.visit_map(MapAccessor { first_key, map })
            }
        }

        // Some of these are not expected to be called, since the only data model
        // mis-match exists between Serde maps and IPLD maps.
        delegate! {
            to self.0 {
                fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E>;
                fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E>;
                fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E>;
                fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E>;
                fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E>;
                fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E>;
                fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E>;
                fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E>;
                fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E>;
                fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E>;
                fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E>;
                fn visit_f32<E: de::Error>(self, v: f32) -> Result<Self::Value, E>;
                fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E>;
                fn visit_char<E: de::Error>(self, v: char) -> Result<Self::Value, E>;
                fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E>;
                fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E>;
                fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E>;
                fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E>;
                fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E>;
                fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E>;
                fn visit_none<E: de::Error>(self) -> Result<Self::Value, E>;
                fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error>;
                fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E>;
                fn visit_newtype_struct<D: Deserializer<'de>>(
                    self,
                    deserializer: D
                ) -> Result<Self::Value, D::Error>;
                fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error>;
                fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error>;
            }
        }
    }

    /// Wraps a `MapAccess` thats had it's first key removed.
    struct MapAccessor<A> {
        first_key: Option<String>,
        map: A,
    }

    impl<'de, A: de::MapAccess<'de>> de::MapAccess<'de> for MapAccessor<A> {
        type Error = A::Error;

        #[inline]
        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: de::DeserializeSeed<'de>,
        {
            use de::IntoDeserializer;

            if self.first_key.is_some() {
                let first_key = self.first_key.take().unwrap();
                seed.deserialize(first_key.into_deserializer()).map(Some)
            } else {
                self.map.next_key_seed(seed)
            }
        }

        delegate! {
            to self.map {
                fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
                where
                    V: de::DeserializeSeed<'de>;
            }
        }
    }

    /// Visits the IPLD types in DagJSON that are map-like (i.e. bytes and links).
    #[derive(Debug)]
    enum MapLikeVisitor<'a> {
        Default,
        Bytes(Vec<u8>),
        Cid(&'a str),
        CidStr(String),
    }

    impl<'de> Deserialize<'de> for MapLikeVisitor<'de> {
        /// Will either deserialize a string (as a link), or a map (as bytes) -
        /// anything else is an error.
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(MapLikeVisitor::Default)
        }
    }

    impl<'de> Visitor<'de> for MapLikeVisitor<'de> {
        type Value = Self;

        #[inline]
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a JSON map, link object or byte object")
        }

        /// In the dag-json codec, links are represented as either Base58 or Base32
        /// strings
        #[inline]
        fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::Cid(s))
        }

        /// In the dag-json codec, links are represented as either Base58 or Base32
        /// strings
        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::CidStr(s.into()))
        }

        /// TODO:
        /// In the dag-json codec, bytes are represented as maps, with the key
        /// always being the string "bytes" and the value always being the bytes
        /// multibase-encoded as a string.
        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let (byte_key, byte_str): (String, String) = map.next_entry()?.ok_or_else(|| {
                de::Error::custom(
                    "expected a \"bytes\" string + multibase-encoded string key-value pair",
                )
            })?;

            if byte_key != "bytes" {
                return Err(de::Error::custom(
                    "DagJSON bytes key must be the string \"bytes\"",
                ));
            }

            match multibase::decode(byte_str) {
                Ok((mb, bytes)) if mb == DEFAULT_MB => Ok(MapLikeVisitor::Bytes(bytes)),
                _ => Err(de::Error::custom(
                    "DagJSON only supports bytes as base64-encoded strings",
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{_codecs::test_utils::*, prelude::*};
    use std::str::FromStr;

    #[test]
    fn test_null() {
        let tests = &[((), "null")];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
        let tests = &[(None as Option<Int>, "null")];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_bool() {
        let tests = &[(true, "true"), (false, "false")];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_number() {
        let tests = &[(123, "123")];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
        let tests = &[(123.123, "123.123")];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_string() {
        let tests = &[(String::from("hello world"), "\"hello world\"")];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_bytes() {
        let tests = &[(
            Bytes::from(vec![0x01, 0x02, 0x03]),
            r#"{"/":{"bytes":"mAQID"}}"#,
        )];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_link() {
        type TestLink = Link<()>;

        let s = String::from("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n");
        let json = format!("{{\"/\":\"{}\"}}", s);

        let tests = &[(
            TestLink::from(Cid::try_from(s.as_str()).unwrap()),
            json.as_str(),
        )];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_seq() {}

    #[test]
    fn test_map() {}
}
