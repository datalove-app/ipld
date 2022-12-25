//! IPLD codec interfaces.

#[cfg(feature = "dag-cbor")]
mod dag_cbor;
#[cfg(feature = "dag-json")]
mod dag_json;
// #[cfg(feature = "dag-rkyv")]
// pub mod dag_rkyv;
// #[cfg(feature = "dag-pb")]
// pub mod dag_pb;
// #[cfg(feature = "dag-jose")]
// pub mod dag_jose;
// #[cfg(feature = "raw")]
// pub mod raw;
// #[cfg(feature = "json")]
// pub mod json;

#[cfg(feature = "dag-cbor")]
pub use dag_cbor::DagCbor;
#[cfg(feature = "dag-json")]
pub use dag_json::DagJson;

use crate::dev::*;
use maybestd::{
    convert::TryFrom,
    fmt,
    io::{Read, Write},
};

/// [Multicodec]() code for the identity codec.
///
/// In this library, this codec is used to convert the type into its
/// (de)serializable representation (sans encoding) so that it can be
/// re-represented (aka converted) to another type, or reified for selection by
/// another type.
pub const IDENTITY: u64 = 0x00;

/*
///
/// keeps multiple counters and flags
/// - is_ignoring: reads 1 byte?, incremenets ignored bytes and start_idx
/// - is reading:
/// -
enum State {
    Default,
    Ignoring { start_idx: usize },
    Reading { start_idx: usize },
}

pub struct TranscoderCursor<R, W> {
    _t: PhantomData<(R, W)>,
}

///
/// impls Deserializer:
/// - on ignore:
///     - set reader to ignore mode
///     - call inner deserializer
///     - flush
///     - calls to types do the op, serializer to writer, tell reader to skip # written bytes
///     - calls to ignore end by telling reader to flush buffer to writer
///     FIXME: cant always tell which bytes were used
pub struct Transcoder<'a, D, S> {
    _t: PhantomData<(D, S)>,
}

impl<'de, D: Deserializer<'de>, S: Serializer> Transcoder<D, S> {
    fn from_rw<R, W>(reader: R, writer: W) -> Self {
        Self { _t: PhantomData }
    }
}
 */

//
// pub trait Transcoder<'de> {
//     type Serializer: Serializer;
//     type Deserializer: Deserializer<'de>;
//
//     fn serializer(&mut self) -> Option<&mut Self::Serializer>;
//     fn deserializer(&mut self) -> &mut Self::Deserializer;
// }

/// An unified trait for all IPLD
/// [Codec](https://github.com/ipld/specs/blob/master/block-layer/codecs/README.dsmd)s,
/// providing methods for reading and writing blocks.
pub trait Codec<T: Representation = Any>: TryFrom<u64, Error = Error> {
    /// The standardized [`Multicodec`] name for this IPLD codec.
    const NAME: &'static str;

    /// The standardized [`Multicodec`] code that identifies this IPLD codec.
    const CODE: u64;

    /// Given a dag, serialize it to a `Vec<u8>`.
    fn encode(&mut self, dag: &T) -> Result<Vec<u8>, Error> {
        let mut vec = vec![];
        self.write(dag, &mut vec)?;
        Ok(vec)
    }

    /// Given a dag and a `Write`, encode it to the writer.
    fn write<W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        W: Write;

    /// Given some bytes, deserialize a dag.
    fn decode<'de>(&mut self, bytes: &'de [u8]) -> Result<T, Error> {
        self.read::<_>(bytes)
    }

    /// Given a `Read`, deserialize a dag.
    fn read<R>(&mut self, reader: R) -> Result<T, Error>
    where
        R: Read;
}

/// An extension to the [`serde::de::Visitor`] trait for visiting
/// [`Representation`]s that contain IPLD links.
///
/// [`Representation`]: crate::prelude::Representation
pub trait LinkVisitor<'de, const MC: u64 = IDENTITY>: Visitor<'de> {
    // /// The value produced by this visitor.
    // type Value;

