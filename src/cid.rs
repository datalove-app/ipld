use crate::dev::{macros::*, *};
use arrayvec::ArrayVec;
use cid::Error as CidError;
use maybestd::{cmp, convert::TryFrom, fmt, hash, io, str::FromStr};
use multibase::Error as MultibaseError;

///
pub const DEFAULT_CID_SIZE: usize = 64;

///
#[derive(Copy, Clone, Debug, Eq)]
pub struct Cid<const S: usize = DEFAULT_CID_SIZE> {
    pub(crate) inner: cid::CidGeneric<S>,
    multibase: Multibase,
}

impl<const S: usize> Cid<S> {
    /// Allocated size of the [`Cid`]'s underlying [`Multihash`], in bytes.
    pub const SIZE: usize = S;

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
    pub const fn multihash(&self) -> &DefaultMultihash {
        // self.inner.hash()
        unimplemented!()
    }

    ///
    #[inline]
    pub const fn multibase(&self) -> Multibase {
        self.multibase
    }

    ///
    #[inline]
    pub fn multicodec(&self) -> Result<Multicodec, Error> {
        Multicodec::try_from(self.multicodec_code())
    }

    ///
    #[inline]
    pub fn multihasher(&self) -> Result<Multihasher, Error> {
        Multihasher::try_from(self.multihash_code())
    }

    ///
    #[inline]
    pub fn generator(&self) -> Result<CidGenerator, Error> {
        Ok(CidGenerator::new(
            self.version(),
            self.multicodec_code(),
            self.multihasher()?,
        ))
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

    // #[inline]
    // const fn len(version: Version, mc: u64, mh: u64, digest_len: usize) -> usize {
    //     // len( varints for version, mc, mh ) + len(digest)
    //     unimplemented!()
    // }

    ///
    pub fn to_writer<W: io::Write>(writer: W) -> Result<(), Error> {
        // calculate total len
        // write varints for cid version, mc, mh
        // write digest
        unimplemented!()
    }

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
        // let multibase = Self::default_multibase(&inner);
        // Self { inner, multibase }
        unimplemented!()
    }

    /// Generates an [`Cid`] from a [`io::Read`] of a block's bytes.
    pub fn from_reader<R: io::Read>(
        cid_version: Version,
        multicodec_code: u64,
        multihash_code: u64,
        mut block: R,
    ) -> Result<Self, Error> {
        let mut generator = CidGenerator::new(
            cid_version,
            multicodec_code,
            Multihasher::try_from(multihash_code)?,
        );
        // io::copy(&mut block, &mut generator).map_err(CidError::Io)?;
        // generator.try_finalize()
        unimplemented!()
    }

    ///
    pub fn derive_from_reader<R: io::Read>(&self, block: R) -> Result<Self, Error> {
        Self::from_reader(
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

    const fn default_multibase<const Sb: usize>(cid: &CidGeneric<S>) -> Multibase {
        match cid.version() {
            Version::V0 => Multibase::Base58Btc,
            _ => Self::DEFAULT_MULTIBASE,
        }
    }
}

/*
impl<const S: usize> Representation for CidGeneric<S> {
    const NAME: &'static str = "Cid";
    const SCHEMA: &'static str = "type Cid &Any";
    const DATA_MODEL_KIND: Kind = Kind::Link;

    fn to_selected_node(&self) -> SelectedNode {
        SelectedNode::Link(*self)
    }

    ///
    #[inline]
    fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        Multicodec::serialize_link::<MC, Se>(self, serializer)
    }

    ///
    #[inline]
    fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CidVisitor;
        impl<'de> Visitor<'de> for CidVisitor {
            type Value = Cid;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "a Cid containing a Multihash of max {} bytes",
                    S
                )
            }
            #[inline]
            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_link_bytes(bytes)
            }
            #[inline]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_link_str(s)
            }
        }
        impl<'de> LinkVisitor<'de> for CidVisitor {
            #[inline]
            fn visit_cid<E>(self, cid: Cid) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(cid)
            }
        }

        if Multicodec::is_known::<MC>() {
            Multicodec::deserialize_link::<MC, _, _>(deserializer, CidVisitor)
        } else {
            Ok(Self::from(Deserialize::deserialize(deserializer)?))
        }
    }
}
 */

impl Representation for Cid {
    const NAME: &'static str = "Cid";
    const SCHEMA: &'static str = "type Cid &Any";
    const DATA_MODEL_KIND: Kind = Kind::Link;

    fn to_selected_node(&self) -> SelectedNode {
        SelectedNode::Link(*self)
    }

    ///
    #[inline]
    fn serialize<const MC: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Multicodec::serialize_link::<MC, S>(self, serializer)
    }

    ///
    #[inline]
    fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CidVisitor<const MC: u64>;
        impl<'de, const MC: u64> Visitor<'de> for CidVisitor<MC> {
            type Value = Cid;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "a Cid containing a Multihash of max {} bytes",
                    <Cid>::SIZE
                )
            }
            #[inline]
            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_link_bytes(bytes)
            }
            #[inline]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_link_str(s)
            }
        }
        impl<'de, const MC: u64> LinkVisitor<'de, MC> for CidVisitor<MC> {
            // #[inline]
            // fn visit_cid<E>(self, cid: Cid) -> Result<Self::Value, E>
            // where
            //     E: de::Error,
            // {
            //     Ok(cid)
            // }
        }

        if Multicodec::is_known::<MC>() {
            Multicodec::deserialize_link::<MC, _, _>(deserializer, CidVisitor)
        } else {
            Ok(Self::from(Deserialize::deserialize(deserializer)?))
        }
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
//     fn deserialize<const MC: u64, D>(self, deserializer: D) -> Result<(), D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         cfg_if::cfg_if! {
//             if #[cfg(feature = "dag-json")] {
//                 if MC == DagJson::CODE {
//                     DagJson::deserialize_link(deserializer, CodecSeed::<MC, _>(self))
//                 } else {
//                     // Deserialize::deserialize(deserializer)
//                     unimplemented!()
//                 }
//             } else if #[cfg(feature = "dag-cbor")] {
//                 if MC == DagJson::CODE {
//                     DagCbor::deserialize_link(deserializer, CodecSeed::<MC, _>(self))
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

