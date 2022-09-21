//! IPLD DagJson codec.

use crate::dev::{macros::derive_more::Unwrap, *};
use delegate::delegate;
use serde::{de, ser};
use serde_json::de::IoRead;
#[cfg(not(feature = "simd"))]
use serde_json::{
    de::Read as JsonRead, from_reader, from_slice, to_writer, Deserializer as JsonDeserializer,
    Error as JsonError, Serializer as JsonSerializer,
};
// #[cfg(feature = "simd")]
// use simd_json::{Serializer as JsonSerializer, Deserializer as JsonDeserializer, Error as JsonError};
use std::{
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
    /// The multicodec code that identifies this IPLD Codec.
    pub const CODE: u64 = 0x0129;

    /// All bytes are encoded with the multibase `base64` w/o padding, resulting
    /// in the prefix `"m"`.
    pub const DEFAULT_MULTIBASE: Multibase = Multibase::Base64;

    /// The map key marking the map value as IPLD bytes or an IPLD link.
    pub const IPLD_KEY: &'static str = "/";

    #[doc(hidden)]
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Serializes bytes as a struct variant, e.g.
    /// `{ "/": { "base64": <some base64-encoded string> } }`.
    #[inline]
    pub(crate) fn serialize_bytes<S: Serializer>(
        bytes: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use ser::SerializeStructVariant;

        let mut sv = serializer.serialize_struct_variant("", 0, Self::IPLD_KEY, 1)?;
        sv.serialize_field("bytes", &multibase::encode(Self::DEFAULT_MULTIBASE, bytes))?;
        sv.end()
    }

    /// Serializes links as a newtype variant, e.g.  `{ "/": "Qm..." }`.
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
    pub(crate) fn deserialize_any<'de, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: IpldVisitorExt<'de>,
    {
        deserializer.deserialize_any(visitor::DagJsonVisitor::<'a', _>(visitor))
    }

    ///
    #[inline]
    pub(crate) fn deserialize_bytes<'de, D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(visitor::DagJsonVisitor::<'b', _>(visitor))
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
        deserializer.deserialize_map(visitor::DagJsonVisitor::<'l', _>(visitor))
    }

    pub(crate) fn read_with_seed<'de, S, R>(&mut self, seed: S, reader: R) -> Result<(), Error>
    where
        // S: DeserializeSeed<'de, Value = ()>,
        // BlockSelectorSeed<C, S>: DeserializeSeed<'de, Value = ()>,
        S: CodecDeserializeSeed<'de>,
        R: Read,
    {
        let mut de = JsonDeserializer::from_reader(reader);
        // let seed = BlockSelectorSeed::<{ Self::CODE }, _>(seed);
        seed.deserialize::<{ Self::CODE }, _>(&mut de)
            .map_err(Error::decoder)
        // unimplemented!()
    }
}

impl Codec for DagJson {
    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write,
    {
        let mut ser = JsonSerializer::new(writer);
        Representation::serialize::<{ Self::CODE }, _>(dag, &mut ser).map_err(Error::encoder)
    }

    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        let mut de = JsonDeserializer::from_slice(bytes);
        Representation::deserialize::<{ Self::CODE }, _>(&mut de).map_err(Error::decoder)
    }

    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read,
    {
        let mut de = JsonDeserializer::from_reader(reader);
        Representation::deserialize::<{ Self::CODE }, _>(&mut de).map_err(Error::decoder)
    }
}

impl TryFrom<u64> for DagJson {
    type Error = Error;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            Self::CODE => Ok(Self),
            _ => Err(Error::UnknownMulticodecCode(code)),
        }
    }
}

// impl<W: Write> CodecExt<{ DagJson::CODE }> for &mut JsonSerializer<W> {}
// impl<'de, R: JsonRead<'de>> CodecExt<{ DagJson::CODE }> for &mut JsonDeserializer<R> {}

#[cfg(feature = "specialization")]
mod specialization {
    use super::*;

    impl<'a, W> Encoder for &'a mut JsonSerializer<W>
    where
        W: Write,
    {
        /// Serializes bytes as a struct variant, e.g.
        /// `{ "/": { "base64": <some base64-encoded string> } }`.
        #[inline]
        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, JsonError> {
            use ser::SerializeStructVariant;

            let mut sv = self.serialize_struct_variant("", 0, DagJson::IPLD_KEY, 1)?;
            sv.serialize_field(
                "bytes",
                &multibase::encode(DagJson::DEFAULT_MULTIBASE, bytes),
            )?;
            sv.end()
        }

        /// Serializes links as a newtype variant, e.g.  `{ "/": "Qm..." }`.
        #[inline]
        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, JsonError> {
            let cid_str = cid.to_string().map_err(<JsonError as ser::Error>::custom)?;
            self.serialize_newtype_variant("", 0, DagJson::IPLD_KEY, &cid_str)
        }
    }

