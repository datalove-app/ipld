use crate::dev::{Multihash, *};
use macros::derive_more::From;
use multihash::Hasher;
use std::{convert::TryFrom, io::BufRead};

///
#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid(DefaultCid);

impl Cid {
    ///
    #[inline]
    pub const fn version(&self) -> Version {
        self.0.version()
    }

    ///
    #[inline]
    pub const fn multicodec_code(&self) -> u64 {
        self.0.codec()
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
        self.0.hash()
    }

    ///
    #[inline]
    pub fn digest(&self) -> &[u8] {
        self.multihash().digest()
    }

    ///
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    ///
    #[inline]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Generates an [`Cid`] from a [`BufRead`] of a block's bytes.
    pub fn generate<R: BufRead>(
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

        let mh = hasher.try_into()?;
        let cid = DefaultCid::new(cid_version, multicodec_code, mh)?;
        Ok(Self(cid))
    }
}

impl<const S: usize> PartialEq<CidGeneric<S>> for Cid {
    fn eq(&self, other: &CidGeneric<S>) -> bool {
        self.version() == other.version()
            && self.multicodec_code() == other.codec()
            && self.digest() == other.hash().digest()
    }
}

impl<'a> TryFrom<&'a [u8]> for Cid {
    type Error = Error;
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self(DefaultCid::try_from(bytes)?))
    }
}

impl<'a> TryFrom<&'a str> for Cid {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(DefaultCid::try_from(s)?))
    }
}

impl TryFrom<String> for Cid {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(Self(DefaultCid::try_from(s.as_str())?))
    }
}

/// Helper [`Visitor`] for visiting [`CidGeneric`]s.
///
/// [`Visitor`]: serde::de::Visitor
/// [`CidGeneric`]: cid::CidGeneric
#[derive(Debug, Default)]
pub struct CidVisitor;

impl<'de> Visitor<'de> for CidVisitor {
    type Value = Cid;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a CID")
    }
}

impl<'de> IpldVisitorExt<'de> for CidVisitor {
    /// The input contains the bytes of a `Cid`.
    #[inline]
    fn visit_link_str<E>(self, cid_str: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::try_from(cid_str).map_err(E::custom)
    }

    /// The input contains the bytes of a `Cid`.
    #[inline]
    fn visit_link_borrowed_str<E>(self, cid_str: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::try_from(cid_str).map_err(E::custom)
    }

    /// The input contains a string representation of a `Cid`.
    #[inline]
    fn visit_link_bytes<E>(self, cid_bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::try_from(cid_bytes).map_err(E::custom)
    }

    /// The input contains a string representation of a `Cid`.
    #[inline]
    fn visit_link_borrowed_bytes<E>(self, cid_bytes: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::try_from(cid_bytes).map_err(E::custom)
    }
}

impl Serialize for Cid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <S as Encoder>::serialize_link(serializer, self)
    }
}

impl<'de> Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = CidVisitor::default();
        let cid = <D as Decoder<'de>>::deserialize_link(deserializer, visitor)?;
        Ok(cid)
    }
}
