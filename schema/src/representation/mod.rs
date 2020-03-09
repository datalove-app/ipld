//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.

// pub mod context;

use async_trait::async_trait;
use ipld::prelude::*;


pub struct Executor<'a, C> {
    context: &'a C,
}

impl<'a, C> Executor<'a, C> {
    pub fn resolve<T>(&mut self, ipld: &T) -> Result<(), ()>
    where
        T: Representation<C>
    {
        Ok(ipld.resolve())
    }

    pub fn context(&self) -> &'a C {
        self.context
    }
}

///
#[async_trait]
pub trait Context {
    /// Internally, this will:
    ///     - get a (concrete?) BlockWriter from a BlockService
    ///     - determine the Codec + Format from the BlockMeta
    ///         - create a
    ///
    /// ## Example:
    /// ```
    /// Context::write(&ipld).await?;
    /// ```
    async fn write<R, B>(&self, ipld: &R, block_meta: B) -> Result<(), ()>
    where
        R: Representation,
        B: Into<BlockMeta>;
}
/// TODO: distinguish between containing links and possessing links
/// TODO:

///
/// TODO:
#[async_trait]
pub trait Representation<C>: Serialize + DeserializeOwned {
    const HAS_LINKS: bool = false;
    const IS_LINK: bool = false;

    ///
    /// if you have links (any recursive that has links directly, or contains types that have links)
    ///     recurse
    ///     ...
    ///     ... ???
    /// if you are a linked dag (a link with a CID and type)
    ///     get a block service from the Context
    ///     BlockService.put
    ///     replace link CID + dag with new CID
    ///     return Ok
    /// else (any primitive, or a link with only a CID)
    ///     serialize yourself with provided Serializer
    ///     return
    ///
    async fn write<S, C>(&self, serializer: S, context: C) -> Result<(), ()>
    where
        S: Serializer,
        C: Context;

    // TODO: should return Result<Self>
    async fn read<'de, D, C>(deserializer: D, context: C) -> Result<(), ()>
    where
        D: Deserializer<'de>,
        C: Context;
}

impl Representation for () {
    async fn write<S, C>(&self, serializer: S, context: C) -> Result<(), ()>
    where
        S: Serializer,
        C: Context,
    {
        Ok(())
    }

    async fn read<'de, D, C>(deserializer: D, context: C) -> Result<(), ()>
    where
        D: Deserializer<'de>,
        C: Context,
    {
        Ok(())
    }
}

impl<R: Representation> Representation for (Cid, Option<R>) {
    async fn write<S, C>(&self, serializer: S, context: C) -> Result<(), ()>
    where
        S: Serializer,
        C: Context,
    {
        Ok(())
    }

    async fn read<'de, D, C>(deserializer: D, context: C) -> Result<(), ()>
    where
        D: Deserializer<'de>,
        C: Context,
    {
        Ok(())
    }
}

// /// An interface for `Encode`ing and `Decode`ing an IPLD Representation.
// ///
// /// Types that have `Representation`s generally follow the same few steps when
// /// encoding (in reverse for decoding):
// ///     - pre-processing, i.e.:
// ///         fetching codecs
// ///         generating signatures
// ///         converting bytes to hex
// ///     - (? optionally) conversion of the type to an Ipld-like
// ///         helpful for ensuring canonicalization
// ///     - serializing the Ipld-like type with a provided Codec
// /// decoding:
// ///     - pre-processing, i.e.:
// ///         fetching blocks
// ///     - deserializing either:
// ///         - to an Ipld-like type, then conversion to native type
// ///         - to a native type directly
// ///
// /// The supplied execution `Context` provides `Codec` to use, and can also:
// ///     - dictate which fields to `Read`/`Write`,
// ///     - provide a source/sink of bytes for a particular `Cid`/`Block`
// #[async_trait]
// pub trait Representation: Sized {
//     type Context = Context;

//     /// Encodes a type to a provided `Context`.
//     ///
//     /// By default, creates an IPLD data type representation from the type, then
//     /// encodes the `Ipld` with the provided `Codec`.
//     async fn encode(
//         &self,
//         ctx: &Self::Context,
//     ) -> Result<Option<Cid>, <Self::Context as Context>::Error> {
//         //        let dag = self.to_ipld(ctx).await?;
//         //        ctx.codec().encode(dag)?
//     }

//     /// `Read` a type from a provided `Context`.
//     async fn decode(
//         bytes: &[u8],
//         ctx: &Self::Context,
//     ) -> Result<Self, <Self::Context as Context>::Error> {
//         //        let dag = ctx.codec().decode(bytes).await?;
//         //        Self::from_ipld(dag, ctx)
//     }

//     //    /// `Read` a type from a provided `Context`.
//     //    async fn read_with_ctx<NewCtx>(ctx: &Ctx) -> Result<Self, Error>
//     //    where
//     //        NewCtx: FromContext<Ctx>,
//     //        Self: Representation<NewCtx>;

//     //    /// `Write` a type to a provided `Context`.
//     //    async fn write_with_ctx<NewCtx>(&self, ctx: &Ctx) -> Result<(), Error>
//     //    where
//     //        Co: 'async_trait,
//     //        R: 'async_trait,
//     //        W: 'async_trait,
//     //        NewCtx: FromContext<Ctx>;
// }
