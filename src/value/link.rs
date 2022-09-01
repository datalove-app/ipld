use crate::dev::*;
use macros::derive_more::From;
use std::{convert::TryFrom, io::BufRead};

pub use anycid::*;
mod anycid {
    use super::*;

    macro_rules! impl_multihasher {
        (@multihash $(
            $(#[$meta:meta])*
            $variant:ident ($code:expr) -> $ty:ty [$size:expr],
        )*) => {
            #[derive(Debug)]
            pub enum Multihasher {
                $(
                    $(#[$meta])*
                    $variant($ty),
                )*
            }

            impl Multihasher {
                ///
                pub fn new(multihash_code: u64) -> Result<Self, Error> {
                    let mh_type = Multihashes::try_from(multihash_code)?;
                    match mh_type {
                        $(Multihashes::$variant => {
                            Ok(Self::$variant(<$ty>::default()))
                        },)*
                        _ => Err(multihash::Error::UnsupportedCode(multihash_code).into()),
                    }
                }

                ///
                pub const fn code(&self) -> u64 {
                    match self {
                        $(Self::$variant(_) => $code,)*
                    }
                }

                ///
                pub const fn size(&self) -> u8 {
                    match self {
                        $(Self::$variant(_) => $size,)*
                    }
                }
            }

            impl Hasher for Multihasher {
                fn update(&mut self, input: &[u8]) {
                    match self {
                        $(Self::$variant(hasher) => hasher.update(input),)*
                    }
                }
                fn finalize(&mut self) -> &[u8] {
                    match self {
                        $(Self::$variant(hasher) => hasher.finalize(),)*
                    }
                }
                fn reset(&mut self) {
                    match self {
                        $(Self::$variant(hasher) => hasher.reset(),)*
                    }
                }
            }

            impl TryInto<DefaultMultihash> for Multihasher {
                type Error = Error;
                fn try_into(mut self) -> Result<DefaultMultihash, Self::Error> {
                    Ok(DefaultMultihash::wrap(self.code(), self.finalize())?)
                }
            }
        };
    }

    impl_multihasher! {@multihash
        ///
        Sha2_256 (0x12) -> multihash::Sha2_256 [32],
        ///
        Sha2_512 (0x13) -> multihash::Sha2_512 [64],
        ///
        Sha3_224 (0x17) -> multihash::Sha3_224 [28],
        ///
        Sha3_256 (0x16) -> multihash::Sha3_256 [32],
        ///
        Sha3_384 (0x15) -> multihash::Sha3_384 [48],
        ///
        Sha3_512 (0x14) -> multihash::Sha3_512 [64],
        ///
        Keccak224 (0x1a) -> multihash::Keccak224 [28],
        ///
        Keccak256 (0x1b) -> multihash::Keccak256 [32],
        ///
        Keccak384 (0x1c) -> multihash::Keccak384 [48],
        ///
        Keccak512 (0x1d) -> multihash::Keccak512 [64],
        ///
        Blake2b256 (0xb220) -> multihash::Blake2bHasher::<32> [32],
        ///
        Blake2b512 (0xb240) -> multihash::Blake2bHasher::<64> [64],
        ///
        Blake2s128 (0xb250) -> multihash::Blake2sHasher::<16> [16],
        ///
        Blake2s256 (0xb260) -> multihash::Blake2sHasher::<32> [32],
        ///
        Blake3_256 (0x1e) -> multihash::Blake3Hasher::<32> [32],
    }

    ///
    #[derive(Copy, Clone, Debug, Default, Eq, From, Hash, Ord, PartialEq, PartialOrd)]
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
            let mut hasher = Multihasher::new(multihash_code)?;

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
        fn try_from(str: &'a str) -> Result<Self, Self::Error> {
            Ok(Self(DefaultCid::try_from(str)?))
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
}

///
#[derive(Clone, Debug, Eq, From, PartialEq)]
pub enum Link<T: Representation = Value> {
    ///
    Cid(Cid),

    ///
    #[from(ignore)]
    Inner { cid: Cid, t: T, dirty: bool },
}

impl<T: Representation> Link<T> {
    ///
    #[inline]
    pub const fn cid(&self) -> &Cid {
        match self {
            Self::Cid(inner) => inner,
            Self::Inner { cid, .. } => cid,
        }
    }

    /*
    ///
    #[inline]
    pub fn to_meta(&self) -> BlockMeta<'_, S> {
        self.cid().into()
    }

    ///
    #[inline]
    pub fn to_meta_prefix(&self) -> BlockMeta<'_, S> {
        let cid = self.cid();
        BlockMeta::from_prefix(cid.codec(), cid.hash().code(), None)
    }
     */
}

impl<T: Representation> Into<Cid> for Link<T> {
    fn into(self) -> Cid {
        match self {
            Self::Cid(cid) => cid,
            Self::Inner { cid, .. } => cid,
        }
    }
}

impl<T: Representation> Representation for Link<T> {
    const NAME: &'static str = concat!("Link<", stringify!(T::NAME), ">");
    const SCHEMA: &'static str = "type Link link";
    const KIND: Kind = Kind::Link;
    const IS_LINK: bool = true;
    const HAS_LINKS: bool = true;

    fn name(&self) -> &'static str {
        match self {
            Self::Cid(_) => Self::NAME,
            Self::Inner { t, .. } => t.name(),
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Self::Cid(_) => Self::KIND,
            Self::Inner { t, .. } => t.kind(),
        }
    }

    fn has_links(&self) -> bool {
        match self {
            Self::Cid(_) => T::HAS_LINKS,
            Self::Inner { t, .. } => t.has_links(),
        }
    }
}

impl<'de, 'a, C, T> Visitor<'de> for ContextSeed<'a, C, Link<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", Link::<T>::NAME)
    }
}

