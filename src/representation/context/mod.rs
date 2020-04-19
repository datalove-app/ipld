//! Execution contexts for `Representation`s to `Read`/`Write` themselves from/to bytes and query/mutate themselves by specializing their implementation around specific `State` changes.
//!
//! While a `Representation` defines how a type traverses it's fields and maps them to bytes or blocks, the `Context` determines what happens with the bytes when encountering nested types, links, etc, before writing to or after reading from the byte stream.
//!
//! For example:
//!     - An `impl Context for EncryptedContext` can provide a byte stream that encrypts bytes written from a type/decrypts bytes read into a type. Later, a `Representation` can be provided with an `EncyptedContext` initialized with a key, transparently encrypting/decrypting the provided byte streams.
//!     - Additionally, we can define an `impl State for Encrypted<R, W>: Context<R, W>` and a type whose `Representation` implementation could derive an encryption/decryption key from within the type, ensuring that the type can only be stored in ciphertext.

use super::Representation;
use async_trait::async_trait;

///
#[async_trait]
pub trait Context {
    // /// Internally, this will:
    // ///     - get a (concrete?) BlockWriter from a BlockService
    // ///     - determine the Codec + Format from the BlockMeta
    // ///         - create a
    // ///
    // /// ## Example:
    // /// ```
    // /// Context::write(&ipld).await?;
    // /// ```
    // async fn write<R, B>(&self, ipld: &R, block_meta: B) -> Result<(), ()>
    // where
    //     R: Representation,
    //     B: Into<BlockMeta>;
}

// impl<'a, Ctx: Context> Context for &'a Ctx {}

pub trait ContextExt {}

static NULL_CONTEXT: () = ();

pub struct DefaultContext;
impl Context for DefaultContext {}

// impl Context for () {}

pub trait FromContext<Ctx> {
    fn from(ctx: &Ctx) -> &Self;
}

impl<Ctx> FromContext<Ctx> for () {
    fn from(_ctx: &Ctx) -> &Self {
        &NULL_CONTEXT
    }
}

impl<Ctx> FromContext<Ctx> for Ctx
where
    Ctx: Context,
{
    fn from(ctx: &Ctx) -> &Self {
        ctx
    }
}

// /// An execution context for `Representation`s to `Read`/`Write` themselves from/to bytes by signalling `State` changes to the `Context`.
// #[async_trait]
// pub trait Context: Sized {
//     type Error: Into<Error>;

//     //    /// Provides ...
//     //    fn codec(&self) -> Codec;

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

//     //    /// Ask the `Context` how much of the type to `Resolve`.
//     //    async fn resolve_range(&self) -> ResolveRange;

//     //    ///
//     //    async fn resolve_block(&self, cid: &Cid) -> Result<(), Error>;

//     //    ///
//     //    async fn flush_block(&self) -> Result<Cid, Self::Error>;

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

//     ///
//     async fn write<W>(t: &T, writer: &mut W) -> Result<(), <Self as Codec>::Error>
//     where
//         W: Write + Seek + Unpin + Send,
//         T: Sync;

//     ///
//     async fn read_offsets<R>(reader: &mut R) -> Result<(u8, usize, u8), <Self as Codec>::Error>
//     where
//         R: Read + Seek + Unpin + Send,
//         T: 'async_trait;

//     ///
//     async fn write_offsets<W>(
//         t: &T,
//         writer: &mut W,
//     ) -> Result<(u8, usize, u8), <Self as Codec>::Error>
//     where
//         W: Write + Seek + Unpin + Send,
//         T: Sync;
// }
