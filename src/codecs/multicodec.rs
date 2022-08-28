use crate::dev::*;
use std::convert::TryFrom;

///
#[derive(Clone, Debug)]
pub enum Multicodec {
    DagCbor(DagCbor),
    DagJson(DagJson),
    // VerkleDagCbor,
    // Custom(Box<dyn Codec>),
}

impl Multicodec {
    #[inline]
    pub fn code(&self) -> u64 {
        match self {
            Self::DagCbor(_) => DagCbor::CODE,
            Self::DagJson(_) => DagJson::CODE,
        }
    }
}

impl Into<u64> for Multicodec {
    #[inline]
    fn into(self) -> u64 {
        self.code()
    }
}

impl TryFrom<u64> for Multicodec {
    type Error = Error;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            DagCbor::CODE => Ok(Multicodec::DagCbor(DagCbor)),
            DagJson::CODE => Ok(Multicodec::DagJson(DagJson)),
            _ => Err(Error::UnknownCodec(code)),
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
            Self::DagCbor(inner) => inner.write(dag, writer),
            Self::DagJson(inner) => inner.write(dag, writer),
        }
    }

    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        match self {
            Self::DagCbor(inner) => inner.decode(bytes),
            Self::DagJson(inner) => inner.decode(bytes),
        }
    }

    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read,
    {
        match self {
            Self::DagCbor(inner) => inner.read(reader),
            Self::DagJson(inner) => inner.read(reader),
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
            Self::DagCbor(inner) => inner.read_with_seed(seed, reader),
            Self::DagJson(inner) => inner.read_with_seed(seed, reader),
        }
    }
}