    impl<'a, 'de, R> Decoder<'de> for &'a mut JsonDeserializer<R>
    where
        R: JsonRead<'de>,
    {
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

        /// `JsonVisitor` wraps a `Visitor` in order to enhance how maps are
        /// deserialized.
        pub struct JsonVisitor<V>(pub V);

        impl<'de, V> Visitor<'de> for JsonVisitor<V>
        where
            V: IpldVisitorExt<'de>,
        {
            type Value = V::Value;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON map, link object or byte object")
            }

            /// Called when a map is found in the input data.
            /// TODO: test that this works with links union-inlined into structs/maps
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let first_key: Option<Cow<'_, str>> = map.next_key()?;
                if Some(DagJson::IPLD_KEY) == first_key.as_deref() {
                    match map.next_value::<MapLikeVisitor<'de>>() {
                        Ok(MapLikeVisitor::Bytes(b)) => self.0.visit_byte_buf(b),
                        Ok(MapLikeVisitor::CidStr(s)) => self.0.visit_link_str(s),
                        Ok(MapLikeVisitor::CidString(s)) => self.0.visit_link_str(&s),
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

            /// In the dag-json codec, links are represented as either Base58 or Base32
            /// strings
            #[inline]
            fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MapLikeVisitor::CidStr(s))
            }

            /// In the dag-json codec, links are represented as either Base58 or Base32
            /// strings
            #[inline]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MapLikeVisitor::CidString(s.into()))
            }

            /// In the dag-json codec, bytes are represented as maps, with the key
            /// always being the string "bytes" and the value always being the bytes
            /// multibase-encoded as a string.
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                map.next_key::<String>()?
                    .filter(|key| key == "bytes")
                    .ok_or_else(|| {
                        A::Error::custom("DagJSON bytes key must be the string \"bytes\"")
                    })?;

                // TODO: empty string
                let byte_str = map.next_value::<String>()?;
                match multibase::decode(byte_str) {
                    Ok((DagJson::DEFAULT_MULTIBASE, bytes)) => Ok(MapLikeVisitor::Bytes(bytes)),
                    Ok((mb, _)) => Err(de::Error::custom(format!(
                        "DagJSON only supports bytes as base64-encoded strings, found multibase {:?}",
                        mb
                    ))),
                    _ => Err(de::Error::custom(
                        "DagJSON only supports bytes as base64-encoded strings",
                    )),
                }
            }
        }

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
    }
}

mod visitor {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct DagJsonVisitor<const C: char, V>(pub(crate) V);

    impl<'de, V: IpldVisitorExt<'de>> Visitor<'de> for DagJsonVisitor<'a', V> {
        type Value = V::Value;
        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
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

    impl<'de, V: Visitor<'de>> Visitor<'de> for DagJsonVisitor<'b', V> {
        type Value = V::Value;

        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a map containing a byte string")
        }

        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            map.next_key::<Cow<'_, str>>()?
                .filter(|key| key == DagJson::IPLD_KEY)
                .ok_or_else(|| A::Error::custom("expected a \"/\" map key"))?;

            match map.next_value::<MapLikeVisitor<'de>>()? {
                MapLikeVisitor::Bytes(b) => self.0.visit_byte_buf(b),
                _ => Err(A::Error::custom("expected a byte string")),
            }
        }
    }

    impl<'de, V: IpldVisitorExt<'de>> Visitor<'de> for DagJsonVisitor<'l', V> {
        type Value = V::Value;

        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a map containing a Cid")
        }

        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            map.next_key::<Cow<'_, str>>()?
                .filter(|key| key == DagJson::IPLD_KEY)
                .ok_or_else(|| {
                    A::Error::custom(format!(
                        "expected the special IPLD map key {}",
                        DagJson::IPLD_KEY
                    ))
                })?;

            match map.next_value::<MapLikeVisitor<'de>>()? {
                MapLikeVisitor::CidStr(s) => self.0.visit_link_str(s),
                MapLikeVisitor::CidString(s) => self.0.visit_link_str(&s),
                _ => Err(A::Error::custom("expected a Cid")),
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

        /// In the dag-json codec, links are represented as either Base58 or Base32
        /// strings
        #[inline]
        fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::CidStr(s))
        }

        /// In the dag-json codec, links are represented as either Base58 or Base32
        /// strings
        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(MapLikeVisitor::CidString(s.into()))
        }

        /// In the dag-json codec, bytes are represented as maps, with the key
        /// always being the string "bytes" and the value always being the bytes
        /// multibase-encoded as a string.
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

            // TODO: empty string
            let byte_str = map.next_value::<String>()?;
            match multibase::decode(byte_str) {
                Ok((DagJson::DEFAULT_MULTIBASE, bytes)) => Ok(MapLikeVisitor::Bytes(bytes)),
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
}

