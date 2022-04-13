//! Execution contexts for `Representation`s to `Read`/`Write` themselves from/to bytes and query/mutate themselves by specializing their implementation around specific `State` changes.
//!
//! While a `Representation` defines how a type traverses it's fields and maps them to bytes or blocks, the `Context` determines what happens with the bytes when encountering nested types, links, etc, before writing to or after reading from the byte stream.
//!
//! For example:
//!     - An `impl Context for EncryptedContext` can provide a byte stream that encrypts bytes written by a type/decrypts bytes read into a type. Later, a `Representation` can be provided with an `EncyptedContext` initialized with a key, transparently encrypting/decrypting the provided byte streams.
//!     - Additionally, we can define an `impl State for Encrypted<R, W>: Context<R, W>` and a type whose `Representation` implementation could derive an encryption/decryption key from within the type, ensuring that the type can only be stored in ciphertext.

#[cfg(feature = "ipfs")]
mod ipfs;

use crate::dev::*;
use macros::derive_more::{AsMut, AsRef};
use std::{
    collections::BTreeMap,
    iter::FromFn,
    marker::PhantomData,
    path::{Path, PathBuf},
};

///
// #[async_trait::async_trait]
pub trait Context: Sized {
    // fn new_seed<'a, T, U>(
    //     &'a mut self,
    //     selector: &'a Selector,
    // ) -> ContextSeed<'a, &'a mut Self, T, U>
    // where
    //     T: Representation,
    //     U: Representation;

    // fn seed_mut(&mut self) -> &mut SelectorSeed;

    // fn block_decoder<'de, Si>(
    //     &mut self,
    //     meta: BlockMeta<'_, Si>,
    // ) -> Result<Box<dyn ErasedDeserializer<'de>>, Error>
    // where
    //     Si: MultihashSize,
    // {
    //     unimplemented!()
    // }

    fn decoder<'de>(&mut self) -> Box<dyn ErasedDeserializer<'de>> {
        unimplemented!()
    }

    fn set_decoder<'de, D: Decoder<'de>>(&mut self, de: &mut D) {
        unimplemented!()
    }

    // fn block_encoder<Si>(
    //     &mut self,
    //     meta: BlockMeta<'_, Si>,
    // ) -> Result<&'_ mut dyn ErasedSerializer, Error>
    // where
    //     Si: MultihashSize,
    // {
    //     unimplemented!()
    // }

    // fn path_encoder<P: AsRef<Path>>(
    //     &mut self,
    //     meta: P,
    // ) -> Result<&'_ mut dyn ErasedSerializer, Error> {
    //     unimplemented!()
    // }

    // fn close_encoder<Si, So>(
    //     &mut self,
    //     replacing: Option<BlockMeta<'_, Si, So>>,
    // ) -> Result<CidGeneric<So>, Error>
    // where
    //     Si: MultihashSize,
    //     So: MultihashSize,
    // {
    //     unimplemented!()
    // }

    //
    //
    //

    // /// Internally, this will:
    // ///     - get a (concrete?) BlockWriter from a BlockService
    // ///     - determine the Codec + Format from the BlockMeta
    // ///         - create a
    // ///
    // /// ## Example:
    // /// ```
    // /// Context::write(&ipld).await?;
    // /// ```
    // async fn write<B, R>(&mut self, dag: &R, block_meta: B) -> Result<(), ()>
    // where
    //     R: Representation<Self>,
    //     B: Into<BlockMeta>;

    // async fn resolve(&mut self)
}

// impl<'a, C: Context> AsMut<&'a mut C> for &'a mut C {
//     fn as_mut(&mut self) -> &'a mut C {
//         &mut self
//     }
// }

impl<'a, C: Context + 'a> Context for &'a mut C {}

pub struct MemoryContext {
    root: PathBuf,
    blocks: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryContext {}

// impl AsMut<Self> for MemoryContext {
//     fn as_mut(&mut self) -> &mut Self {
//         &mut (*self)
//     }
// }

// #[async_trait::async_trait]
impl Context for MemoryContext {
    // fn new_seed<'a, T, U>(&'a mut self, selector: &'a Selector) -> ContextSeed<'a, Self, T, U>
    // where
    //     T: Representation,
    //     U: Representation,
    // {
    //     ContextSeed::<'a, Self, T, U>::from(
    //         selector,
    //         SelectorState::new(&mut self.root, None),
    //         self,
    //     )
    // }

