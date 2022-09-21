//! IPLD codec interfaces.

#[cfg(feature = "dag-cbor")]
pub mod dag_cbor;
#[cfg(feature = "dag-json")]
pub mod dag_json;
// #[cfg(feature = "dag-pb")]
// pub mod dag_pb;
#[path = "serde.rs"]
mod _serde;

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

///
/// TODO: potentially get rid of this, in order to support raw JSON and CBOR codecs
#[cfg(feature = "specialization")]
pub use specialization::*;
#[cfg(feature = "specialization")]
mod specialization {
    use crate::dev::*;

    /// The IPLD and Serde data models do not map 1:1. As a result, Serde may
    /// encounter types that require special handling when serializing (i.e. bytes
    /// and links).
    pub trait Encoder: Serializer {
        /// Serialize a sequence of bytes.
        ///
        /// Because some codecs are text-based rather than binary, `Codec`s may define
        /// custom default behaviour for serializing bytes.
        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error>;

        /// Serialize an IPLD link.
        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }

    /// The IPLD and Serde data models do not map 1:1. As a result, Serde may
    /// encounter types that are not equivalent in IPLD (such as byte and link maps
    /// in DagJSON), or types it cannot handle altogether (such as IPLD links).
    pub trait Decoder<'de>: Deserializer<'de> {
        /// Deserialize any IPLD data type, mapping any encountered Serde type to the
        /// appropriate `Visitor` or `IpldVisitorExt` method.
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>;

        /// Deserialize a sequence of borrowed bytes.
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            // TODO: get rid of this extra trait bound
            V: IpldVisitorExt<'de>;

        /// Deserialize a sequence of bytes.
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            // TODO: get rid of this extra trait bound
            V: IpldVisitorExt<'de>;

        /// Deserialize an IPLD link.
        fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>;
    }

    // default impl<'de, V> IpldVisitorExt<'de> for V where V: Visitor<'de> {}

    // TODO remove this macro, since I dont think we need erased-serde anymore
    macro_rules! default_impl_codec {
        (@ser {$($generics:tt)*} $ty:ty) => {
            /// Default (specialized) implementation for all `Serializer`s (to avoid having
            /// to introduce an extension to `Serialize`).
            impl<$($generics)*> Encoder for $ty
            // where
            //     <$ty as serde::Serializer>::Error: 'static
            {
                /// Default behaviour is to delegate to `Serializer::serialize_bytes`.
                #[inline]
                default fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                    Serializer::serialize_bytes(self, bytes)
                }

                /// Default behaviour is to serialize the link directly as bytes.
                #[inline]
                default fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>
                {
                    Serializer::serialize_bytes(self, cid.to_bytes().as_ref())
                }
            }
        };

