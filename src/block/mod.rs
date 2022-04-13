use crate::dev::*;
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    error::Error as StdError,
    marker::PhantomData,
    path::{Path, PathBuf},
};

// mod read;
// mod write;

// pub use read::BlockReader;
// pub use write::BlockWriter;

// ///
// /// TODO: replace Cid with a Link type? that way, we can assert at compile-time what we expect to `get`, but also assert that `set`ting doesnt mis-match types
// #[async_trait]
// pub trait BlockService {
//     ///
//     type Reader: BlockReader;
//     type Writer: BlockWriter;
//     type Error: Debug + StdError;
//
//     /// Interprets the `Format` to use from the `Cid`, then returns a
//     /// `BlockReader` that provides the bytes for the desired block.
//     /// TODO? we need a Decoder to go along with this
//     async fn reader(cid: &Cid) -> Result<Self::Reader, Self::Error>;
//
//     /// Interprets the `Format` to use from the `BlockMeta`, then returns a
//     /// `BlockWriter` that can be written to
//     /// TODO? take in a BlockStream?
//     async fn writer<B>(meta: B) -> Result<Self::Writer, Self::Error>
//     where
//         B: Into<BlockMeta>;
//
//     async fn multi_reader(cid: [&Cid]) -> Result<>;
//     async fn multi_writer<B>(meta: [&BlockMeta]) -> Result<>;
//
//     ///
//     async fn delete(cid: &Cid) -> Result<(), Self::Error>;
// }

///
#[derive(Debug, Clone)]
pub enum BlockMeta<'a, Si: MultihashSize = DefaultMultihashSize, So: MultihashSize = Si> {
    /// Signals to the [`Context`] that the new block should be created with
    /// the same codec and multihash, and (if applicable) should also replace an
    /// existing block with the provided `Cid`.
    ///
    /// [`Context`]
    Cid {
        cid: &'a CidGeneric<Si>,
        // path_alias: Option<PathBuf>,
    },

    /// Signals to the [`Context`] that the new block should be created with
    /// the provided multicodec and multihash.
    ///
    /// [`Context`]
    Prefix {
        version: cid::Version,
        multicodec: u64,
        multihash: u64,
        size: PhantomData<So>,
        // path_alias: Option<PathBuf>,
    },
    // /// Signals to the [`Context`] that
    // ///
    // /// [`Context`]
    // PathAlias(&'a Path),
}

impl<'a, Si: MultihashSize, So: MultihashSize> BlockMeta<'a, Si, So> {
    ///
    #[inline]
    pub fn multicodec(&self) -> Result<Multicodec, Error> {
        let code = match self {
            Self::Cid { cid } => cid.codec(),
            Self::Prefix { multicodec, .. } => *multicodec,
            // Self::PathAlias(_) => {
            //     return Err(Error::BlockMeta(
            //         "path-aliased blocks do not have multicodecs",
            //     ))
            // }
        };

        Multicodec::try_from(code)
    }

    ///
    #[inline]
    pub fn multihash(&self) -> Result<Multihash, Error> {
        let code = match self {
            Self::Cid { cid } => cid.hash().code(),
            Self::Prefix { multihash, .. } => *multihash,
            // Self::PathAlias(_) => {
            //     return Err(Error::BlockMeta(
            //         "path-aliased blocks do not have multihashes",
            //     ))
            // }
        };

        Ok(Multihash::try_from(code)?)
    }

    ///
    #[inline]
    pub fn prefix(&self) -> (cid::Version, u64, u64) {
        match self {
            Self::Cid { cid, .. } => (cid.version(), cid.codec(), cid.hash().code()),
            Self::Prefix {
                version,
                multicodec,
                multihash,
                size: PhantomData,
                ..
            } => (*version, *multicodec, *multihash),
            // Self::PathAlias(_) => None,
        }
    }

    ///
    #[inline]
    pub fn from_prefix(multicodec: u64, multihash: u64, path_alias: Option<PathBuf>) -> Self {
        Self::Prefix {
            version: if multicodec == DagPb::CODE {
                cid::Version::V0
            } else {
                cid::Version::V1
            },
            multicodec,
            multihash,
            size: PhantomData,
            // path_alias,
        }
    }

    ///
    #[inline]
    pub fn from_link(cid: &'a CidGeneric<Si>) -> Self {
        Self::Cid { cid }
    }

    // #[inline]
    // pub fn hasher(&self) -> Option<Box<dyn MultihashDigest>> {
    //     self.prefix().mh_type.hasher()
    // }
}

impl<'a, Si: MultihashSize, So: MultihashSize> Default for BlockMeta<'a, Si, So> {
    #[inline]
    fn default() -> Self {
        BlockMeta::from_prefix(DagCbor::CODE, multihash::Code::Sha2_256.into(), None)
    }
}

impl<'a, Si: MultihashSize, So: MultihashSize> From<&'a CidGeneric<Si>> for BlockMeta<'a, Si, So> {
    #[inline]
    fn from(cid: &'a CidGeneric<Si>) -> Self {
        BlockMeta::Cid {
            cid,
            // path_alias: None,
        }
    }
}

// impl From<cid::Prefix> for BlockMeta {
//     #[inline]
//     fn from(prefix: cid::Prefix) -> Self {
//         BlockMeta::Prefix(prefix)
//     }
// }

// impl<'a, Si: MultihashSize> From<u64> for BlockMeta<'a, Si> {
//     #[inline]
//     fn from(multicodec: u64) -> Self {
//         BlockMeta::from_prefix(multicodec, multihash::Code::Sha2_256.into())
//     }
// }

impl<'a, Si: MultihashSize, So: MultihashSize> From<multihash::Code> for BlockMeta<'a, Si, So> {
    #[inline]
    fn from(multihash: multihash::Code) -> Self {
        BlockMeta::from_prefix(DagCbor::CODE, multihash.into(), None)
    }
}

impl<'a, Si: MultihashSize, So: MultihashSize> From<(u64, u64)> for BlockMeta<'a, Si, So> {
    #[inline]
    fn from(prefix: (u64, u64)) -> Self {
        BlockMeta::from_prefix(prefix.0, prefix.1, None)
    }
}

// impl<T> From<Option<T>> for BlockMeta
// where
//     T: Into<BlockMeta>,
// {
//     #[inline]
//     fn from(meta: Option<T>) -> Self {
//         match meta {
//             None => Self::default(),
//             Some(meta) => meta.into(),
//         }
//     }
// }

// impl<'a, Si: MultihashSize, So: MultihashSize> From<&'a Path> for BlockMeta<'a, Si, So> {
//     #[inline]
//     fn from(path: &'a Path) -> Self {
//         BlockMeta::PathAlias(path.into())
//     }
// }

#[derive(Debug)]
pub struct Block<'a, Si: MultihashSize = DefaultMultihashSize> {
    meta: BlockMeta<'a, Si>,
    bytes: bytes::Bytes,
}