    // type Reader = Cursor<Vec<u8>>;
    // type Writer = Vec<u8>;
    //
    // async fn block_reader<Si>(&mut self, cid: &CidGeneric<Si>) -> Result<Self::Reader, Error>
    // where
    //     Si: MultihashSize,
    // {
    //     let cid_bytes = cid.to_bytes();
    //     let block = self
    //         .blocks
    //         .get(&cid_bytes)
    //         .ok_or(Error::Context(anyhow::anyhow!(
    //             "block with cid {} not found",
    //             &cid
    //         )))?;
    //
    //     Ok(Cursor::new(block.clone()))
    // }
    //
    // async fn block_writer(&mut self) -> Result<Self::Writer, Error> {
    //     Ok(Vec::new())
    // }
    //
    // async fn finish_block_writer<'a, Si, So>(
    //     &mut self,
    //     mut writer: Self::Writer,
    //     replacing: Option<BlockMeta<'a, Si, So>>,
    // ) -> Result<CidGeneric<So>, Error>
    // where
    //     Si: MultihashSize,
    //     So: MultihashSize,
    // {
    //     writer.flush();
    //     // let new_cid: CidGeneric<So> = CidGeneric::<Si>::random();
    //     // self.blocks.insert()
    //
    //     if let Some(BlockMeta::Cid(cid)) = replacing {
    //         self.blocks.remove(&cid.to_bytes());
    //     }
    //
    //     // Ok(new_cid)
    //     unimplemented!()
    // }
}

/// A helper type for guided decoding of a dag, using a selector to direct
/// and/or ignore fields or entire blocks, and a linked context to fetch more
/// blocks.
#[derive(Debug)]
pub struct ContextSeed<'a, C, T, U = T>
where
    C: Context,
    T: Representation,
    U: Representation,
{
    pub selector: &'a Selector,
    pub state: &'a mut SelectorState,

    // pub state &'a mut SelectorMode2<'b, C, T, U>,
    pub ctx: &'a mut C,
    // pub mode: SelectionMode2<'a, T, U>,
    // the type the selector is applied to, and the type to be selected
    _t: PhantomData<(T, U)>,
}

// impl<'a, C: Context + Clone, T: Representation> Clone for ContextSeed<'a, C, T> {
//     fn clone(&self) -> Self {
//         Self {
//             selector: self.selector,
//             state: self.state.clone(),
//             ctx: self.ctx,
//             _t: PhantomData,
//         }
//     }
// }

// impl<C: Context, T: Representation> AsRef<Selector> for ContextSeed<C, T> {
//     fn as_ref(&self) -> &Selector {
//         self.state.as_selector()
//     }
// }

// impl<'de, C: Context, T: Representation> From<ContextSeed<C, T>> for (Selector, SelectorState, C) {
//     #[inline]
//     fn from(seed: ContextSeed<C, T>) -> (Selector, SelectorState, C) {
//         (seed.selector, seed.state, seed.ctx)
//     }
// }

// impl<'a, C: Context, T: Representation> Context for ContextSeed<'a, C, T> {}

/*
impl<'de, C: Context, T: Representation, V: Visitor<'de>> Visitor<'de>
    for ContextSeed<C, T, V>
{
    type Value = V::Value;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.visitor.expecting(formatter)
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_bool(v)
    }

    #[inline]
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i8(v)
    }

    #[inline]
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i16(v)
    }

    #[inline]
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i32(v)
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i64(v)
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i128(v)
    }

    #[inline]
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u8(v)
    }

    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u16(v)
    }

    #[inline]
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u32(v)
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u64(v)
    }

    #[inline]
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u128(v)
    }

    #[inline]
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_f32(v)
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_f64(v)
    }

    #[inline]
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_char(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_str(v)
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_borrowed_str(v)
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_string(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_bytes(v)
    }

    #[inline]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_byte_buf(v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visitor.visit_some(deserializer)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_unit()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visitor.visit_newtype_struct(deserializer)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.visitor.visit_seq(seq)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.visitor.visit_map(map)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        self.visitor.visit_enum(data)
    }
}
*/

// TODO: impl this for every Selectable T, defining:
// TODO:    1) how to create Self from T's own visitor
// TODO:    2) defining which deserializer method to call
// default impl<'a: 'de, 'de, C, T> DeserializeSeed<'de> for ContextSeed<'a, C, T, T>
// where
//     C: Context,
//     T: Representation,
// {
//     type Value = Option<T>;
//
//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Err(D::Error::custom("unimplemented"))
//     }
// }
//
// TODO: impl this for every Selectable T, defining:
// TODO:    1) what to do when visited based on the current selector
// TODO:    2) how to create Self as a seed for the next type to deserialize
// impl<'de, C, T> IpldVisitorExt<'de> for ContextSeed<C, T>
// where
//     C: Context,
//     T: Representation,
//     // V: Visitor<'de>,
// {
//     type Value = T;
//
//     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         unimplemented!()
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

// pub trait FromContext<Ctx> {
//     fn from(ctx: &Ctx) -> &Self;
// }
//
// impl<Ctx> FromContext<Ctx> for () {
//     fn from(_ctx: &Ctx) -> &Self {
//         &NULL_CONTEXT
//     }
// }
//
// impl<Ctx> FromContext<Ctx> for Ctx
// where
//     Ctx: Context<S>,
//     S: ISelector,
// {
//     fn from(ctx: &Ctx) -> &Self {
//         ctx
//     }
// }
//
// /// An execution context for `Representation`s to `Read`/`Write` themselves from/to bytes by signalling `State` changes to the `Context`.
// #[async_trait]
// pub trait Context: Sized {
//     type Error: Into<Error>;
//
//     //    /// Provides ...
//     //    fn codec(&self) -> Codec;
//
//     //    /// `Read`s the `Representation` using the provided `Context`.
//     //    async fn decode<T>(&self) -> Result<T, Self::Error>
//     //    where
//     //        T: Representation<Self>,
//     //    {
//     //        T::decode(self).await
//     //    }
//     //
//     //    ///
//     //    async fn encode<T>(&self, value: T) -> Result<Option<Cid>, Self::Error>
//     //    where
//     //        T: Representation<Self>
//     //    {
//     //        value.encode(self).await?;
//     //        Ok(None)
//     //    }
//
//     //    ///
//     //    async fn read_with_ctx<NewCtx, NewCo, C, T>(&self) -> Result<T, Self::Error>
//     //    where
//     //        C: Command,
//     //        NewCtx: Handler<Co, Command = C>,
//     //        NewCo: Codec,
//     //        T: Representation<NewCtx, NewCo, R, W>,
//     //        Self: IntoHandler<NewCo, R, W, C, NewCtx>,
//     //    {
//     //        self.into_handler().read().await
//     //    }
//     //
//     //    ///
//     //    async fn write_with_ctx<NewCtx, T>(&self, value: T) -> Result<Option<Cid>,
//     // Self::Error>;
//
//     //    /// Ask the `Context` how much of the type to `Resolve`.
//     //    async fn resolve_range(&self) -> ResolveRange;
//
//     //    ///
//     //    async fn resolve_block(&self, cid: &Cid) -> Result<(), Error>;
//
//     //    ///
//     //    async fn flush_block(&self) -> Result<Cid, Self::Error>;
//
//     //    /// Attempts to apply the current `Command`, triggering optional
//     //    /// side-effects within `Context`, allowing it to drive the
//     //    /// `Representation` operation.
//     //    ///
//     //    /// This is done by implementing `Handler<C>` for your `Context`(s) for each
//     //    /// `Command` your IPLD types require.
//     //    async fn apply<C, H>(&self, command: C) -> C::Result
//     //    where
//     //        Co: 'async_trait,
//     //        R: 'async_trait,
//     //        W: 'async_trait,
//     //        C: Command + Send,
//     //        H: Handler<Co, R, W, Command = C> + Send + Sync,
//     //        Self: IntoHandler<Co, R, W, C, H>,
//     //    {
//     //        self.into_handler().handle(command).await
//     //    }
// }

///// Handles a `Context` `Command`.
//#[async_trait]
//pub trait Handler<Co, R, W>: Context<Co, R, W>
//where
//    Co: Codec,
//    R: Read,
//    W: Write,
//{
//    type Command: Command;
//
//    ///
//    async fn handle(&self, command: Self::Command) -> <Self::Command as Command>::Result;
//}
//
///// Converts a `Context` into a `Handler` that can apply a `Command`.
//pub trait IntoHandler<Co, R, W, C, H>: Context<Co, R, W>
//where
//    Co: Codec,
//    R: Read,
//    W: Write,
//    C: Command,
//    H: Handler<Co, R, W, Command = C>,
//{
//    fn into_handler(&self) -> &H;
//}
//
///// Blanket conversion for a given `Context` that can already `Handler` a
///// given `Command`.
//impl<Co, R, W, C, H> IntoHandler<Co, R, W, C, H> for H
//where
//    Co: Codec,
//    R: Read,
//    W: Write,
//    C: Command,
//    H: Handler<Co, R, W, Command = C>,
//{
//    fn into_handler(&self) -> &H {
//        self
//    }
//}

// impl<Ctx, R, W, T> CodecExt<T> for Ctx
// where
//     Ctx: Codec + Context<<Self as CodecExt<T>>, R, W>,
//     R: Read,
//     W: Write,
//     T: Representation<Self, <Self as CodecExt<T>>, R, W>,
// {
//         async fn read<R>(reader: &mut R) -> Result<T, <Self as Codec>::Error>
//     where
//         R: Read + Seek + Unpin + Send,
//         T: 'async_trait;
//
//     ///
//     async fn write<W>(t: &T, writer: &mut W) -> Result<(), <Self as Codec>::Error>
//     where
//         W: Write + Seek + Unpin + Send,
//         T: Sync;
//
//     ///
//     async fn read_offsets<R>(reader: &mut R) -> Result<(u8, usize, u8), <Self as Codec>::Error>
//     where
//         R: Read + Seek + Unpin + Send,
//         T: 'async_trait;
//
//     ///
//     async fn write_offsets<W>(
//         t: &T,
//         writer: &mut W,
//     ) -> Result<(u8, usize, u8), <Self as Codec>::Error>
//     where
//         W: Write + Seek + Unpin + Send,
//         T: Sync;
// }

impl<'a, C, T, U> From<(&'a Selector, &'a mut SelectorState, &'a mut C)>
    for ContextSeed<'a, C, T, U>