impl<'de, 'a, C, T> IpldVisitorExt<'de> for ContextSeed<'a, C, Link<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
{
    // TODO:
}

impl<'de, 'a, C, T> DeserializeSeed<'de> for ContextSeed<'a, C, Link<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_link(self)
    }
}

// impl<'a, 'de, C, T> Select<C> for Link<T>
// where
//     C: Context,
//     T: Representation + Send + Sync + 'static,
//     // ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
// }

impl_ipld_serde! { @select_with_seed
    { T: Representation + Send + Sync + 'static }
    { for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
    Link<T>
}

////////////////////////////////////////////////////////////////////////////////
// additional implementations
////////////////////////////////////////////////////////////////////////////////

// impl<const SI: usize, const SO: usize, T> From<CidGeneric<SI>> for Link<SO, T>
// where
//     T: Representation,
// {
//     fn from(cid: CidGeneric<SI>) -> Self {
//         Self::Cid(Cid::Generic(cid))
//     }
// }

// impl<const S: usize, T> From<Link<T>> for CidGeneric<S>
// where
//     T: Representation,
// {
//     fn from(link: Link<T>) -> Self {
//         match link {
//             Link::Cid(Cid::Generic(inner)) => inner,
//             Link::Inner { cid, .. } => cid,
//         }
//     }
// }

// TODO dirty links?
impl<T> Serialize for Link<T>
where
    T: Representation,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        <Se as Encoder>::serialize_link(serializer, self.cid())
    }
}

impl<'de, T> Deserialize<'de> for Link<T>
where
    T: Representation,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::Cid(Cid::deserialize(deserializer)?))
    }
}

// impl<'de, C, H> Visitor<'de> for CidGeneric<C, H>
// where
//     C: Into<u64> + TryFrom<u64> + Copy,
//     <C as TryFrom<u64>>::Error: Debug,
//     H: Into<u64> + TryFrom<u64> + Copy,
//     <H as TryFrom<u64>>::Error: Debug,
// {
// }

// impl<'de, C, H> IpldVisitorExt<'de> for CidGeneric<C, H>
// where
//     C: Into<u64> + TryFrom<u64> + Copy,
//     <C as TryFrom<u64>>::Error: Debug,
//     H: Into<u64> + TryFrom<u64> + Copy,
//     <H as TryFrom<u64>>::Error: Debug,
// {
// }
