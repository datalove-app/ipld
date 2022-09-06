use crate::dev::{Multihash, *};
use macros::derive_more::From;
use multihash::Hasher;
use std::{convert::TryFrom, fmt, io::BufRead};

///
#[derive(Copy, Clone, Debug, Default, Eq, From, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid(cid::CidGeneric<{ Self::SIZE }>);

impl Cid {
    /// Allocated size of the [`Cid`]'s underlying [`Multihash`], in bytes.
    pub const SIZE: usize = 64;

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
    pub fn new<R: BufRead>(
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

    ///
    pub fn derive_new<R: BufRead>(&self, block: R) -> Result<Self, Error> {
        Self::new(
            self.version(),
            self.multicodec_code(),
            self.multihash_code(),
            block,
        )
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
        Ok(Self(DefaultCid::read_bytes(bytes)?))
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

// impl Representation for Cid {
//     const NAME: &'static str = "Link";
//     const SCHEMA: &'static str = "type Link &Any";
//     const DATA_MODEL_KIND: Kind = Kind::Link;
//     const IS_LINK: bool = true;
// }

// impl_ipld_serde! { @context_visitor {} {} Cid {
//     #[inline]
//     fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", Cid::NAME)
//     }
// }}

// impl_ipld_serde! { @context_visitor_ext {} {} Cid {
//     #[inline]
//     fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", Cid::NAME)
//     }
// }}

// impl_ipld_serde! { @context_select
//     { T: Representation + Send + Sync + 'static }
//     { for<'b, 'de> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
//     List<T>
// }

// impl_ipld_serde! { @select_with_seed
//     { T: Representation + Send + Sync + 'static }
//     { for<'b, 'de> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
//     Link<T>
// }

/// Helper [`Visitor`] for visiting [`CidGeneric`]s.
///
/// [`Visitor`]: serde::de::Visitor
/// [`CidGeneric`]: cid::CidGeneric
#[derive(Debug, Default)]
pub struct CidVisitor;

impl<'de> Visitor<'de> for CidVisitor {
    type Value = Cid;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a Cid of a multihash no longer than {} bytes", Cid::SIZE)
    }
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
        let cid = <D as Decoder<'de>>::deserialize_link(deserializer, CidVisitor)?;
        Ok(cid)
    }
}
