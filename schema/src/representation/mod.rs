//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.

mod context;
mod implementations;

use crate::{executor::Executor, selectors::Selector};
pub use context::*;
use ipld::dev::*;
use serde::de::DeserializeSeed;
use std::convert::TryFrom;

/// TODO: distinguish between containing links and possessing links
/// TODO:

///
#[derive(Debug, Eq, PartialEq)]
pub enum SchemaKind {
    ///
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Bytes,
    List,
    Map,
    Link,
    Struct,
    Enum,
    Union,
    Copy,
}

/// TODO? == what are the requirements?
///     - serialize a type to a block
///         - serialize an ipld as this type to a block
///     - deserialize a type from a block
///         -
///         - deserialize an ipld as this type from a block
///     - focus a type to a value (typed or ipld?) based on a selector
///         - ? call a closure on the selected type?
///     - transform a value within a type based on a selector (? and a closure ?)
///         - return a typed value or ipld?
///
/// TODO? selection can only happen *accurately* against fully-resolved types and blocks
///
/// TODO: what to impl?
///     - focus<T>(&self, selector, context) -> Result<T>
///     - patch<T, F>(&mut self, selector, f: F, context) -> Result<()>
///         where F: Fn(&mut T, context);
///         - based on success of recursing, flags any link type as dirty
///     - flush(&self, context) -> Result<Selector>
///     TODO? << other impls >>
///     - validate_selector(selector)
///         - TODO: ? returns a stateful Visitor + DeserializeSeed?
///     - derive Serialize
///     - in focus<T>(...), impl Deserialize
///         - TODO: ? stateful visitor derived from selector + type?
///         - TODO: ? impl DeserializeSeed for selector?
///         - TODO: ? Representation::visitor(selector: &Selector)
///
///
///
///
///
#[async_trait]
pub trait Representation<Ctx>: Sized
where
    Ctx: Context + Sync,
{
    /// The stringified name of the IPLD type.
    const NAME: &'static str;
    // /// The stringified IPLD typedef.
    // const SCHEMA: &'static str;
    ///
    const KIND: SchemaKind;
    // ///
    // const HAS_LINKS: bool = false;

    ///
    /// for unions, this might just delegate to the variant's type.name
    fn name(&self) -> &str {
        Self::NAME
    }

    // ///
    // /// focus a selector into Ipld
    // ///
    // /// TODO? T? Self? additional method into_type?
    // /// TODO? support validating against some user-defined constraints on type value
    // /// TODO: when focusing w/in links, must replace Link::Cid w/ Link::Dag
    // /// TODO? &mut self? interior mutability?
    // async fn focus<'a, T>(
    //     &self,
    //     selection: &Selector,
    //     executor: &'a Executor<'a, Ctx>,
    // ) -> Result<BorrowedIpld<'a>, ()>
    // where
    //     Selector: DeserializeSeed<'a, Value = T>,
    //     T: Representation<Ctx> + Into<BorrowedIpld<'a>>;

    //
    //
    //
    //
    //
    //

    // TODO:
    //  - take in &self or just return type?
    //  - return IPLD ??
    //  - how do we map a selector to an "RPC" call (such that selectors (+ something)) can be the basis of all queries and mutations?

    // /// Reads a selector into a `BorrowedIpld<'a>`...
    // /// TODO?
    // async fn read_ipld<'a, T>(
    //     ipld: &BorrowedIpld<'a>,
    //     selector: &Selector,
    //     executor: &'a Executor<'a, Ctx>,
    // ) -> Result<BorrowedIpld<'a>, ()>
    //     // CtxT: Context + FromContext<Ctx> + Sync, // TODO? is this right?
    //     T: Representation<Ctx> + TryFrom<BorrowedIpld<'a>>,
    //     <T as TryFrom<BorrowedIpld<'a>>>::Error: Debug;

    // /// Writes a `BorrowedIpld<'a>`...
    // /// TODO?
    // async fn write_ipld<'a>(
    //     ipld: &BorrowedIpld<'a>,
    //     executor: &'a Executor<'a, Ctx>,
    // ) -> Result<Selector, ()>;

    // /// Reads a selector into...
    // /// TODO?
    // async fn read<'a, T>(
    //     &'a self,
    //     selector: &Selector,
    //     executor: &'a Executor<'a, Ctx>,
    // ) -> Result<T, ()>
    // where
    //     &'a Self: Into<BorrowedIpld<'a>>,
    //     // CtxT: Context + FromContext<Ctx> + Sync, // TODO? is this right?
    //     T: Representation<Ctx> + TryFrom<BorrowedIpld<'a>>,
    //     <T as TryFrom<BorrowedIpld<'a>>>::Error: Debug,
    // {
    //     let self_as_ipld = self.into();
    //     let ipld = Self::read_ipld::<T>(&self_as_ipld, selector, executor).await?;
    //     Ok(T::try_from(ipld).unwrap())
    // }

    // /// Writes... the type's changes? everything?
    // /// TODO?
    // async fn write<'a>(&'a self, executor: &'a Executor<'a, Ctx>) -> Result<Selector, ()>
    // where
    //     &'a Self: Into<BorrowedIpld<'a>>,
    // {
    //     let self_as_ipld = self.into();
    //     let selector = Self::write_ipld(&self_as_ipld, executor).await?;
    //     Ok(selector)
    // }

    //
    //
    //
    //
    //
    //

    // /// Resolves a selector against the type, producing a value.
    // /// TODO?
    // async fn resolve_mut<'a, I, O>(
    //     &'a mut self,
    //     selector: &Selector,
    //     executor: &'a Executor<'a, Ctx>,
    //     input: I,
    // ) -> Result<O, ()>
    // where
    //     I: Representation<Ctx> + Send,
    //     O: Representation<Ctx>,
    // {
    //     unimplemented!()
    // }

    // TODO? what should this do for primtiives?
    // ///
    // async fn read<'a>(executor: &'a Executor<'a, Ctx>) -> Result<Self, ()>;
    // TODO? what should this do for primtiives?
    // ///
    // async fn write<'a>(&self, executor: &'a Executor<'a, Ctx>) -> Result<Cid, ()>;

    // ///
    // /// if you have links (any recursive that has links directly, or contains types that have links)
    // ///     recurse
    // ///     ...
    // ///     ... ???
    // /// if you are a linked dag (a link with a CID and type)
    // ///     get a block service from the Context
    // ///     BlockService.put
    // ///     replace link CID + dag with new CID
    // ///     return Ok
    // /// else (any primitive, or a link with only a CID)
    // ///     serialize yourself with provided Serializer
    // ///     return
    // ///
    // async fn write<S, C>(&self, serializer: S, context: C) -> Result<(), ()>
    // where
    //     S: Serializer,
    //     C: Context;

    // // TODO: should return Result<Self>
    // async fn read<'de, D, C>(deserializer: D, context: C) -> Result<(), ()>
    // where
    //     D: Deserializer<'de>,
    //     C: Context;
}