impl<const Sa: usize, const Sb: usize> PartialEq<CidGeneric<Sb>> for Cid<Sa> {
    #[inline]
    fn eq(&self, other: &CidGeneric<Sb>) -> bool {
        self.version() == other.version()
            && self.multicodec_code() == other.codec()
            && self.digest() == other.hash().digest()
    }
}

impl<const Sa: usize, const Sb: usize> PartialEq<Cid<Sb>> for Cid<Sa> {
    #[inline]
    fn eq(&self, other: &Cid<Sb>) -> bool {
        self.eq(&other.inner)
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
//
// impl<'de> Deserialize<'de> for Cid {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Ok(Self::from(DefaultCid::deserialize(deserializer)?))
//     }
// }

/// A wrapper around an [`io::Write`] that can produce a [`Cid`] of the written
/// block bytes.
#[derive(
    // Builder,
    Debug,
)]
// #[builder(pattern = "owned")]
pub struct BlockWriter<W> {
    generator: CidGenerator,
    writer: W,
}

impl<W: io::Write> BlockWriter<W> {
    ///
    #[inline]
    pub const fn new(generator: CidGenerator, writer: W) -> Self {
        Self { generator, writer }
    }

    /// Tap the [`BlockWriter`] to generate a [`Cid`] from the current
    /// [`Multihasher`], resetting it.
    pub fn tap(&mut self) -> Result<Cid, Error> {
        Ok(self.generator.try_finalize()?)
    }

    // ///
    // pub fn into_inner(self) -> impl io::Write {
    //     self.writer
    // }
}

impl<W: io::Write> io::Write for BlockWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.generator.multihasher.update(&buf[..len]);
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// A wrapper around an [`io::Read`] that can produce a [`Cid`] of the read
/// block bytes, useful for pass-through verification.
#[derive(
    // Builder,
    Debug,
)]
// #[builder(pattern = "owned")]
pub struct BlockReader<R, const S: usize> {
    buf: ArrayVec<u8, S>,
    generator: CidGenerator,
    reader: R,
}

impl<const S: usize, R: io::Read> BlockReader<R, S> {
    ///
    #[inline]
    pub const fn new(generator: CidGenerator, reader: R) -> Self {
        Self {
            buf: ArrayVec::new_const(),
            generator,
            reader,
        }
    }

    /// Tap the [`BlockReader`] to generate a [`Cid`] from the current
    /// [`Multihasher`], resetting it.
    pub fn tap(&mut self) -> Result<Cid, Error> {
        Ok(self.generator.try_finalize()?)
    }

    // ///
    // pub fn into_inner(self) -> impl io::Read {
    //     self.reader
    // }
}

impl<const S: usize, R: io::Read> io::Read for BlockReader<R, S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // let len = self.reader.read(buf)?;
        // // FIXME: not recommend to read from buf
        // self.generator.multihasher.update(&buf[..len]);
        // Ok(len)
        unimplemented!()
    }
}

impl<const S: usize, R: io::Read> io::BufRead for BlockReader<R, S> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // io::BufRead::fill_buf(&mut self.reader)
        unimplemented!()
    }

    fn consume(&mut self, amt: usize) {
        // io::BufRead::consume(&mut self.reader, amt)
        unimplemented!()
    }
}

/// A generator of [`Cid`]s.
#[derive(
    // Builder,
    Debug,
)]
// #[builder(pattern = "owned")]
pub struct CidGenerator {
    pub version: Version,
    pub mc_code: u64,
    // #[builder(field(type = "u64", build = "self.multihasher.try_into()?"))]
    pub(crate) multihasher: Multihasher,
}

// impl From<Error>

impl CidGenerator {
    ///
    #[inline]
    pub const fn new(version: Version, multicodec_code: u64, multihasher: Multihasher) -> Self {
        Self {
            version,
            mc_code: multicodec_code,
            multihasher,
        }
    }

    // pub const fn with_multihasher(self, multihasher: Multihasher) -> Self {
    //     self.multihasher = multihasher;
    //     self
    // }

    ///
    #[inline]
    pub fn try_finalize(&mut self) -> Result<Cid, Error> {
        let inner = DefaultCid::new(self.version, self.mc_code, self.multihasher.try_finalize()?)?;
        Ok(Cid::from(inner))
    }
}

impl io::Write for CidGenerator {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.multihasher.update(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "dep:rkyv")]
mod rkyv {}

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