    // /// Format a message stating what data this Visitor expects to receive.
    // fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// The input contains the string of a [`Cid`].
    ///
    /// The default implementation fails with a type error.
    fn visit_link_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // let cid = Cid::try_from(s).map_err(E::custom)?;
        // self.visit_cid(cid)
        todo!()
    }

    /// The input contains the string of a [`Cid`].
    ///
    /// The default implementation delegates to [`LinkVisitor::visit_link_str`].
    #[inline]
    fn visit_link_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_link_str(s)
    }

    /// The input contains a string representation of a [`Cid`].
    ///
    /// The default implementation fails with a type error.
    #[inline]
    fn visit_link_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // let cid = Cid::try_from(b).map_err(E::custom)?;
        // self.visit_cid(cid)
        todo!()
    }

    /// The input contains a string representation of a [`Cid`].
    ///
    /// The default implementation delegates to [`LinkVisitor::visit_link_bytes`].
    #[inline]
    fn visit_link_borrowed_bytes<E>(self, b: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_link_bytes(b)
    }

    /// The input contains an already parsed [`Cid`].
    fn visit_cid<E>(self, _: Cid) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(E::invalid_type(de::Unexpected::Other("Cid"), &self))
    }

    // fn visit_dag<E>(self, _: T) -> Result<Self::Value, E>
    // where
    //     E: serde::de::Error,
    // {
    //     unimplemented!()
    // }

    // fn visit_ref<E>(self, _: &T) -> Result<Self::Value, E>
    // where
    //     E: serde::de::Error,
    // {
    //     unimplemented!()
    // }

    // fn visit_ref_mut<E>(self, _: &T) -> Result<Self::Value, E>
    // where
    //     E: serde::de::Error,
    // {
    //     unimplemented!()
    // }

    ////////////////////////////////////////////////////////////////////////
    // Visitor API from serde
    ////////////////////////////////////////////////////////////////////////

    /*
    /// See [`Visitor::visit_bool`].
    ///
    /// [`Visitor::visit_bool`]: serde::Visitor::visit_bool
    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_bool(self, v)
    }
    /// See [`Visitor::visit_i8`].
    ///
    /// [`Visitor::visit_i8`]: serde::Visitor::visit_i8
    #[inline]
    fn visit_i8<E>(self, v: i8) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(v as i64)
    }
    /// See [`Visitor::visit_i16`].
    ///
    /// [`Visitor::visit_i16`]: serde::Visitor::visit_i16
    #[inline]
    fn visit_i16<E>(self, v: i16) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(v as i64)
    }
    /// See [`Visitor::visit_i32`].
    ///
    /// [`Visitor::visit_i32`]: serde::Visitor::visit_i32
    #[inline]
    fn visit_i32<E>(self, v: i32) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(v as i64)
    }
    /// See [`Visitor::visit_i64`].
    ///
    /// [`Visitor::visit_i64`]: serde::Visitor::visit_i64
    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_i64(self, v)
    }
    serde::serde_if_integer128! {
        /// See [`Visitor::visit_i128`].
        ///
        /// [`Visitor::visit_i128`]: serde::Visitor::visit_i128
        #[inline]
        fn visit_i128<E>(self, v: i128) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
        where
            E: de::Error,
        {
            <Self as Visitor<'de>>::visit_i128(self, v)
        }
    }
    /// See [`Visitor::visit_u8`].
    ///
    /// [`Visitor::visit_u8`]: serde::Visitor::visit_u8
    #[inline]
    fn visit_u8<E>(self, v: u8) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(v as u64)
    }
    /// See [`Visitor::visit_u16`].
    ///
    /// [`Visitor::visit_u16`]: serde::Visitor::visit_u16
    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(v as u64)
    }
    /// See [`Visitor::visit_u32`].
    ///
    /// [`Visitor::visit_u32`]: serde::Visitor::visit_u32
    #[inline]
    fn visit_u32<E>(self, v: u32) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(v as u64)
    }
    /// See [`Visitor::visit_u64`].
    ///
    /// [`Visitor::visit_u64`]: serde::Visitor::visit_u64
    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_u64(self, v)
    }
    serde::serde_if_integer128! {
        /// See [`Visitor::visit_u128`].
        ///
        /// [`Visitor::visit_u128`]: serde::Visitor::visit_u128
        #[inline]
        fn visit_u128<E>(self, v: u128) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
        where
            E: de::Error,
        {
            <Self as Visitor<'de>>::visit_u128(self, v)
        }
    }
    /// See [`Visitor::visit_f32`].
    ///
    /// [`Visitor::visit_f32`]: serde::Visitor::visit_f32
    #[inline]
    fn visit_f32<E>(self, v: f32) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_f64(v as f64)
    }
    /// See [`Visitor::visit_f64`].
    ///
    /// [`Visitor::visit_f64`]: serde::Visitor::visit_f64
    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_f64(self, v)
    }
    /// See [`Visitor::visit_char`].
    ///
    /// [`Visitor::visit_char`]: serde::Visitor::visit_char
    #[inline]
    fn visit_char<E>(self, v: char) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_char(self, v)
    }
    /// See [`Visitor::visit_str`].
    ///
    /// [`Visitor::visit_str`]: serde::Visitor::visit_str
    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_str(self, v)
    }
    /// See [`Visitor::visit_borrowed_str`].
    ///
    /// [`Visitor::visit_borrowed_str`]: serde::Visitor::visit_borrowed_str
    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v)
    }
    /// See [`Visitor::visit_string`].
    ///
    /// [`Visitor::visit_string`]: serde::Visitor::visit_string
    #[inline]
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, v: String) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&v)
    }
    /// See [`Visitor::visit_bytes`].
    ///
    /// [`Visitor::visit_bytes`]: serde::Visitor::visit_bytes
    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_bytes(self, v)
    }
    /// See [`Visitor::visit_borrowed_bytes`].
    ///
    /// [`Visitor::visit_borrowed_bytes`]: serde::Visitor::visit_borrowed_bytes
    #[inline]
    fn visit_borrowed_bytes<E>(
        self,
        v: &'de [u8],
    ) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(v)
    }
    /// See [`Visitor::visit_byte_buf`].
    ///
    /// [`Visitor::visit_byte_buf`]: serde::Visitor::visit_byte_buf
    #[inline]
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(&v)
    }
    /// See [`Visitor::visit_none`].
    ///
    /// [`Visitor::visit_none`]: serde::Visitor::visit_none
    #[inline]
    fn visit_none<E>(self) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_none(self)
    }
    /// See [`Visitor::visit_some`].
    ///
    /// [`Visitor::visit_some`]: serde::Visitor::visit_some
    #[inline]
    fn visit_some<D>(
        self,
        deserializer: D,
    ) -> Result<<Self as LinkVisitor<'de, MC>>::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Self as Visitor<'de>>::visit_some(self, deserializer)
    }
    /// See [`Visitor::visit_unit`].
    ///
    /// [`Visitor::visit_unit`]: serde::Visitor::visit_unit
    #[inline]
    fn visit_unit<E>(self) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>
    where
        E: de::Error,
    {
        <Self as Visitor<'de>>::visit_unit(self)
    }
    /// See [`Visitor::visit_newtype_struct`].
    ///
    /// [`Visitor::visit_newtype_struct`]: serde::Visitor::visit_newtype_struct
    #[inline]
    fn visit_newtype_struct<D>(
        self,
        deserializer: D,
    ) -> Result<<Self as LinkVisitor<'de, MC>>::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Self as Visitor<'de>>::visit_newtype_struct(self, deserializer)
    }
    /// See [`Visitor::visit_seq`].
    ///
    /// [`Visitor::visit_seq`]: serde::Visitor::visit_seq
    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<<Self as LinkVisitor<'de, MC>>::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        <Self as Visitor<'de>>::visit_seq(self, seq)
    }
    /// See [`Visitor::visit_map`].
    ///
    /// [`Visitor::visit_map`]: serde::Visitor::visit_map
    #[inline]
    fn visit_map<A>(self, map: A) -> Result<<Self as LinkVisitor<'de, MC>>::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        <Self as Visitor<'de>>::visit_map(self, map)
    }
    /// See [`Visitor::visit_enum`].
    ///
    /// [`Visitor::visit_enum`]: serde::Visitor::visit_enum
    #[inline]
    fn visit_enum<A>(self, data: A) -> Result<<Self as LinkVisitor<'de, MC>>::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        <Self as Visitor<'de>>::visit_enum(self, data)
    }
    #[doc(hidden)]
    fn __private_visit_untagged_option<D>(
        self,
        deserializer: D,
    ) -> Result<<Self as LinkVisitor<'de, MC>>::Value, ()>
    where
        D: Deserializer<'de>,
    {
        <Self as Visitor<'de>>::__private_visit_untagged_option(self, deserializer)
    }
     */
}