///
/// Some types have in-memory representations distinct from their IPLD representations:
///     - Links can map to types, so they can represent both CIDs and the underling types
///     - Signed/encrypted payloads can be further resolved into native types after verifying the signature/performing decryption
#[async_trait]
pub trait RepresentationExt<Ctx, T>: Representation<Ctx>
where
    Ctx: Context + Sync,
    T: Representation<Ctx>,
{
    // async fn resolve<'a>(self, executor: &'a Executor<'a, Ctx>) -> Result<T, ()> {
    //     unimplemented!()
    // }
}

//
//
//
//
//
//

// pub trait Dag<Ctx> {
//     type Repr: Representation<Ctx>;
// }

// impl<Ctx, T> Serialize for Dag<Ctx, Repr = T> where T: Representation<Ctx> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         T::serialize()
//     }
// }

// impl<'de, Ctx, T> Deserialize<'de> for Dag<Ctx, Repr = T> where T: Representation<Ctx> {}

// pub trait RepresentationExt<'a, Ctx>: Representation<Ctx> {
//     fn serialize_ipld<S>(ipld: &BorrowedIpld<'a>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer;

//     fn deserialize_ipld<D>(deserializer: D) -> Result<BorrowedIpld<'a>, D::Error>
//     where
//         D: Deserializer<'a>,
// }

//
//
//
//
//
//

// impl Representation for () {
//     async fn write<S, C>(&self, serializer: S, context: C) -> Result<(), ()>
//     where
//         S: Serializer,
//         C: Context,
//     {
//         Ok(())
//     }

//     async fn read<'de, D, C>(deserializer: D, context: C) -> Result<(), ()>
//     where
//         D: Deserializer<'de>,
//         C: Context,
//     {
//         Ok(())
//     }
// }

// impl<R: Representation> Representation for (Cid, Option<R>) {
//     async fn write<S, C>(&self, serializer: S, context: C) -> Result<(), ()>
//     where
//         S: Serializer,
//         C: Context,
//     {
//         Ok(())
//     }

//     async fn read<'de, D, C>(deserializer: D, context: C) -> Result<(), ()>
//     where
//         D: Deserializer<'de>,
//         C: Context,
//     {
//         Ok(())
//     }
// }

//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
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
