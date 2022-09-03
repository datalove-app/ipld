use crate::dev::*;
use std::convert::TryFrom;

///
#[derive(Debug)]
pub struct DagPb;

// schema! {
//     #[ipld_attr(internal)]
//     #[derive(Clone, Debug)]
//     pub type DagPbNode struct {
//         Links [DabPbLink],
//         Data optional Bytes
//     }
// }

schema! {
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type DagPbLink struct {
        Hash Link,
        Name optional String,
        Tsize optional Int,
    };
}

// pub struct DagPbNode {
//     data: &'a [u8],
//     links: Vec<DagPbLink>,
//     serializedSize: usize,
//     size: usize,
// }

// pub struct DagPbLink {
//     name: String,
//     size: usize,
//     cid: Cid,
// }

impl DagPb {
    pub const CODE: u64 = 0x70;
}

impl Into<u64> for DagPb {
    fn into(self) -> u64 {
        Self::CODE
    }
}

impl TryFrom<u64> for DagPb {
    type Error = Error;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            Self::CODE => Ok(Self),
            _ => Err(Error::UnknownMulticodec(code)),
        }
    }
}

impl Codec for DagPb {
    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write,
    {
        unimplemented!()
    }

    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        unimplemented!()
    }

    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read,
    {
        unimplemented!()
    }

    /// Given a `Read`, deserialize a dag.
    fn read_with_seed<'de, S, R>(
        &mut self,
        seed: S,
        reader: R,
    ) -> Result<<S as DeserializeSeed<'de>>::Value, Error>
    where
        S: DeserializeSeed<'de>,
        R: Read,
    {
        unimplemented!()
    }
}

// impl<'a, W: CborWrite> Encoder for &'a mut CborSerializer<W> {
//     #[inline]
//     fn serialize_link(self, cid: &Cid) -> Result<<Self as Serializer>::Ok, CborError> {
//         let vec: Vec<u8> = cid.to_bytes();
//         let bytes: &[u8] = vec.as_ref();
//         Tagged::new(Some(CBOR_LINK_TAG), bytes).serialize(self)
//     }
// }

// impl<'de, 'a, R: CborRead<'de>> Decoder<'de> for &'a mut CborDeserializer<R> {
//     #[inline]
//     fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, CborError>
//     where
//         V: IpldVisitorExt<'de>,
//     {
//         match current_cbor_tag() {
//             Some(CBOR_LINK_TAG) => {
//                 let bytes = <&[u8]>::deserialize(self)?;
//                 let cid = ToCid::to_cid(bytes)
//                     .or::<CborError>(Err(de::Error::custom("expected a CID")))?;
//                 visitor.visit_link(cid)
//             }
//             Some(_tag) => Err(de::Error::custom("unexpected CBOR tag")),
//             _ => Err(de::Error::custom("expected a CID")),
//         }
//     }
// }