where
    C: Context,
    T: Representation,
    U: Representation,
{
    #[inline]
    fn from((selector, state, ctx): (&'a Selector, &'a mut SelectorState, &'a mut C)) -> Self {
        Self::from(selector, state, ctx)
    }
}

impl<'a, C, T, U> ContextSeed<'a, C, T, U>
where
    C: Context,
    T: Representation,
    U: Representation,
{
    ///
    #[inline]
    pub fn from(selector: &'a Selector, state: &'a mut SelectorState, ctx: &'a mut C) -> Self {
        Self {
            selector,
            state,
            ctx,
            // visitor,
            _t: PhantomData,
        }
    }

    #[inline]
    pub fn into<V, W>(self) -> ContextSeed<'a, C, V, W>
    where
        // 'a: 'b,
        V: Representation,
        W: Representation,
    {
        ContextSeed::<'a, C, V, W> {
            selector: self.selector,
            state: self.state,
            ctx: self.ctx,
            _t: PhantomData,
        }
    }

    pub fn decode<'de>(self) -> Result<Option<U>, Error>
    where
        Self: DeserializeSeed<'de, Value = Option<U>>,
    {
        let decoder = self.ctx.decoder();
        self.deserialize(decoder).map_err(Error::decoder)
    }

    #[inline]
    pub fn mode(&self) -> SelectionMode {
        self.state.mode()
    }

    #[inline]
    pub fn descend_index(mut self, index: usize) -> Result<Self, Error> {
        self.state.descend_index(index, T::IS_LINK)?;
        self.selector = self.selector.next(None)?;
        Ok(self)
    }

    #[inline]
    pub fn descend_field<P: AsRef<Path>>(mut self, field: P) -> Result<Self, Error> {
        self.state.descend_field(field.as_ref(), T::IS_LINK)?;
        self.selector = self.selector.next(Some(field.as_ref()))?;
        Ok(self)
    }

    #[inline]
    pub fn send_selection(&self, node: Node) -> Result<(), Error> {
        self.state.send_selection(node)
    }

    #[inline]
    pub fn send_matched(&self, node: Node, label: Option<String>) -> Result<(), Error> {
        self.state.send_matched(node, label)
    }

    #[inline]
    pub fn send_dag<V: Into<U>>(&self, dag: V, label: Option<String>) -> Result<(), Error>
    where
        U: Send + Sync + 'static,
    {
        Ok(self.state.send_dag::<U>(dag.into(), label)?)
    }

    // ///
    // #[inline]
    // pub fn descend<P: AsRef<Path>, U: Representation>(
    //     mut self,
    //     next_selector: Selector,
    //     next_path_segment: P,
    //     is_link: bool,
    // ) -> Result<ContextSeed<C, U>, Error> {
    //     self.state
    //         .descend(next_selector, next_path_segment, is_link)?;
    //     Ok(ContextSeed::<C, U> {
    //         state: self.state,
    //         ctx: self.ctx,
    //         _t: PhantomData,
    //     })
    // }

    // ///
    // #[inline]
    // pub fn ascend<U: Representation>(
    //     mut self_: ContextSeed<C, U>,
    //     previous_selector: Selector,
    //     is_link: bool,
    // ) -> Result<Self, Error> {
    //     self_.state.ascend(previous_selector, is_link)?;
    //     Ok(Self {
    //         state: self_.state,
    //         ctx: self_.ctx,
    //         _t: PhantomData,
    //     })
    // }
}

