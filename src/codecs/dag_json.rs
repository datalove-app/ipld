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
            _ => Err(Error::UnknownCodec(code)),
        }
    }
}

impl Codec for DagJson {
    const CODE: u64 = 0x0129;

    fn write<T, W>(dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation + Serialize,
        W: Write,
    {
        to_writer(writer, dag).map_err(|e| Error::Encoder(anyhow::Error::new(e)))
    }

    fn decode<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation + Deserialize<'de>,
    {
        from_slice(bytes).map_err(|e| Error::Decoder(anyhow::Error::new(e)))
    }

    fn read<T, R>(reader: R) -> Result<T, Error>
    where
        T: Representation + for<'de> Deserialize<'de>,
        R: Read,
    {
        from_reader(reader).map_err(|e| Error::Decoder(anyhow::Error::new(e)))
    }
}

impl<'a, W: Write> Encoder for &'a mut JsonSerializer<W> {
    /// Serializes bytes as a struct variant, e.g.
    /// `{ "/": { "base64": <some base64-encoded string> } }`.
    #[inline]
    fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, JsonError> {
        use ser::SerializeStructVariant as SV;

        let mut sv = self.serialize_struct_variant("", 0, "/", 1)?;
        SV::serialize_field(&mut sv, "bytes", &multibase::encode(DEFAULT_MB, bytes))?;
        SV::end(sv)
    }

    /// Serializes links as a newtype variant, e.g.  `{ "/": "Qm..." }`.
    #[inline]
    fn serialize_link<S>(self, cid: &CidGeneric<S>) -> Result<Self::Ok, JsonError>
    where
        S: MultihashSize,
    {
        let mb = match cid.version() {
            cid::Version::V0 => Multibase::Base58Btc,
            cid::Version::V1 => Multibase::Base32Lower,
        };
        let cid_str = multibase::encode(mb, cid.to_bytes());
        self.serialize_newtype_variant("", 0, "/", &cid_str)
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
        Deserializer::deserialize_any(self, JsonVisitor(visitor))
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Decoder::deserialize_byte_buf(self, visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Deserializer::deserialize_map(self, JsonVisitor(visitor))
    }

    #[inline]
    fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, JsonError>
    where
        V: IpldVisitorExt<'de>,
    {
        Deserializer::deserialize_map(self, JsonVisitor(visitor))
    }
}

/// `JsonVisitor` wraps an "any" type `Visitor` in order to enhance how maps are
/// deserialized.
struct JsonVisitor<V>(V);

impl<'de, V: IpldVisitorExt<'de>> Visitor<'de> for JsonVisitor<V> {
    type Value = V::Value;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        if let Some("/") = first_key.as_deref() {
            match map.next_value::<MapLikeVisitor>() {
                Ok(MapLikeVisitor::Bytes(b)) => self.0.visit_byte_buf(b),
                Ok(MapLikeVisitor::Cid(b)) => self.0.visit_link(b),
                _ => Err(de::Error::custom("expected a CID or byte string")),
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

/// Visits the IPLD types in DagJSON that are map-like (i.e. bytes and links).
#[derive(Debug)]
enum MapLikeVisitor {
    Default,
    Bytes(Vec<u8>),
    Cid(Box<[u8]>),
}

impl<'de> Deserialize<'de> for MapLikeVisitor {
    /// Will either deserialize a string (as a link), or a map (as bytes) -
    /// anything else is an error.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MapLikeVisitor::Default)
    }
}

impl<'de> Visitor<'de> for MapLikeVisitor {
    type Value = Self;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a JSON map, link object or byte object")
    }

    /// TODO:
    /// In the dag-json codec, links are represented as either Base58 or Base32
    /// strings
    #[inline]
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // TODO
        // let cid = ToCid::to_cid(s).map_err(|_| de::Error::custom("expected a CID"))?;
        // Ok(MapLikeVisitor::Cid(cid))
        unimplemented!()
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
        // TODO: why do these have to be Strings instead of &str?
        let (base, byte_str): (String, String) = map.next_entry()?.ok_or_else(|| {
            de::Error::custom("expected a base string + multibase-encoded string key-value pair")
        })?;

        if base.as_str() != "bytes" {
            return Err(de::Error::custom(
                "DagJSON only supports base64-encoded strings",
            ));
        }

        let (mb, bytes) = multibase::decode(byte_str)
            .map_err(|_| de::Error::custom("expected a base64 multibase-encoded string"))?;

        match mb {
            DEFAULT_MB => Ok(MapLikeVisitor::Bytes(bytes)),
            _ => Err(de::Error::custom(
                "DagJSON only supports base64-encoded strings",
            )),
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

#[cfg(test)]
mod tests {
    use crate::{_codecs::test_utils::*, prelude::*};
    use std::str::FromStr;

    fn roundtrip<'de, T>(cases: &[(T, &'de str)])
    where
        T: PartialEq + Debug + Representation + Serialize + DeserializeOwned,
    {
        roundtrip_str_codec::<DagJson, T>(cases)
    }

    #[test]
    fn test_null() {
        let tests = &[((), "null")];
        roundtrip(tests);
        let tests = &[(None as Option<Int>, "null")];
        roundtrip(tests);
        let tests = &[(Null, "null")];
        roundtrip(tests);
    }

    #[test]
    fn test_bool() {
        let tests = &[(true, "true"), (false, "false")];
        roundtrip(tests);
    }

    #[test]
    fn test_number() {
        let tests = &[(123, "123")];
        roundtrip(tests);
        let tests = &[(123.123, "123.123")];
        roundtrip(tests);
    }

    #[test]
    fn test_string() {
        let tests = &[(String::from("hello world"), "\"hello world\"")];
        roundtrip(tests);
    }

    #[test]
    fn test_bytes() {
        let tests = &[(
            Bytes::from(vec![0x01, 0x02, 0x03]),
            r#"{"/":{"bytes":"mAQID"}}"#,
        )];
        roundtrip(tests);
    }

    #[test]
    fn test_link() {
        let s = String::from("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n");
        let cid = CidGeneric::<typenum::U32>::from_str(&s).unwrap();
        let json = format!("{{\"/\":\"{}\"}}", s);

        let tests = &[(Link::<Null, typenum::U32>::from(cid), json.as_str())];
        roundtrip(tests);
    }

    #[test]
    fn test_seq() {}

    #[test]
    fn test_map() {}
}
