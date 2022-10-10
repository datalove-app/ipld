use crate::dev::{macros::*, *};
use cid::Error as CidError;
use maybestd::{cmp, convert::TryFrom, fmt, hash, io, str::FromStr};
use multibase::Error as MultibaseError;

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
    pub fn new<R: io::Read>(
        cid_version: Version,
        multicodec_code: u64,
        multihash_code: u64,
        block: R,
    ) -> Result<Self, Error> {
        let hasher = Multihash::try_from(multihash_code)?;
        CidGenerator::new(cid_version, multicodec_code, hasher).derive(block)
    }

    ///
    pub fn derive_new<R: io::Read>(&self, block: R) -> Result<Self, Error> {
        Self::new(
            self.version(),
            self.multicodec_code(),
            self.multihash_code(),
            block,
        )?
        .with_multibase(self.multibase)
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
    type ReprKind = type_kinds::Link;

    const NAME: &'static str = "Cid";
    const SCHEMA: &'static str = "type Cid &Any";
    const DATA_MODEL_KIND: Kind = Kind::Link;

    ///
    #[inline]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[cfg(feature = "dag-json")]
        if C == DagJson::CODE {
            return DagJson::serialize_cid(self, serializer);
        }
        #[cfg(feature = "dag-cbor")]
        if C == DagCbor::CODE {
            return DagCbor::serialize_cid(self, serializer);
        }

        Serialize::serialize(&self.inner, serializer)
    }

    ///
    #[inline]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CidVisitor;
        impl<'de> Visitor<'de> for CidVisitor {
            type Value = Cid;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a Cid containing a Multihash of max {} bytes", Cid::SIZE)
            }
        }
        impl<'de> IpldVisitorExt<'de> for CidVisitor {
            #[inline]
            fn visit_cid<E>(self, cid: Cid) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(cid)
            }
        }

        #[cfg(feature = "dag-json")]
        if C == DagJson::CODE {
            return DagJson::deserialize_cid(deserializer, CidVisitor);
        }
        #[cfg(feature = "dag-cbor")]
        if C == DagCbor::CODE {
            return DagCbor::deserialize_cid(deserializer, CidVisitor);
        }

        Ok(Self::from(Deserialize::deserialize(deserializer)?))
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
            _ => s
                .chars()
                .next()
                .ok_or_else(|| MultibaseError::InvalidBaseString)
                .and_then(Multibase::from_code)?,
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

impl FromStr for Cid {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

// TODO: support direct encoding to formatter, rather than allocating a string
impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// /// Defaults to the `Serialize` impl of `CidGeneric<S>`.
// impl Serialize for Cid {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         self.inner.serialize(serializer)
//     }
// }

// impl<'de> Deserialize<'de> for Cid {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Ok(Self::from(DefaultCid::deserialize(deserializer)?))
//     }
// }

/// A generator of [`Cid`]s.
#[derive(Debug)]
pub struct CidGenerator {
    version: Version,
    mc_code: u64,
    hasher: Multihash,
}

impl CidGenerator {
    ///
    pub fn new(version: Version, multicodec_code: u64, hasher: Multihash) -> Self {
        Self {
            version,
            mc_code: multicodec_code,
            hasher,
        }
    }

    fn derive<R: io::Read>(mut self, mut block: R) -> Result<Cid, Error> {
        io::copy(&mut block, &mut self).map_err(CidError::Io)?;
        let inner = DefaultCid::new(self.version, self.mc_code, self.hasher.try_finalize()?)?;
        Ok(Cid::from(inner))
    }
}

impl multihash::Hasher for CidGenerator {
    #[inline]
    fn update(&mut self, input: &[u8]) {
        self.hasher.update(input)
    }
    #[inline]
    fn finalize(&mut self) -> &[u8] {
        self.hasher.finalize()
    }
    #[inline]
    fn reset(&mut self) {
        self.hasher.reset()
    }
}

impl std::io::Write for CidGenerator {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
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