/// Returns an iterator ...
pub fn seq_iter<'a: 'de, 'de, C, A, T, U>(
    selector: &'a Selector,
    mut state: &'a mut SelectorState,
    mut ctx: &'a mut C,
    seq: A,
) -> Box<dyn Iterator<Item = Result<Option<U>, A::Error>> + 'a>
// ) -> FromFn<&'a mut dyn FnMut() -> Option<Result<Option<U>, Error>>>
where
    C: Context,
    A: SeqAccess<'de> + 'a,
    T: Representation + Select<C>,
    U: Representation,
    Option<U>: Select<C>,
{
    // return an iterator
    // for each item:
    //  - descend index, get next selector
    //  - create a deserializer, set it to state
    //      -
    //  - call T::select(inputs)

    let mut decoder = SeqIterDecoder::<_, C, T, U> {
        seq,
        end: false,
        _t: PhantomData,
    };
    Box::new(std::iter::from_fn(move || {
        if decoder.end {
            None
        } else {
            ctx.set_decoder(&mut decoder);
            // todo: using option isnt right
            match T::select::<Option<U>>(&selector, &mut state, &mut ctx).transpose() {
                Some(Err(err)) => Some(Err(A::Error::custom(err))),
                Some(Ok(inner)) => Some(Ok(inner)),
                None => None,
            }
        }
    }))

    // T::select::<U>(selector, state, ctx)
    // unimplemented!()
}

