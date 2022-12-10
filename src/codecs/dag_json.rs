//! IPLD DagJson codec.

use crate::dev::*;
use delegate::delegate;
use serde::{de, ser};
#[cfg(not(feature = "simd"))]
use serde_json::{Deserializer as JsonDeserializer, Serializer as JsonSerializer};
// #[cfg(feature = "simd")]
// use simd_json::{Serializer as JsonSerializer, Deserializer as JsonDeserializer, Error as JsonError};
use maybestd::{
    borrow::Cow,
    convert::TryFrom,
    fmt,
    io::{Read, Write},
};

// TODO: add support for simd-json
#[cfg(not(feature = "simd"))]
/// The [DagJSON](https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-json.md) codec, that delegates to `serde_json`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DagJson;

impl DagJson {
    /// All bytes are encoded with the multibase `base64` w/o padding, resulting
    /// in the prefix `"m"`.
    pub const MB_BYTES: Multibase = Multibase::Base64;

    /// The map key marking the map value as IPLD bytes or an IPLD link.
    pub const IPLD_KEY: &'static str = "/";

    #[doc(hidden)]
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Serializes bytes as a struct variant, e.g. `{ "/": { "bytes":
    /// <base64-encoded string> } }`.
    #[inline]
    pub(crate) fn serialize_bytes<S: Serializer>(
        bytes: impl AsRef<[u8]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use ser::SerializeStructVariant;

        let mut sv = serializer.serialize_struct_variant("", 0, Self::IPLD_KEY, 1)?;
        sv.serialize_field("bytes", &multibase::encode(Self::MB_BYTES, bytes))?;
        sv.end()
    }

    /// Serializes links as a newtype variant, e.g. `{ "/": <base*-encoded Cid>
    /// }`.
    #[inline]
    pub(crate) fn serialize_link<S: Serializer>(
        cid: &Cid,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let cid_str = cid.to_string().map_err(S::Error::custom)?;
        serializer.serialize_newtype_variant("", 0, Self::IPLD_KEY, &cid_str)
    }

    /// Deserialize any IPLD data type, mapping any encountered Serde type to the
    /// appropriate `Visitor` or `IpldVisitorExt` method.
    #[inline]
    pub(crate) fn deserialize_any<'de, const MC: u64, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: LinkVisitor<'de, MC>,
    {
        debug_assert!(MC == <Self as Codec>::CODE);
        deserializer.deserialize_any(visitor::DagJsonVisitor(visitor))
    }

    ///
    #[inline]
    pub(crate) fn deserialize_bytes<'de, const MC: u64, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: LinkVisitor<'de, MC>,
    {
        debug_assert!(MC == <Self as Codec>::CODE);
        deserializer.deserialize_any(visitor::DagJsonVisitor(visitor))
    }

    ///
    #[inline]
    pub(crate) fn deserialize_link<'de, const MC: u64, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: LinkVisitor<'de, MC>,
    {
        debug_assert!(MC == <Self as Codec>::CODE);
        deserializer.deserialize_any(visitor::DagJsonVisitor(visitor))
    }

    #[doc(hidden)]
    #[inline]
    pub fn read_with_seed<Ctx, T, R>(seed: SelectorSeed<'_, Ctx, T>, reader: R) -> Result<(), Error>
    where
        Ctx: Context,
        T: Select<Ctx>,
        R: Read,
    {
        let mut de = JsonDeserializer::from_reader(reader);
        T::__select_de::<{ <Self as Codec>::CODE }, _>(seed, &mut de).map_err(Error::decoder)
    }
}

impl<T: Representation> Codec<T> for DagJson {
    const NAME: &'static str = "dag-json";
    const CODE: u64 = 0x0129;

    fn write<W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        W: Write,
    {
        let mut ser = JsonSerializer::new(writer);
        Representation::serialize::<{ <Self as Codec>::CODE }, _>(dag, &mut ser)
            .map_err(Error::encoder)
    }

    fn decode<'de>(&mut self, bytes: &'de [u8]) -> Result<T, Error> {
        let mut de = JsonDeserializer::from_slice(bytes);
        Representation::deserialize::<{ <Self as Codec>::CODE }, _>(&mut de).map_err(Error::decoder)
    }

