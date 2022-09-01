use crate::dev::*;
use std::convert::TryFrom;

macro_rules! impl_multicodec {
    ($(
        $(#[$meta:meta])*
        $variant:ident -> $ty:ty,
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
            pub const fn code(&self) -> u64 {
                match self {
                    $(Self::$variant(_) => <$ty>::CODE,)*
                }
            }
        }

        impl TryFrom<u64> for Multicodec {
            type Error = Error;
            #[inline]
            fn try_from(code: u64) -> Result<Self, Self::Error> {
                match code {
                    $(<$ty>::CODE => Ok(Self::$variant(<$ty>::default())),)*
                    _ => Err(Error::UnknownMulticodec(code)),
                }
            }
        }

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
    DagCbor -> DagCbor,
    DagJson -> DagJson,
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
