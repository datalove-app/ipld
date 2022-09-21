//! IPLD codec interfaces.

#[cfg(feature = "dag-cbor")]
pub mod dag_cbor;
#[cfg(feature = "dag-json")]
pub mod dag_json;
// #[cfg(feature = "dag-pb")]
// pub mod dag_pb;

use crate::dev::*;
use serde::{de, ser};
use std::{
    convert::TryFrom,
    io::{Read, Write},
};

// pub trait CodecExt<'de>: Codec {
//     type Encoder: Encoder;
//     type Decoder: Decoder<'de>;
//
//     fn encoder<W: Write>(writer: W) -> Result<Self::Encoder, Error>;
//
//     fn decoder<R: Read>(reader: R) -> Result<Self::Decoder, Error>;
// }

//
// pub trait Transcoder<'de> {
//     type Serializer: Serializer;
//     type Deserializer: Deserializer<'de>;
//
//     fn serializer(&mut self) -> Option<&mut Self::Serializer>;
//     fn deserializer(&mut self) -> &mut Self::Deserializer;
// }

/// A helper trait for visiting special and recursive IPLD types.
///
/// Should be implemented by any types representing IPLD links and maps.
pub trait IpldVisitorExt<'de>: Visitor<'de> {
    /// The input contains the string of a `Cid`.
    ///
    /// The default implementation fails with a type error.
    fn visit_link_str<E>(self, cid_str: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(E::invalid_type(de::Unexpected::Other("Cid"), &self))
    }

    /// The input contains the string of a `Cid`.
    ///
    /// The default implementation delegates to [`visit_link_str`].
    #[inline]
    fn visit_link_borrowed_str<E>(self, cid_str: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_link_str(cid_str)
    }

    /// The input contains a string representation of a `Cid`.
    ///
    /// The default implementation fails with a type error.
    fn visit_link_bytes<E>(self, cid_bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(E::invalid_type(de::Unexpected::Other("Cid"), &self))
    }

    /// The input contains a string representation of a `Cid`.
    ///
    /// The default implementation delegates to [`visit_link_bytes`].
    fn visit_link_borrowed_bytes<E>(self, cid_bytes: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_link_bytes(cid_bytes)
    }
}

pub(crate) mod test_utils {
    use crate::dev::*;

    use std::{convert::TryFrom, fmt::Debug, io::Read, string::ToString};

    pub fn roundtrip_bytes_codec<'de, T>(code: u64, cases: &[(T, &'de [u8])])
    where
        T: PartialEq + Debug + Representation,
    {
        let mut codec = Multicodec::try_from(code).expect("should find codec");

        for (ref dag, expected) in cases {
            // writing
            let bytes = write_to_bytes::<T>(&mut codec, dag).expect(&format!(
                "Failed to encode `{}` {:?} into {:?}",
                dag.name(),
                dag,
                expected,
            ));
            assert_eq!(expected, &bytes.as_slice(), "Writing failure");

            // decoding
            let v = decode_from_bytes::<T>(&mut codec, expected).expect(&format!(
                "Failed to decode `{}` from {:?}",
                dag.name(),
                expected,
            ));
            assert_eq!(dag, &v, "Decoding failure");

            // reading
            let v = codec.read(*expected).expect(&format!(
                "Failed to read `{}` from {:?}",
                dag.name(),
                expected,
            ));
            assert_eq!(dag, &v, "Reading failure");
        }
    }

    pub fn roundtrip_str_codec<'de, T>(code: u64, cases: &[(T, &'de str)])
    where
        T: PartialEq + Debug + Representation,
    {
        let mut codec = Multicodec::try_from(code).expect("should find codec");

        for (ref dag, expected) in cases {
            // writing
            let string = write_to_str::<T>(&mut codec, dag).expect(&format!(
                "Failed to encode `{}` {:?} into {}",
                dag.name(),
                dag,
                expected,
            ));
            assert_eq!(*expected, string.as_str(), "Writing failure");

            // decoding
            let v = decode_from_str::<T>(&mut codec, expected).expect(&format!(
                "Failed to decode `{}` from {}",
                dag.name(),
                expected,
            ));
            assert_eq!(dag, &v, "Decoding failure");

            // reading
            let v = codec.read(expected.as_bytes()).expect(&format!(
                "Failed to read `{}` from {}",
                dag.name(),
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