// pub struct MulticodecVisitor<const MC: u64, V>(V);
// impl<const MC: u64, V:Visitor<'de>> Visitor<'de> for MulticodecVisitor<MC> {
//     type Value = V::Value;

//     delegate! {
//         to self.0 {
//             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
//             fn visit_bool<E: de::Error>(self, v: bool) -> Result<<Self as LinkVisitor<'de, MC>>::Value, E>;
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
//             fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E>;
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
//             fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error>;
//         }
//     }
// }

pub(crate) mod test_utils {
    use crate::dev::*;
    use maybestd::{convert::TryFrom, fmt::Debug};

    pub fn roundtrip_bytes_codec<'de, const C: u64, T>(cases: &[(T, &'de [u8])])
    where
        T: PartialEq + Debug + Representation,
    {
        let mut codec = Multicodec::try_from(C).expect("should find codec");

        for (ref dag, expected) in cases {
            // writing
            let bytes = write_to_bytes::<T>(&mut codec, dag).expect(&format!(
                "Failed to encode `{}` {:?} into {:?}",
                T::NAME,
                dag,
                expected,
            ));
            assert_eq!(expected, &bytes.as_slice(), "Writing failure");

            // decoding
            let v = decode_from_bytes::<T>(&mut codec, expected).expect(&format!(
                "Failed to decode `{}` from {:?}",
                T::NAME,
                expected,
            ));
            assert_eq!(dag, &v, "Decoding failure");

            // reading
            let v = codec.read(*expected).expect(&format!(
                "Failed to read `{}` from {:?}",
                T::NAME,
                expected,
            ));
            assert_eq!(dag, &v, "Reading failure");
        }
    }

    pub fn roundtrip_str_codec<'de, const C: u64, T>(cases: &[(T, &'de str)])
    where
        T: PartialEq + Debug + Representation,
    {
        let mut codec = Multicodec::try_from(C).expect("should find codec");

        for (ref dag, expected) in cases {
            // writing
            let string = write_to_str::<T>(&mut codec, dag).expect(&format!(
                "Failed to encode `{}` {:?} into {}",
                T::NAME,
                dag,
                expected,
            ));
            assert_eq!(*expected, string.as_str(), "Writing failure");

            // decoding
            let v = decode_from_str::<T>(&mut codec, expected).expect(&format!(
                "Failed to decode `{}` from {}",
                T::NAME,
                expected,
            ));
            assert_eq!(dag, &v, "Decoding failure");

            // reading
            let v = codec.read(expected.as_bytes()).expect(&format!(
                "Failed to read `{}` from {}",
                T::NAME,
                expected,
            ));
            assert_eq!(dag, &v, "Reading failure");
        }
    }

    fn write_to_bytes<T>(codec: &mut Multicodec, dag: &T) -> Result<Vec<u8>, Error>
    where
        T: Representation,
    {
        let mut bytes = Vec::new();
        codec.write(dag, &mut bytes)?;
        Ok(bytes)
    }

    fn write_to_str<T>(codec: &mut Multicodec, dag: &T) -> Result<String, Error>
    where
        T: Representation,
    {
        let bytes = write_to_bytes::<T>(codec, dag)?;
        Ok(String::from_utf8(bytes).unwrap())
    }

    fn decode_from_bytes<'de, T>(codec: &mut Multicodec, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        codec.decode(bytes)
    }

    fn decode_from_str<'de, T>(codec: &mut Multicodec, s: &'de str) -> Result<T, Error>
    where
        T: Representation,
    {
        codec.decode(s.as_bytes())
    }
}
