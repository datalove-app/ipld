use crate::dev::{macros::*, *};
use cid::Error as CidError;
use multibase::Error as MultibaseError;
use std::{cmp, convert::TryFrom, fmt, hash, io};

///
#[derive(Copy, Clone, Debug, Eq)]
pub struct Cid {
    inner: cid::CidGeneric<{ Self::SIZE }>,
    multibase: Multibase,
}

impl Cid {
    /// Allocated size of the [`Cid`]'s underlying [`Multihash`], in bytes.
    pub const SIZE: usize = 64;

    /// The default [`Multibase`] to use when encoding a non-v0 [`Cid`] as a
    /// string.
    pub const DEFAULT_MULTIBASE: Multibase = Multibase::Base32Lower;

    ///
    #[inline]
    pub const fn version(&self) -> Version {
        self.inner.version()
    }

    ///
    #[inline]
    pub const fn multicodec_code(&self) -> u64 {
        self.inner.codec()
    }

    ///
    #[inline]
    pub const fn multihash_code(&self) -> u64 {
        self.multihash().code()
    }

    // ///
    // #[inline]
    // pub const fn len(&self) -> Option<u8> {
    //     use Version::*;
    //     match (self, self.version()) {
    //         (_, V0) => Some(34),
    //         (Self(cid), V1) => Some(4 + cid.hash().size()),
    //         _ => None,
    //     }
    // }

    ///
    #[cfg(feature = "multicodec")]
    #[inline]
    pub fn multicodec(&self) -> Result<Multicodec, Error> {
        Multicodec::try_from(self.multicodec_code())
    }

    ///
    #[inline]
    pub const fn multihash(&self) -> &DefaultMultihash {
        self.inner.hash()
    }

    ///
    #[inline]
    pub const fn multibase(&self) -> Multibase {
        self.multibase
    }

    ///
    #[inline]
    pub fn digest(&self) -> &[u8] {
        self.multihash().digest()
    }

    // #[inline]
    // pub fn write_bytes<W: io::Write>(&self, writer: W) -> Result<(), Error> {
    //     unimplemented!()
    // }

    // #[inline]
    // pub fn write_str<W: fmt::Write>(&self, writer: W) -> Result<(), Error> {
    //     unimplemented!()
    // }

    ///
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes()
    }

    ///
    #[inline]
    pub fn to_string(&self) -> Result<String, Error> {
        let s = self.inner.to_string_of_base(self.multibase)?;
        Ok(s)
    }

    ///
    #[inline]
    pub const fn from(inner: DefaultCid) -> Self {
        let multibase = Self::default_multibase(&inner);
        Self { inner, multibase }
    }

    /// Generates an [`Cid`] from a [`BufRead`] of a block's bytes.
    pub fn new<R: io::BufRead>(
        cid_version: Version,
        multicodec_code: u64,
        multihash_code: u64,
        mut block: R,
    ) -> Result<Self, Error> {
        let mut hasher = Multihash::try_from(multihash_code)?;

        loop {
            let bytes = block.fill_buf().map_err(multihash::Error::Io)?;
            match bytes.len() {
                0 => break,
                len => {
                    hasher.update(bytes);
                    block.consume(len);
                }
            }
        }

        let mh = hasher.finalize()?;
        let inner = DefaultCid::new(cid_version, multicodec_code, mh)?;
        Ok(Self::from(inner))
    }

    ///
    pub fn derive_new<R: io::BufRead>(&self, block: R) -> Result<Self, Error> {
        Self::new(
            self.version(),
            self.multicodec_code(),
            self.multihash_code(),
            block,
        )
    }

    ///
    pub const fn with_multibase(self, multibase: Multibase) -> Result<Self, Error> {
        match (self.version(), multibase) {
            (Version::V0, Multibase::Base58Btc) => Ok(self),
            (Version::V0, _) => Err(Error::Cid(CidError::InvalidCidV0Base)),
            (_, multibase) => Ok(Self {
                inner: self.inner,
                multibase,
            }),
        }
    }

    const fn default_multibase<const S: usize>(cid: &CidGeneric<S>) -> Multibase {
        match cid.version() {
            Version::V0 => Multibase::Base58Btc,
            _ => Self::DEFAULT_MULTIBASE,
        }
    }
}

impl Representation for Cid {
    const NAME: &'static str = "Cid";
    const SCHEMA: &'static str = "type Cid &Any";
    const DATA_MODEL_KIND: Kind = Kind::Link;

    ///
    #[inline]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        cfg_if::cfg_if! {
            if #[cfg(feature = "dag-json")] {
                if C == DagJson::CODE {
                    return DagJson::serialize_link(self, serializer);
                }
            }
        }
        cfg_if::cfg_if! {
            if #[cfg(feature = "dag-cbor")] {
                if C == DagCbor::CODE {
                    return DagCbor::serialize_link(self, serializer);
                }
            }
        }

        Serialize::serialize(self, serializer)
    }

    ///
    #[inline]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        cfg_if::cfg_if! {
            if #[cfg(feature = "dag-json")] {
                if C == DagJson::CODE {
                    return DagJson::deserialize_link(deserializer, CidVisitor);
                }
            }
        }
        cfg_if::cfg_if! {
            if #[cfg(feature = "dag-cbor")] {
                if C == DagCbor::CODE {
                    return DagCbor::deserialize_link(deserializer, CidVisitor);
                }
            }
        }

        Deserialize::deserialize(deserializer)
    }
}

