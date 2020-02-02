//! `Ipld` codecs.
use crate::Error;
use async_trait::async_trait;
use cid::Cid;
use failure::Fail;
use serde::{
    de::DeserializeOwned,
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt::Debug,
    io::{Read, Write},
};

/// Codec trait.
#[async_trait]
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;

    /// Codec code.
    const CODEC: cid::Codec;

    /// Error type.
    type Error: Debug + Fail + Into<Error>;

    // /// Encode function.
    // async fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error>;
    // /// Decode function.
    // async fn decode(data: &[u8]) -> Result<Ipld, Self::Error>;
}

/// Extension trait for `Codec`s that can delegate to `serde`.
pub trait CodecExt: Codec {
    /// Given a dag, serialize it to bytes.
    fn encode<S>(dag: &S) -> Result<Box<[u8]>, Self::Error>
    where
        S: Serialize;

    /// Given some bytes, deserialize it to a dag.
    fn decode<'de, D>(bytes: &'de [u8]) -> Result<D, Self::Error>
    where
        D: Deserialize<'de>;

    /// Given a `Write`, serialize it to bytes.
    ///
    /// Panics by default.
    fn write<S, W>(dag: &S, writer: W) -> Result<(), Self::Error>
    where
        S: Serialize,
        W: Write,
    {
        unimplemented!()
    }

    /// Given a `Read`, deserialize it to a dag.
    ///
    /// Panics by default.
    fn read<D, R>(reader: R) -> Result<D, Self::Error>
    where
        D: DeserializeOwned,
        R: Read,
    {
        unimplemented!()
    }

    ///
    /// Because some codecs are text-based rather than binary, `Codec`s may define
    /// custom default behaviour for serializing bytes.
    fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    /// Serialize an IPLD link.
    ///
    /// Default behaviour is to serialize the link directly as bytes.
    fn serialize_link<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(cid.to_bytes().as_ref())
    }

    /// Deserialize an unknown Serde type.
    ///
    /// Because the IPLD data model doesn't map 1:1 with the Serde data model,
    /// a type's `Visitor` may be asked to visit an enum or a newtype struct.
    /// In these cases, the type can hand off
    fn deserialize_unknown<'de, D, V>(deserializer: D, visitor: V) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: IpldVisitor<'de>,
    {
        deserializer.deserialize_bytes(visitor)
    }
}

/// A helper trait for visiting a link, used by types that need `Cid`s.
pub trait IpldVisitor<'de>: Visitor<'de> {
    fn visit_link<E>(self, cid: Cid) -> Result<<Self as Visitor<'de>>::Value, E>
    where
        E: de::Error;
}
