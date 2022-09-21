use crate::prelude::*;

/// The GitRaw codec.
pub struct GitRaw;

pub struct Commit {
    tree: Cid,
    parents: Vec<Cid>,
    author: String,
    committer: String,
    encoding: String,
    #[serde(rename = "mergetag")]
    merge_tag: Vec<Tag>,
    signature: String,
    message: String,
}

pub struct Tag {}
pub struct Tree {}

impl Codec for DagPb {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::GitRaw;

    type Error = CborError;

    fn encode<S>(dag: &S) -> Result<Box<[u8]>, Self::Error>
    where
        S: Serialize,
    {
    }

    fn decode<'de, D>(bytes: &'de [u8]) -> Result<D, Self::Error>
    where
        D: Deserialize<'de>,
    {
    }

    fn write<S, W>(dag: &S, writer: W) -> Result<(), Self::Error>
    where
        S: Serialize,
        W: Write,
    {
    }

    fn read<D, R>(reader: R) -> Result<D, Self::Error>
    where
        D: DeserializeOwned,
        R: Read,
    {
    }
}

impl<'a, W: CborWrite> Encoder for &'a mut CborSerializer<W> {
    #[inline]
    fn serialize_link(self, cid: &Cid) -> Result<<Self as Serializer>::Ok, CborError> {
        let vec: Vec<u8> = cid.to_bytes();
        let bytes: &[u8] = vec.as_ref();
        Tagged::new(Some(CBOR_LINK_TAG), bytes).serialize(self)
    }
}

impl<'de, 'a, R: CborRead<'de>> Decoder<'de> for &'a mut CborDeserializer<R> {
    #[inline]
    fn deserialize_link<V>(self, visitor: V) -> Result<V::Value, CborError>
    where
        V: IpldVisitorExt<'de>,
    {
        match current_cbor_tag() {
            Some(CBOR_LINK_TAG) => {
                let bytes = <&[u8]>::deserialize(self)?;
                let cid = ToCid::to_cid(bytes)
                    .or::<CborError>(Err(de::Error::custom("expected a CID")))?;
                visitor.visit_link(cid)
            }
            Some(_tag) => Err(de::Error::custom("unexpected CBOR tag")),
            _ => Err(de::Error::custom("expected a CID")),
        }
    }
}