// impl_selector_seed_serde! { @codec_seed_visitor {} {} Cid {
//     #[inline]
//     fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "a {}", Cid::NAME)
//     }
// }}
//
// impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} Cid {} }
//
// impl_selector_seed_serde! { @selector_seed_codec_deseed {} {} Cid {
//     #[inline]
//     fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<(), D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         cfg_if::cfg_if! {
//             if #[cfg(feature = "dag-json")] {
//                 if C == DagJson::CODE {
//                     DagJson::deserialize_link(deserializer, CodecSeed::<C, _>(self))
//                 } else {
//                     // Deserialize::deserialize(deserializer)
//                     unimplemented!()
//                 }
//             } else if #[cfg(feature = "dag-cbor")] {
//                 if C == DagJson::CODE {
//                     DagCbor::deserialize_link(deserializer, CodecSeed::<C, _>(self))
//                 } else {
//                     // Deserialize::deserialize(deserializer)
//                     unimplemented!()
//                 }
//             } else {
//                 // Deserialize::deserialize(deserializer)
//                 unimplemented!()
//             }
//         }
//     }
// }}
//
// impl_selector_seed_serde! { @selector_seed_select {} {} Cid {} }

impl Default for Cid {
    fn default() -> Self {
        Self::from(DefaultCid::default())
    }
}

impl hash::Hash for Cid {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.inner, state)
    }
}

impl Ord for Cid {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl PartialOrd for Cid {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.inner)
    }
}

impl PartialOrd<DefaultCid> for Cid {
    fn partial_cmp(&self, other: &DefaultCid) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other)
    }
}

impl PartialEq for Cid {
    fn eq(&self, other: &Self) -> bool {
        self.eq(&other.inner)
    }
}

impl<const S: usize> PartialEq<CidGeneric<S>> for Cid {
    #[inline]
    fn eq(&self, other: &CidGeneric<S>) -> bool {
        self.version() == other.version()
            && self.multicodec_code() == other.codec()
            && self.digest() == other.hash().digest()
    }
}

impl From<DefaultCid> for Cid {
    fn from(inner: DefaultCid) -> Self {
        Self::from(inner)
    }
}

impl<'a> TryFrom<&'a [u8]> for Cid {
    type Error = Error;
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self::from(DefaultCid::read_bytes(bytes)?))
    }
}

impl<'a> TryFrom<&'a str> for Cid {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let inner = DefaultCid::try_from(s)?;
        let multibase = match inner.version() {
            Version::V0 => Multibase::Base58Btc,
            _ => {
                let code = s
                    .chars()
                    .next()
                    .ok_or_else(|| Error::Multibase(MultibaseError::InvalidBaseString))?;
                Multibase::from_code(code)?
            }
        };

        Ok(Self { inner, multibase })
    }
}

impl TryFrom<String> for Cid {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

/// Defaults to the `Serialize` impl of `CidGeneric<S>`.
impl Serialize for Cid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // cfg_if::cfg_if! {
        //     if #[cfg(feature = "serde-codec")] {
        //         (&mut &mut &mut Encoder(serializer)).serialize_link(self)
        //     } else {
        //         self.inner.serialize(serializer)
        //     }
        // }
        // serializer.serialize_link(self)

        self.inner.serialize(serializer)
    }
}

struct CidVisitor;
impl<'de> Visitor<'de> for CidVisitor {
    type Value = Cid;
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a Cid containing a multihash no longer than {} bytes",
            Cid::SIZE
        )
    }
    // #[inline]
    // fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    // where
    //     E: de::Error,
    // {
    //     self.visit_link_str(s)
    // }
    // #[inline]
    // fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    // where
    //     E: de::Error,
    // {
    //     self.visit_link_bytes(bytes)
    // }
}
impl<'de> IpldVisitorExt<'de> for CidVisitor {
    #[inline]
    fn visit_link_str<E>(self, cid_str: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::try_from(cid_str).map_err(E::custom)
    }
    #[inline]
    fn visit_link_bytes<E>(self, cid_bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::try_from(cid_bytes).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // struct CidVisitor;
        // impl<'de> Visitor<'de> for CidVisitor {
        //     type Value = Cid;
        //     #[inline]
        //     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //         write!(f, "a Cid of a multihash no longer than {} bytes", Cid::SIZE)
        //     }
        // }
        // impl<'de> IpldVisitorExt<'de> for CidVisitor {
        //     #[inline]
        //     fn visit_link_str<E>(self, cid_str: &str) -> Result<Self::Value, E>
        //     where
        //         E: de::Error,
        //     {
        //         Self::Value::try_from(cid_str).map_err(E::custom)
        //     }
        //     #[inline]
        //     fn visit_link_bytes<E>(self, cid_bytes: &[u8]) -> Result<Self::Value, E>
        //     where
        //         E: de::Error,
        //     {
        //         Self::Value::try_from(cid_bytes).map_err(E::custom)
        //     }
        // }

        // cfg_if::cfg_if! {
        //     if #[cfg(feature = "serde-codec")] {
        //         (&mut &mut &mut Decoder(deserializer)).deserialize_link(CidVisitor)
        //     } else {
        //         Ok(Self::from(DefaultCid::deserialize(deserializer)?))
        //     }
        // }

        // deserializer.deserialize_link(CidVisitor)

        Ok(Self::from(DefaultCid::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_dag_json() {
        let mut codec = DagJson::new();
        let mut bytes = Vec::new();
        codec.write(&Cid::default(), &mut bytes).unwrap();

        println!("json str: {}", std::str::from_utf8(&bytes).unwrap());

        assert!(false)
    }
}