    fn read<R>(&mut self, reader: R) -> Result<T, Error>
    where
        R: Read,
    {
        let mut de = JsonDeserializer::from_reader(reader);
        Representation::deserialize::<{ <Self as Codec>::CODE }, _>(&mut de).map_err(Error::decoder)
    }
}

impl TryFrom<u64> for DagJson {
    type Error = Error;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            <Self as Codec>::CODE => Ok(Self),
            _ => Err(Error::UnknownMulticodecCode(code)),
        }
    }
}

mod visitor {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct DagJsonVisitor<const MC: u64, V>(pub(crate) V);

    // visitor for any
    impl<'de, const MC: u64, V> Visitor<'de> for DagJsonVisitor<MC, V>
    where
        // V: LinkVisitor<'de, { <DagJson as Codec>::CODE }>,
        V: LinkVisitor<'de, MC>,
    {
        type Value = V::Value;
        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            /// Wraps a `MapAccess` thats had it's first key removed.
            struct MapAccessor<'a, A> {
                first_key: Option<Cow<'a, str>>,
                map: A,
            }

            impl<'de, A: de::MapAccess<'de>> de::MapAccess<'de> for MapAccessor<'de, A> {
                type Error = A::Error;

                #[inline]
                fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
                where
                    K: de::DeserializeSeed<'de>,
                {
                    use de::IntoDeserializer;

                    if let Some(first_key) = self.first_key.take() {
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

            let first_key: Option<Cow<'_, str>> = map.next_key()?;
            if Some(DagJson::IPLD_KEY) == first_key.as_deref() {
                match map.next_value::<MapLikeVisitor<'de>>()? {
                    MapLikeVisitor::Bytes(b) => self.0.visit_byte_buf(b),
                    MapLikeVisitor::CidStr(s) => self.0.visit_link_str(s),
                    MapLikeVisitor::CidString(s) => self.0.visit_link_str(&s),
                    _ => Err(A::Error::custom("expected a CID or byte string: {:?}")),
                }
            } else {
                self.0.visit_map(MapAccessor { first_key, map })
            }
        }

        // Some of these are not expected to be called, since the only data model
        // mis-match exists between Serde maps and IPLD maps.
        delegate! {
            to self.0 {
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
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

    /**************************************************************************/
    /**************************************************************************/

    /// Visits the IPLD types in DagJSON that are map-like (i.e. bytes and links).
    #[derive(Debug)]
    enum MapLikeVisitor<'a> {
        Default,
        Bytes(Vec<u8>),
        CidStr(&'a str),
        CidString(String),
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
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a JSON map, link object or byte object")
        }

        #[inline]
        fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::CidStr(s))
        }