#[cfg(feature = "wrapped")]
mod wrapped {
    use super::*;

    struct DagJsonSerializer<'a, W, F>(&'a mut JsonSerializer<W, F>);
    // #[derive(Unwrap)]
    // enum Compound<'a, W, F>
    // where
    //     W: 'a + Write,
    //     F: 'a + serde_json::ser::Formatter,
    // {
    //     Seq(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeSeq),
    //     Tuple(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeTuple),
    //     TupleStruct(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeTupleStruct),
    //     TupleVariant(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeTupleVariant),
    //     Map(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeMap),
    //     Struct(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeStruct),
    //     StructVariant(<&'a mut JsonSerializer<W, F> as Serializer>::SerializeStructVariant),
    // }

    impl<'a, W, F> Serializer for DagJsonSerializer<'a, W, F>
    where
        W: Write,
        F: serde_json::ser::Formatter,
    {
        type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
        type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;

        type SerializeSeq = Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeSeq>;
        type SerializeTuple =
            Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeTuple>;
        type SerializeTupleStruct =
            Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeTupleStruct>;
        type SerializeTupleVariant =
            Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeTupleVariant>;
        type SerializeMap = Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeMap>;
        type SerializeStruct =
            Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeStruct>;
        type SerializeStructVariant =
            Compound<<&'a mut JsonSerializer<W, F> as Serializer>::SerializeStructVariant>;

        delegate! {
            to self.0 {
                fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error>;
                fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error>;
                fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error>;
                fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error>;
                fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error>;
                fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error>;
                fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error>;
                fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error>;
                fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error>;
                fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error>;
                fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error>;
                fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error>;
                fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error>;
                fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error>;
                fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error>;

                fn serialize_unit(self) -> Result<Self::Ok, Self::Error>;
                fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error>;
                fn serialize_none(self) -> Result<Self::Ok, Self::Error>;
                fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
                where
                    T: ?Sized + Serialize;
            }
        }

        #[inline]
        fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
            // use serde::ser::SerializeSeq;
            // let mut seq = tri!(self.serialize_seq(Some(value.len())));
            // for byte in value {
            //     tri!(seq.serialize_element(byte));
            // }
            // seq.end()
            unimplemented!()
        }

        #[inline]
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            self.serialize_str(variant)
        }

        /// TODO: handle the CID_SERDE_PRIVATE_IDENTIFIER to
        #[inline]
        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            value.serialize(self)
        }

        #[inline]
        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            unimplemented!()
        }

        #[inline]
        fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            unimplemented!()
        }

        #[inline]
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            self.serialize_seq(Some(len))
        }

        #[inline]
        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            self.serialize_seq(Some(len))
        }

        #[inline]
        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            unimplemented!()
        }

        #[inline]
        fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            unimplemented!()
        }

        #[inline]
        fn serialize_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            unimplemented!()
        }

        #[inline]
        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            unimplemented!()
        }

        fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + std::fmt::Display,
        {
            unimplemented!()
        }
    }

    #[cfg(feature = "")]
    mod compound {
        use super::*;

        struct Compound<'a, W: 'a, F: 'a> {
            ser: DagJsonSerializer<'a, W, F>,
            state: serde_json::ser::State,
        }

        impl<'a, W, F> ser::SerializeSeq for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
            fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + Serialize,
            {
            }
        }
        impl<'a, W, F> ser::SerializeTuple for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
        }
        impl<'a, W, F> ser::SerializeTupleStruct for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
        }
        impl<'a, W, F> ser::SerializeTupleVariant for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
        }
        impl<'a, W, F> ser::SerializeMap for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
        }
        impl<'a, W, F> ser::SerializeStruct for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
        }
        impl<'a, W, F> ser::SerializeStructVariant for Compound<'a, W, F>
        where
            W: Write,
            F: serde_json::ser::Formatter,
        {
            type Ok = <&'a mut JsonSerializer<W, F> as Serializer>::Ok;
            type Error = <&'a mut JsonSerializer<W, F> as Serializer>::Error;
        }
    }
}