        (@de {$($generics:tt)*} $ty:ty) => {
            /// Default (specialized) implementation for all `Deserializer`s (to avoid
            /// having to introduce an extension to `Deserialize`).
            impl<'de, $($generics)*> Decoder<'de> for $ty
            // where
            //     <$ty as serde::Deserializer<'de>>::Error: 'static
            {
                /// Default behaviour is to delegate directly to
                /// [`Deserializer::deserialize_any`].
                #[inline]
                default fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: IpldVisitorExt<'de>,
                {
                    Deserializer::deserialize_any(self, visitor)
                }

                /// Default behaviour is to delegate directly to
                /// [`Deserializer::deserialize_bytes`].
                #[inline]
                default fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: IpldVisitorExt<'de>,
                {
                    Deserializer::deserialize_bytes(self, visitor)
                }

                /// Default behaviour is to delegate directly to
                /// [`Deserializer::deserialize_byte_buf`].
                #[inline]
                default fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: IpldVisitorExt<'de>,
                {
                    Deserializer::deserialize_byte_buf(self, visitor)
                }

                /// Default behaviour is to deserialize some borrowed bytes (or,
                /// if the deserializer is human-readable, a borrowed str) and
                /// parse them directly as a `Cid`.
                #[inline]
                default fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: IpldVisitorExt<'de>,
                {
                    if self.is_human_readable() {
                        let s = <&'de str>::deserialize(self)?;
                        visitor.visit_link_str(s)
                    } else {
                        let bytes = <&'de [u8]>::deserialize(self)?;
                        visitor.visit_link_bytes(bytes)
                    }
                }
            }
        };
    }

    default_impl_codec!(@ser {S: Serializer} S);
    default_impl_codec!(@de {D: Deserializer<'de>} D);

    // default_impl_codec!(@ser {'a} &'a mut dyn ErasedSerializer);
    // default_impl_codec!(@ser {'a} &'a mut (dyn ErasedSerializer + Send));
    // default_impl_codec!(@ser {'a} &'a mut (dyn ErasedSerializer + Sync));
    // default_impl_codec!(@ser {'a} &'a mut (dyn ErasedSerializer + Send + Sync));
    // default_impl_codec!(@de {'a} &'a mut dyn ErasedDeserializer<'de>);
    // default_impl_codec!(@de {'a} &'a mut (dyn ErasedDeserializer<'de> + Send));
    // default_impl_codec!(@de {'a} &'a mut (dyn ErasedDeserializer<'de> + Sync));
    // default_impl_codec!(@de {'a} &'a mut (dyn ErasedDeserializer<'de> + Send + Sync));

    // default_impl_codec!(@ser {'a} Box<dyn ErasedSerializer + 'a>);
    // default_impl_codec!(@ser {'a} Box<dyn ErasedSerializer + Send + 'a>);
    // default_impl_codec!(@ser {'a} Box<dyn ErasedSerializer + Sync + 'a>);
    // default_impl_codec!(@ser {'a} Box<dyn ErasedSerializer + Send + Sync + 'a>);
    // default_impl_codec!(@de {'a} Box<dyn ErasedDeserializer<'de> + 'a>);
    // default_impl_codec!(@de {'a} Box<dyn ErasedDeserializer<'de> + Send + 'a>);
    // default_impl_codec!(@de {'a} Box<dyn ErasedDeserializer<'de> + Sync + 'a>);
    // default_impl_codec!(@de {'a} Box<dyn ErasedDeserializer<'de> + Send + Sync + 'a>);
    // default_impl_codec!(@de {} Rc<dyn ErasedDeserializer<'de>>);
    // default_impl_codec!(@de {} Rc<dyn ErasedDeserializer<'de> + Send>);
    // default_impl_codec!(@de {} Rc<dyn ErasedDeserializer<'de> + Sync>);
    // default_impl_codec!(@de {} Rc<dyn ErasedDeserializer<'de> + Send + Sync>);
}

// macro_rules! autoref_codec {
//     (@encoder $trait_name:ident) => {
//
//     };
// }

/************************************************************************/
// notes about autoref
// Encoder<T>: doesnt work b/c the autoref callers cant move
// &mut Encoder<T> generic impl can't move b/c of the ref,
//  + many concrete Serializers are impled for &mut, all callers find the generic
//  + even if we made autoreffed specials <T>, they also cant move
// &mut Encoder<&mut T> doesnt work b/c Serialize impls pass concrete types, so none match
//
// Ultimate problem:
//  - we will ALWAYS be provided a concrete type
//      - so our autoref wrapper MUST always have concrete type
//      - our autoref wrapper can NEVER hold a ref type
//  - we cant move the inner (to call the inner methods) b/c of the ref
//  - we
//
/************************************************************************/

#[cfg(feature = "autoref")]
mod autoref {
    use super::*;

    ///
    #[derive(Debug)]
    pub struct Encoder<S>(pub S);

    ///
    pub trait GenericEncoder {
        type Ok;
        type Error;

        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error>;

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }

    // impl<T> GenericEncoder for Encoder<T>
    // where
    //     T: Serializer,
    // {
    //     type Ok = <T as Serializer>::Ok;
    //     type Error = <T as Serializer>::Error;
    impl<'a, T> GenericEncoder for &'a mut Encoder<T>
    where
        T: Serializer,
    {
        type Ok = <T as Serializer>::Ok;
        type Error = <T as Serializer>::Error;
        // impl<'a, T> GenericEncoder for &'a mut Encoder<&'a mut T>
        // where
        //     &'a mut T: Serializer,
        //     // for<'b> &'b mut T: Serializer,
        // {
        //     type Ok = <&'a mut T as Serializer>::Ok;
        //     type Error = <&'a mut T as Serializer>::Error;
        fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
            // self.0.serialize_bytes(bytes)
            Err(Self::Error::custom(
                "Use autoref Encoder behind #[cfg(feature = \"serde-codec\")]",
            ))
        }

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            // self.0.serialize_bytes(cid.to_bytes().as_slice())
            Err(Self::Error::custom(
                "Use autoref Encoder behind #[cfg(feature = \"serde-codec\")]",
            ))
        }
    }

    ///
    #[derive(Debug)]
    pub struct Decoder<D>(pub D);

    ///
    pub trait GenericDecoder<'de> {
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

    // impl<'de, T> GenericDecoder<'de> for Decoder<T>
    // where
    //     T: Deserializer<'de>,
    // {
    //     type Error = <T as Deserializer<'de>>::Error;
    impl<'a, 'de, T> GenericDecoder<'de> for &'a mut Decoder<T>
    where
        T: Deserializer<'de>,
    {
        type Error = <T as Deserializer<'de>>::Error;
        // impl<'a, 'de, T> GenericDecoder<'de> for &'a mut Decoder<&'a mut T>
        // where
        //     &'a mut T: Deserializer<'de>,
        //     // for<'b> &'b mut T: Deserializer<'de>,
        // {
        //     type Error = <&'a mut T as Deserializer<'de>>::Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            // self.0.deserialize_any(visitor)
            Err(Self::Error::custom(
                "Use autoref Decoder behind #[cfg(feature = \"serde-codec\")]",
            ))
        }

        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            // self.0.deserialize_bytes(visitor)
            Err(Self::Error::custom(
                "Use autoref Decoder behind #[cfg(feature = \"serde-codec\")]",
            ))
        }

        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            // self.0.deserialize_byte_buf(visitor)
            Err(Self::Error::custom(
                "Use autoref Decoder behind #[cfg(feature = \"serde-codec\")]",
            ))
        }

        fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: IpldVisitorExt<'de>,
        {
            // if self.0.is_human_readable() {
            //     let s = <&'de str>::deserialize(self.0)?;
            //     visitor.visit_link_str(s)
            // } else {
            //     let bytes = <&'de [u8]>::deserialize(self.0)?;
            //     visitor.visit_link_bytes(bytes)
            // }
            Err(Self::Error::custom(
                "Use autoref Decoder behind #[cfg(feature = \"serde-codec\")]",
            ))
        }
    }
}

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
            assert_eq!(*dag, v, "Decoding failure");

            // reading
            let v = codec.read(*expected).expect(&format!(
                "Failed to read `{}` from {:?}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Reading failure");
        }
    }

    pub fn roundtrip_str_codec<'de, T>(code: u64, cases: &[(T, &'de str)])
    where
        T: PartialEq + Debug + Representation,
    {
        let mut codec = Multicodec::try_from(code).expect("should find codec");

        for (ref dag, expected) in cases {
            // decoding
            let v = decode_from_str::<T>(&mut codec, expected).expect(&format!(
                "Failed to decode `{}` from {}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Decoding failure");

            // writing
            let string = write_to_str::<T>(&mut codec, dag).expect(&format!(
                "Failed to encode `{}` {:?} into {}",
                dag.name(),
                dag,
                expected,
            ));
            assert_eq!(*expected, string.as_str(), "Writing failure");

            // reading
            let v = codec.read(expected.as_bytes()).expect(&format!(
                "Failed to read `{}` from {}",
                dag.name(),
                expected,
            ));
            assert_eq!(*dag, v, "Reading failure");
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