pub struct SeqIterDecoder<A, C, T, U> {
    pub seq: A,
    pub end: bool,
    _t: PhantomData<(C, T, U)>,
}

impl<A, C, T, U> SeqIterDecoder<A, C, T, U> {
    // fn new<'de>(seq: A) -> Box<dyn ErasedDeserializer<'de>>
    // where
    //     A: SeqAccess<'de> + 'static,
    // {
    //     Box::new(<dyn ErasedDeserializer<'de>>::erase(Self {
    //         seq,
    //         end: false,
    //     }))
    // }

    // fn decode<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    // where
    //     V: Visitor<'de>,
    // {

    // }
}

macro_rules! deserialize {
    ($fn:ident) => {
        // todo issue with this is that we cant add constraints to the V to use it as a seed
        fn $fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            // if type_eq::<ContextSeed<'_, C, T, U>, V>() {
            //     if let Some(next) = self.seq.next_element(self)
            // }
        }
    };
}

impl<'de, A: SeqAccess<'de>, C, T, U> Deserializer<'de> for SeqIterDecoder<A, C, T, U> {
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    serde::serde_if_integer128! {
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            let _ = visitor;
            Err(Self::Error::custom("i128 is not supported"))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    serde::serde_if_integer128! {
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            let _ = visitor;
            Err(Self::Error::custom("u128 is not supported"))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

// default impl<'de, 'a, C, T, U> DeserializeSeed<'de> for ContextSeed<'a, C, T, U>
// where
//     C: Context,
//     T: Representation,
//     U: Representation,
// {
//     default type Value = Option<U>;

//     #[inline]
//     default fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         D::Error::custom("must be implemented by an `ipld` crate type")
//     }
// }