#[cfg(feature = "autoref")]
mod autoref {

    pub trait DagJsonEncoder {
        type Ok;
        type Error;

        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error>;

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }

    impl<'a, W> DagJsonEncoder for &'a mut &'a mut Encoder<&mut JsonSerializer<W>>
    where
        W: Write,
    {
        type Ok = ();
        type Error = JsonError;

        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
            use ser::SerializeStructVariant;

            let mut sv = self
                .0
                .serialize_struct_variant("", 0, DagJson::IPLD_KEY, 1)?;
            sv.serialize_field(
                "bytes",
                &multibase::encode(DagJson::DEFAULT_MULTIBASE, bytes),
            )?;
            sv.end()
        }

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            let cid_str = cid
                .to_string()
                .map_err(<Self::Error as ser::Error>::custom)?;
            (&mut self.0).serialize_newtype_variant("", 0, DagJson::IPLD_KEY, &cid_str)
        }
    }

    pub trait DagJsonDecoder<'de> {
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

    impl<'a, 'de, R> DagJsonDecoder<'de> for &'a mut &'a mut Decoder<&mut JsonDeserializer<R>>
    where
        R: JsonRead<'de>,
    {
        type Error = JsonError;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            self.0.deserialize_any(DagJsonVisitor::<'a', V>(visitor))
        }

        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.0.deserialize_map(DagJsonVisitor::<'b', V>(visitor))
        }

        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.0.deserialize_map(DagJsonVisitor::<'b', V>(visitor))
        }

        fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            self.0.deserialize_map(DagJsonVisitor::<'l', V>(visitor))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{codecs_::test_utils::*, prelude::*};

    #[test]
    fn test_null() {
        let tests = &[(Null, "null")];
        roundtrip_str_codec::<Null>(DagJson::CODE, tests);
        let tests = &[(None as Option<Int>, "null")];
        roundtrip_str_codec::<Option<Int>>(DagJson::CODE, tests);
    }

    #[test]
    fn test_bool() {
        let tests = &[(true, "true"), (false, "false")];
        roundtrip_str_codec::<Bool>(DagJson::CODE, tests);
    }

    #[test]
    fn test_number() {
        let tests = &[(123, "123"), (65535, "65535")];
        roundtrip_str_codec::<Int>(DagJson::CODE, tests);
        let tests = &[(123.123, "123.123")];
        roundtrip_str_codec::<Float>(DagJson::CODE, tests);
    }

    #[test]
    fn test_string() {
        let tests = &[
            // standard string
            (IpldString::from("hello world"), "\"hello world\""),
            // non-standard UTF-8 string TODO:
            // (IpldString::from("ÅΩ"), "\"ÅΩ\""),
        ];
        roundtrip_str_codec::<_>(DagJson::CODE, tests);
    }

    #[test]
    fn test_bytes() {
        let tests = &[(
            Bytes::from(vec![0x01, 0x02, 0x03]),
            r#"{"/":{"bytes":"mAQID"}}"#,
        )];
        roundtrip_str_codec::<Bytes>(DagJson::CODE, tests);
    }

    #[test]
    fn test_link() {
        type TestLink = Link<Any>;

        let s = String::from("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n");
        let json = format!("{{\"/\":\"{}\"}}", s);

        let tests = &[(
            TestLink::from(Cid::try_from(s.as_str()).unwrap()),
            json.as_str(),
        )];
        roundtrip_str_codec::<TestLink>(DagJson::CODE, tests);
    }

    #[test]
    fn test_seq() {
        let tests = &[
            // (vec![], "[]"),
            // (vec![Any::Int(1), Any::Int(2)], "[1,2]"),
            // // (
            // //     vec![Any::Link(Link::Cid(Default::default()).into())],
            // //     "[{\"/\": }]",
            // // ),
        ];
        roundtrip_str_codec::<List>(DagJson::CODE, tests);
    }

    #[test]
    fn test_map() {
        let tests = &[];
        roundtrip_str_codec::<Map>(DagJson::CODE, tests);
    }
}
