use crate::{dev::*, Error};
use derive_more::{Constructor, From};
use libipld_base::codec::CodecExt;
use std::{marker::PhantomData, ops::Range};

/// A state change operation to be applied to the `Context`.
//#[async_trait]
pub trait Command: Sized {
    /// Return value of the state change operation.
    type Result;
}

/// Dictates how much of the type to resolve. Every `Context` needs to `Handle` this command.
#[derive(Debug, From)]
pub enum ResolveRange {
    /// Fully resolve the type to bytes/a type.
    Full,

    /// Maintain the `Read`'s/`Write`'s `Seek` position and skip resolving the
    /// type, instead returning the default type/result.
    None,

    /// Move the `Read`'s/`Write`'s position by either the provided
    /// amount or by the total length of the serialized type (on reads) / type when serialized (on writes), then skip resolving the type, instead returning the default type/result.
    Seek(Option<usize>),

    /// `Seek` to `locations.start` in `Read/Write` stream, then `Read` the range of
    /// raw bytes into the type / `Write` the range of raw bytes of the
    /// type to the the writer, then `Seek` to `locations.end`.
    ///
    /// Currently only used by `String` and `Bytes` `Representations`.
    Bytes {
        locations: Range<usize>,
        range: Range<usize>,
    },
}

/// Resolving a `Cid` into a dag / block.
///
#[derive(Constructor, Debug, From)]
pub struct ResolveBlock<'a, T, Co: CodecExt<T>>(&'a Cid, PhantomData<Co>, PhantomData<T>);
impl<'a, T, Co> Command for ResolveBlock<'a, T, Co>
where
    Co: CodecExt<T>,
{
    /// Whether or not the link should be fully resolved (i.e. into a
    /// dag/into a new block)
    type Result = Option<Co>;
}

/// Finishing resolving a `Cid` into a dag / block.
#[derive(Constructor, Debug, From)]
pub struct FlushBlock<'a>(&'a Cid);
impl<'a> Command for FlushBlock<'a> {
    /// The resulting `Cid` from writing the previous block.
    type Result = Result<Cid, Error>;
}