        /// In the dag-json codec, CIDs are represented as
        /// [`multibase`]()-encoded strings.
        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::CidString(s.into()))
        }

        #[inline]
        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::CidString(s))
        }

        /// In the dag-json codec, bytes are represented as maps, with the key
        /// always being the string "bytes" and the value always being the bytes
        /// [`multibase`]()-encoded-encoded as a string.
        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            map.next_key::<Cow<'_, str>>()?
                .filter(|key| key == "bytes")
                .ok_or_else(|| {
                    A::Error::custom("DagJSON bytes key must be the string \"bytes\"")
                })?;

            // let byte_str = map
            //     .next_value_seed(DeserializeWrapper::<{ DagJson::CODE }, IpldString>::default())?;
            let byte_str = map.next_value::<Cow<'_, str>>()?;
            match multibase::decode(byte_str) {
                Ok((DagJson::MB_BYTES, bytes)) => Ok(MapLikeVisitor::Bytes(bytes)),
                // Ok((mb, _)) => Err(de::Error::custom(format!(
                //     "DagJSON only supports bytes as base64-encoded strings, found multibase {:?}",
                //     mb
                // ))),
                _ => Err(de::Error::custom(
                    "DagJSON only supports bytes as base64-encoded strings",
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{codecs_::test_utils::*, *};

    const C: u64 = <DagJson as Codec>::CODE;

    #[test]
    fn test_null() {
        let cases = &[(Null, "null")];
        roundtrip_str_codec::<C, _>(cases);
        let cases = &[(None, "null")];
        roundtrip_str_codec::<C, Option<Int>>(cases);

        let cases = &[(Null.into(), "null")];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_bool() {
        let cases = &[(true, "true"), (false, "false")];
        roundtrip_str_codec::<C, _>(cases);

        let cases = &[(true.into(), "true"), (false.into(), "false")];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_number() {
        // ints
        let cases = &[
            (123, "123"),
            (65535, "65535"),
            (i64::MAX, &i64::MAX.to_string()),
        ];
        roundtrip_str_codec::<C, Int>(cases);

        // floats
        let cases = &[(123.123, "123.123")];
        roundtrip_str_codec::<C, Float>(cases);

        // any
        let cases = &[
            (123.into(), "123"),
            (65535.into(), "65535"),
            (Int::MAX.into(), &Int::MAX.to_string()),
            (123.123.into(), "123.123"),
        ];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_string() {
        let cases = &[
            // standard string
            (String::from("hello world"), "\"hello world\""),
            (String::default(), "\"\""),
            // non-standard UTF-8 string TODO:
            // (String::from("ÅΩ"), "\"ÅΩ\""),
        ];
        roundtrip_str_codec::<C, String>(cases);

        let cases = &[
            // standard string
            (String::from("hello world").into(), "\"hello world\""),
            (String::default().into(), "\"\""),
            // non-standard UTF-8 string TODO:
            // (String::from("ÅΩ").into(), "\"ÅΩ\""),
        ];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_bytes() {
        let cases = &[
            (
                Bytes::from([1u8, 2, 3].as_ref()),
                r#"{"/":{"bytes":"mAQID"}}"#,
            ),
            // TODO: empty bytes
            // (Bytes::from(&[]), r#"{"/":{"bytes":"m"}}"#),
        ];
        roundtrip_str_codec::<C, Bytes>(cases);

        let cases = &[
            (
                Bytes::from([1u8, 2, 3].as_ref()).into(),
                r#"{"/":{"bytes":"mAQID"}}"#,
            ),
            // TODO: empty bytes
            // (Bytes::from(&[]), r#"{"/":{"bytes":"m"}}"#),
        ];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_list() {
        // raw types
        let cases = &[(vec![], "[]"), (vec![1, 2], "[1,2]")];
        roundtrip_str_codec::<C, List<i64>>(cases);

        // any types
        let cases = &[
            (vec![].into(), "[]"),
            (vec![1.into(), 2.into()].into(), "[1,2]"),
            // (
            //     vec![Any::Link(Link::Cid(Default::default()).into())],
            //     "[{\"/\": }]",
            // ),
            (
                vec![vec![].into(), vec![Any::Int(1), Any::Float(123.123)].into()].into(),
                "[[],[1,123.123]]",
            ),
        ];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_map() {
        // raw types
        let cases = &[
            (Default::default(), "{}"),
            (
                {
                    let mut map: Map<String, i64> = Default::default();
                    map.insert("abc".into(), 123i64);
                    map
                },
                "{\"abc\":123}",
            ),
        ];
        roundtrip_str_codec::<C, Map<String, i64>>(cases);

        // any types
        let cases = &[
            (Any::Map(Default::default()), "{}"),
            (
                {
                    let mut map: Map = Default::default();
                    map.insert("abc".into(), Any::Int(123));
                    map.into()
                },
                "{\"abc\":123}",
            ),
        ];
        roundtrip_str_codec::<C, Any>(cases);
    }

    #[test]
    fn test_link() {
        let s = String::from("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n");
        let json = format!("{{\"/\":\"{}\"}}", s);

        let cases = &[(
            Link::from(Cid::try_from(s.as_str()).unwrap()),
            json.as_str(),
        )];
        roundtrip_str_codec::<C, Link>(cases);

        // any
        let cases = &[(
            Any::Link(Link::from(Cid::try_from(s.as_str()).unwrap()).into()),
            json.as_str(),
        )];
        roundtrip_str_codec::<C, Any>(cases);
    }
}
