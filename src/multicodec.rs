use crate::dev::*;
use std::{
    convert::TryFrom,
    io::{Read, Write},
};

/// An unified trait for all IPLD
/// [Codec](https://github.com/ipld/specs/blob/master/block-layer/codecs/README.dsmd)s,
/// providing methods for reading and writing blocks.
pub trait Codec: Into<u64> + TryFrom<u64, Error = Error> {
    /// Given a dag, serialize it to a vector of bytes.
    fn encode<T>(&mut self, dag: &T) -> Result<Vec<u8>, Error>
    where
        T: Representation,
    {
        let mut vec = vec![];
        self.write(dag, &mut vec)?;
        Ok(vec)
    }

    /// Given a dag and a `Write`, encode it to the writer.
    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write;

    /// Given some bytes, deserialize a dag.
    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        self.read(bytes)
    }

    // /// Given some bytes, deserialize a dag.
    // fn decode_with_seed<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    // where
    //     T: Representation,
    // {
    //     unimplemented!()
    // }

    /// Given a `Read`, deserialize a dag.
    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read;

    /// Given a `Read`, deserialize a dag.
    fn read_with_seed<'de, S, R>(
        &mut self,
        seed: S,
        reader: R,
    ) -> Result<<S as DeserializeSeed<'de>>::Value, Error>
    where
        S: DeserializeSeed<'de>,
        R: Read;
}

macro_rules! impl_multicodec {
    ($(
        $(#[$meta:meta])*
        $variant:ident -> $ty:ty as $name:expr,
    )*) => {
        ///
        #[derive(Clone, Debug)]
        pub enum Multicodec {
            $(
                $(#[$meta])*
                $variant($ty),
            )*
        }

        impl Multicodec {
            ///
            #[inline]
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => $name,)*
                }
            }

            ///
            #[inline]
            pub const fn code(&self) -> u64 {
                match self {
                    $(Self::$variant(_) => <$ty>::CODE,)*
                }
            }

            ///
            #[inline]
            pub const fn from_code<const C: u64>() -> Result<Self, Error> {
                match C {
                    $(<$ty>::CODE => Ok(Self::$variant(<$ty>::new())),)*
                    code => Err(Error::UnknownMulticodecCode(code))
                }
            }

            ///
            #[inline]
            pub fn from_name(name: &str) -> Result<Self, Error> {
                match name {
                    $($name => Ok(Self::$variant(<$ty>::new())),)*
                    name => Err(Error::UnknownMulticodecName(name.to_string()))
                }
            }
        }

        impl TryFrom<u64> for Multicodec {
            type Error = Error;
            #[inline]
            fn try_from(code: u64) -> Result<Self, Self::Error> {
                match code {
                    $(<$ty>::CODE => Ok(Self::$variant(<$ty>::new())),)*
                    _ => Err(Error::UnknownMulticodecCode(code)),
                }
            }
        }

        // impl<'a> TryFrom<&'a str> for Multicodec {
        //     type Error = Error;
        //     fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        //         match s {
        //             $($str => Ok(Self::$variant(<$ty>::default())),)*
        //             _ => Err(Error::UnknownMulticodec(code)),
        //         }
        //     }
        // }

        impl Codec for Multicodec {
            fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
            where
                T: Representation,
                W: Write,
            {
                match self {
                    $(Self::$variant(inner) => inner.write(dag, writer),)*
                }
            }

            fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
            where
                T: Representation,
            {
                match self {
                    $(Self::$variant(inner) => inner.decode(bytes),)*
                }
            }

            fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
            where
                T: Representation,
                R: Read,
            {
                match self {
                    $(Self::$variant(inner) => inner.read(reader),)*
                }
            }

            fn read_with_seed<'de, S, R>(
                &mut self,
                seed: S,
                reader: R,
            ) -> Result<<S as DeserializeSeed<'de>>::Value, Error>
            where
                S: DeserializeSeed<'de>,
                R: Read,
            {
                match self {
                    $(Self::$variant(inner) => inner.read_with_seed(seed, reader),)*
                }
            }
        }
    };
}

impl_multicodec! {
    DagCbor -> DagCbor as "dag-cbor",
    DagJson -> DagJson as "dag-json",
    // VerkleDagCbor,
    // Custom(Box<dyn Codec>),
}

impl Into<u64> for Multicodec {
    #[inline]
    fn into(self) -> u64 {
        self.code()
    }
}

impl<'a, const S: usize> TryFrom<&'a CidGeneric<S>> for Multicodec {
    type Error = Error;
    fn try_from(cid: &CidGeneric<S>) -> Result<Self, Self::Error> {
        Multicodec::try_from(cid.codec())
    }
}

impl<'a> TryFrom<&'a Cid> for Multicodec {
    type Error = Error;
    fn try_from(cid: &Cid) -> Result<Self, Self::Error> {
        Multicodec::try_from(cid.multicodec_code())
    }
}
