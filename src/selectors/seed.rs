use super::*;
use crate::dev::{macros::derive_more::From, *};
use std::{fmt, marker::PhantomData};

/// A helper type for guided decoding of a dag, using a selector to direct
/// and/or ignore fields or entire blocks, and a linked context to fetch more
/// blocks.
/// TODO: rename to SelectorSeed
pub struct SelectorSeed<'a, Ctx, T = Any> {
    pub(crate) selector: &'a Selector,
    pub(crate) state: &'a mut State,
    pub(crate) callback: Callback<'a, Ctx, T>,
    pub(crate) ctx: &'a mut Ctx,
}

/// A marked [`SelectorSeed`] that's aware of the codec of the block it's
/// currenly selecting against.
///
/// TODO: a few issues to consider (going backwards):
/// ? 1. codec needs to receive a CodecDeserializeSeed (or otherwise CodedSeed, to associate the right const C)
/// ? 2. const C needs to carry down until we reach bytes/links
/// ? 3. Link needs to take some seed, uncode it, then use inner to call SelectorSeed::select for the next block
#[doc(hidden)]
#[derive(Debug)]
pub struct CodecSeed<const C: u64, const D: bool, S, T = Any>(pub(crate) S, PhantomData<T>);

impl<const C: u64, const D: bool, S, T> CodecSeed<C, D, S, T> {
    pub const D: bool = D;
    pub const fn is_select() -> bool {
        !D
    }
}

impl<'a, const C: u64, Ctx, T> CodecSeed<C, false, SelectorSeed<'a, Ctx, T>, T> {
    ///
    #[inline]
    pub(crate) fn from_parts(
        selector: &'a Selector,
        state: &'a mut State,
        callback: Callback<'a, Ctx, T>,
        ctx: &'a mut Ctx,
    ) -> Self {
        Self(
            SelectorSeed {
                selector,
                state,
                callback,
                ctx,
                // visitor,
                // _t: PhantomData,
            },
            PhantomData,
        )
    }

    ///
    #[inline]
    #[doc(hidden)]
    pub(crate) fn into_parts(
        self,
    ) -> (
        &'a Selector,
        &'a mut State,
        Callback<'a, Ctx, T>,
        &'a mut Ctx,
    ) {
        (self.0.selector, self.0.state, self.0.callback, self.0.ctx)
    }
}

impl<'de, const C: u64, T> CodecSeed<C, true, PhantomData<T>, T>
// where
//     Self: Visitor<'de>,
{
    #[inline]
    #[doc(hidden)]
    pub fn empty() -> Self {
        CodecSeed(PhantomData, PhantomData)
    }
}

// impl<'a, const C: u64, Ctx, T> CodecSeed<C, false, SelectorSeed<'a, Ctx, T>> {
//     fn from(inner: SelectorSeed<'a, Ctx, T>) -> Self {
//         Self(inner)
//     }
// }
// impl<const C: u64, T> CodecSeed<C, true, PhantomData<T>> {
//     fn from(inner: PhantomData<T>) -> Self {
//         Self(inner)
//     }
// }
impl<'a, const C: u64, Ctx, T> From<SelectorSeed<'a, Ctx, T>>
    for CodecSeed<C, false, SelectorSeed<'a, Ctx, T>, T>
{
    fn from(inner: SelectorSeed<'a, Ctx, T>) -> Self {
        Self(inner, PhantomData)
    }
}
impl<const C: u64, T> From<PhantomData<T>> for CodecSeed<C, true, PhantomData<T>, T> {
    fn from(inner: PhantomData<T>) -> Self {
        Self(inner, PhantomData)
    }
}
// impl<'a, const C: u64, Ctx, T> Into<CodecSeed<C, false, Self>> for SelectorSeed<'a, Ctx, T> {
//     fn into(self) -> CodecSeed<C, false, Self> {
//         CodecSeed(self)
//     }
// }
// impl<const C: u64, T> Into<CodecSeed<C, true, Self>> for PhantomData<T> {
//     fn into(self) -> CodecSeed<C, true, Self> {
//         CodecSeed(self)
//     }
// }

// impl<const C: u64, const D: bool, T> CodecSeed<C, D, T> {
//     pub fn into_inner(self) -> T {
//         self.0
//     }
// }

///
#[doc(hidden)]
pub type CodedSelectorSeed<'a, const C: u64, const D: bool, Ctx, T> =
    CodecSeed<C, D, SelectorSeed<'a, Ctx, T>, T>;

// /// Replacement trait for [`serde::de::DeserializeSeed`], that allows us to
// /// switch deserialization behaviour based on the current block's [`Codec`].
// ///
// /// How to use:
// /// ? implement this for SelectorSeed
// ///     ? call the appropriate deserializer method for the codec
// ///     ? pass itself as a visitor
// /// ? for compounds, create CodecSeed<SelectorSeed>
// ///     ? pass that as a seed
// #[doc(hidden)]
// pub trait CodecDeserializeSeed<'de> {
//     type Value;
//     // pub trait CodecDeserializeSeed<'de>: DeserializeSeed<'de, Value = ()> {
//     fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         // CodecSeed<C, false, Self>: DeserializeSeed<'de, Value = ()>,
//         D: Deserializer<'de>;
//     // {
//     //     // CodecSeed::<C, _>(self).deserialize(deserializer)
//     //     DeserializeSeed::<'_>::deserialize(self, deserializer)
//     // }
//
//     fn mode(&self) -> SelectionMode {
//         unimplemented!()
//     }
//
//     // fn to_field_select_seed<'b, U>(
//     //     &mut self,
//     //     field: Field<'b>,
//     //     match_cb: Option,
//     //     post_cb: Option,
//     // ) -> Result<(), Error> {
//     //     unimplemented!()
//     // }
// }
//
// // impl<'de, T> CodecDeserializeSeed<'de> for PhantomData<T>
// // where
// //     T: Representation,
// // {
// //     type Value = T;
// //
// //     fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<Self::Value, D::Error>
// //     where
// //         D: Deserializer<'de>,
// //     {
// //         <T as Representation>::deserialize::<C, D>(deserializer)
// //     }
// // }
// //
// // impl<'a, 'de, Ctx, T> CodecDeserializeSeed<'de> for SelectorSeed<'a, Ctx, T>
// // // where
// // //     CodecSeed<C, false, Self>: DeserializeSeed<'de, Value = ()>,
// // {
// //     const D: bool = false;
// //     type Value = ();
// //     fn deserialize<const C: u64, De>(self, deserializer: De) -> Result<(), De::Error>
// //     where
// //         // CodecSeed<C, { <SelectorSeed<'a, Ctx, T> as CodecDeserializeSeed<'de>>::D }, Self>:
// //         //     DeserializeSeed<'de, Value = ()>,
// //         De: Deserializer<'de>,
// //     {
// //         CodecSeed::<C, false, _>(self).deserialize(deserializer)
// //     }
// // }

impl<'a, Ctx, T> fmt::Debug for SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: fmt::Debug + Representation,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectorSeed")
            .field("selector", &self.selector)
            .field("state", &self.state)
            .field("callback", &self.callback)
            .finish()
    }
}

// TODO: impl this for every Selectable T, defining:
// TODO:    1) how to create Self from T's own visitor
// TODO:    2) defining which deserializer method to call
// default impl<'a: 'de, 'de, Ctx, T> DeserializeSeed<'de> for SelectorSeed<'a, Ctx, T, T>
// where
//     Ctx: Context,
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
// impl<'de, Ctx, T> IpldVisitorExt<'de> for SelectorSeed<Ctx, T>
// where
//     Ctx: Context,
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
//     //    async fn read_with_ctx<NewCtx, NewCo, Ctx, T>(&self) -> Result<T, Self::Error>
//     //    where
//     //        Ctx: Command,
//     //        NewCtx: Handler<Co, Command = Ctx>,
//     //        NewCo: Codec,
//     //        T: Representation<NewCtx, NewCo, R, W>,
//     //        Self: IntoHandler<NewCo, R, W, Ctx, NewCtx>,
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
//     //    /// This is done by implementing `Handler<Ctx>` for your `Context`(s) for each
//     //    /// `Command` your IPLD types require.
//     //    async fn apply<Ctx, H>(&self, command: Ctx) -> Ctx::Result
//     //    where
//     //        Co: 'async_trait,
//     //        R: 'async_trait,
//     //        W: 'async_trait,
//     //        Ctx: Command + Send,
//     //        H: Handler<Co, R, W, Command = Ctx> + Send + Sync,
//     //        Self: IntoHandler<Co, R, W, Ctx, H>,
//     //    {
//     //        self.into_handler().handle(command).await
//     //    }
// }
//
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
//pub trait IntoHandler<Co, R, W, Ctx, H>: Context<Co, R, W>
//where
//    Co: Codec,
//    R: Read,
//    W: Write,
//    Ctx: Command,
//    H: Handler<Co, R, W, Command = Ctx>,
//{
//    fn into_handler(&self) -> &H;
//}
//
///// Blanket conversion for a given `Context` that can already `Handler` a
///// given `Command`.
//impl<Co, R, W, Ctx, H> IntoHandler<Co, R, W, Ctx, H> for H
//where
//    Co: Codec,
//    R: Read,
//    W: Write,
//    Ctx: Command,
//    H: Handler<Co, R, W, Command = Ctx>,
//{
//    fn into_handler(&self) -> &H {
//        self
//    }
//}
//
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

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation,
{
    // #[inline]
    // pub(crate) fn mode(&self) -> SelectionMode<'_> {
    //     match (&self.callback, self.selector) {
    //         (Callback::SelectNode { .. }, Selector::Matcher(m)) => {
    //             SelectionMode::SelectNode(m.label.as_deref())
    //         }
    //         (Callback::SelectNode { .. }, _) => SelectionMode::SelectNode(None),
    //         (Callback::SelectDag { .. } | Callback::MatchDag { .. }, _) => SelectionMode::SelectDag,
    //         // Self::Patch { .. } => SelectionMode::Patch,
    //     }
    // }

    ///
    #[inline]
    pub const fn mode(&self) -> SelectionMode {
        match &self.callback {
            Callback::SelectNode { .. } => SelectionMode::SelectNode,
            Callback::SelectDag { .. } | Callback::MatchDag { .. } => SelectionMode::SelectDag,
            // Self::Patch { .. } => SelectionMode::Patch,
        }
    }

    ///
    #[inline]
    pub const fn is_node(&self) -> bool {
        match self.callback {
            Callback::SelectNode { .. } => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub const fn is_dag(&self) -> bool {
        match self.callback {
            Callback::SelectDag { .. } | Callback::MatchDag { .. } => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub(crate) fn from_parts(
        selector: &'a Selector,
        state: &'a mut State,
        callback: Callback<'a, Ctx, T>,
        ctx: &'a mut Ctx,
    ) -> Self {
        Self {
            selector,
            state,
            callback,
            ctx,
            // visitor,
            // _t: PhantomData,
        }
    }

    ///
    #[inline]
    #[doc(hidden)]
    pub(crate) fn into_parts(
        self,
    ) -> (
        &'a Selector,
        &'a mut State,
        Callback<'a, Ctx, T>,
        &'a mut Ctx,
    ) {
        (self.selector, self.state, self.callback, self.ctx)
    }
}

// dag selection methods
impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation,
{
    ///
    #[doc(hidden)]
    #[inline]
    pub fn select(params: Params<'_, Ctx, T>, mut ctx: &mut Ctx) -> Result<(), Error>
    where
        Ctx: 'a,
        T: Select<Ctx>,
    {
        let Params {
            cid,
            selector,
            max_path_depth,
            max_link_depth,
            callback,
        } = params;
        let mut state = State {
            max_path_depth,
            max_link_depth,
            ..Default::default()
        };

        let root_cid = cid.ok_or_else(|| {
            Error::InvalidSelectionParams("selection must start against some cid")
        })?;
        let block = ctx.block_reader(&root_cid)?;
        let default_selector = Selector::DEFAULT;
        let seed = SelectorSeed {
            selector: &selector.unwrap_or(&default_selector),
            state: &mut state,
            callback,
            ctx: &mut ctx,
        };

        root_cid.multicodec()?.read_from_seed(seed, block)?;
        Ok(())
    }

    ///
    pub(crate) fn select_node(&mut self, node: SelectedNode) -> Result<(), Error> {
        self.callback
            .select_node(NodeSelection::new(self.state.path(), node), self.ctx)
    }

    ///
    pub(crate) fn select_matched_node(
        &mut self,
        node: SelectedNode,
        label: Option<&str>,
    ) -> Result<(), Error> {
        self.callback.select_node(
            NodeSelection::new_match(self.state.path(), node, label),
            self.ctx,
        )
    }

    ///
    pub(crate) fn select_matched_dag(&mut self, dag: T, label: Option<&str>) -> Result<(), Error>
    where
        T: Representation + 'static,
    {
        self.callback
            .select_dag(DagSelection::new(self.state.path(), dag, label), self.ctx)
    }

    #[inline]
    pub(crate) fn field_select_seed<'b, U>(
        selector: &'b Selector,
        state: &'b mut State,
        callback: &mut Callback<'a, Ctx, T>,
        ctx: &'b mut Ctx,
        field: Field<'b>,
        match_cb: Option<Box<dyn MatchDagOp<U, Ctx> + 'b>>,
        // match_cb: Option<F>
    ) -> Result<SelectorSeed<'b, Ctx, U>, Error>
    where
        'a: 'b,
        U: Representation,
        // F: FnOnce(U, Ctx) -> Result<(), Error>,
    {
        let next = selector
            .next(Some(&field))
            .ok_or_else(|| Error::missing_next_selector(selector))?;
        let callback = match (match_cb, callback) {
            //
            (None, Callback::SelectNode { cb, only_matched }) => Callback::SelectNode {
                cb: cb.clone(),
                only_matched: *only_matched,
            },
            //
            (None, Callback::SelectDag { cb }) => Callback::SelectDag { cb: cb.clone() },
            // matching the field
            (Some(field_cb), _) => Callback::MatchDag { cb: field_cb },
            _ => unreachable!(),
        };

        state.descend::<U>(field)?;
        Ok(SelectorSeed::from_parts(next, state, callback, ctx))
    }

    // pub(crate) fn select_field<'b, 'de, U, D>(
    //     &'b mut self,
    //     field: Field<'b>,
    //     match_cb: Option<Box<dyn MatchDagOp<U, Ctx> + 'b>>,
    //     deserializer: D,
    // ) -> Result<(), D::Error>
    // where
    //     'a: 'b,
    //     U: Representation + Select<Ctx>,
    //     D: Deserializer<'de>,
    // {
    //     let next = self
    //         .selector
    //         .next(Some(&field))
    //         .ok_or_else(|| Error::missing_next_selector(self.selector))
    //         .map_err(D::Error::custom)?;
    //     let callback = match (match_cb, &self.callback) {
    //         //
    //         (None, Callback::SelectNode { cb, only_matched }) => Callback::SelectNode {
    //             cb: cb.clone(),
    //             only_matched: *only_matched,
    //         },
    //         //
    //         (None, Callback::SelectDag { cb }) => Callback::SelectDag { cb: cb.clone() },
    //         // matching the field
    //         (Some(field_cb), _) => Callback::MatchDag { cb: field_cb },
    //         _ => unreachable!(),
    //     };
    //     self.state.descend::<U>(field).map_err(D::Error::custom)?;
    //
    //     let seed = SelectorSeed {
    //         selector: next,
    //         state: &mut self.state,
    //         callback,
    //         ctx: &mut self.ctx,
    //     };
    //
    //     U::__select_from(seed, deserializer).map_err(D::Error::custom)?;
    //     self.state.ascend::<T>().map_err(D::Error::custom)?;
    //     Ok(())
    // }
}

// patch methods
impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation,
{
    /*

    ///
    #[inline]
    pub fn to_field_patch_seed<'b, V, P>(
        self,
        field: P,
        parent_dag: &'b mut V,
    ) -> Result<(SelectorSeed<'b, Ctx, V, U>, &'b Selector), Error>
    where
        'a: 'b,
        P: AsRef<Path>,
        V: Representation,
        U: Representation + 'b,
    {
        let (selector, state, callback, ctx) = self.into_parts();

        state.descend(field.as_ref(), T::IS_LINK)?;
        let next = selector
            .next(Some(field.as_ref()))
            .ok_or_else(|| Error::MissingNextSelector(""))?;
        let callback = {
            // callback.to_patch::<V>(parent_dag)
            unimplemented!()
        };

        let seed = SelectorSeed {
            selector,
            state,
            callback,
            ctx,
        };

        Ok((seed, next))
    }

    ///
    #[inline]
    pub fn patch(&mut self, dag: &mut U) -> Result<(), Error> {
        match &self.callback {
            Params::Patch { op, flush, .. } => {
                // op(dag, self.ctx)?;
                op(self.ctx)?;

                if *flush {
                    // self.encode(&dag)?;
                }
                Ok(())
            }
            _ => Err(Error::InvalidSelectionMode(
                "`Params` not in patch mode",
            )),
        }
    }

     */
}

/// Provides skeletons for conveniently implementing serde-compatibility for
/// IPLD types.
#[macro_export]
macro_rules! impl_selector_seed_serde {
    // visitor for SelectorSeed

        // // impl Visitor for SelectorSeed
        // (@selector_seed_visitor
        //     { $($generics:tt)* } { $($bounds:tt)* }
        //     $ty:ty
        //     { $($visit_fns:tt)* }
        // ) => {
        //     impl<'a, 'de, Ctx, $($generics)*> $crate::dev::Visitor<'de> for $crate::dev::SelectorSeed<'a, Ctx, $ty>
        //     where
        //         Ctx: $crate::dev::Context,
        //         $($bounds)*
        //     {
        //         type Value = ();
        //
        //         $($visit_fns)*
        //     }
        // };
        // // impl IpldVisitorExt for SelectorSeed
        // (@selector_seed_visitor_ext
        //     { $($generics:tt)* } { $($bounds:tt)* }
        //     $ty:ty
        //     { $($visit_fns:tt)* }
        // ) => {
        //     impl<'a, 'de, Ctx, $($generics)*> $crate::dev::IpldVisitorExt<'de> for $crate::dev::SelectorSeed<'a, Ctx, $ty>
        //     where
        //         Ctx: $crate::dev::Context,
        //         $($bounds)*
        //     {
        //         $($visit_fns)*
        //     }
        // };

    // visitor for CodedSeed

        // impl Visitor for CodedSeed
        (@codec_seed_visitor
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
            { $($visit_fns:tt)* }
        ) => {
            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                const _D: bool = false;
                impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                    Visitor<'de> for
                    CodecSeed<_C, _D, SelectorSeed<'_a, Ctx, $ty>, $ty>
                where
                    Ctx: Context,
                    $($bounds)*
                {
                    type Value = ();
                    $($visit_fns)*
                }
            };

            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                const _D: bool = true;
                // impl<'a, 'de, const _C: u64, $($generics)*>
                //     Visitor<'de> for
                //     CodecSeed<_C, _D, std::marker::PhantomData<$ty>>
                // where
                //     $($generics)*
                // {
                //     type Value = $ty;
                //     $($visit_fns)*
                // }
            };
        };
        // impl IpldVisitorExt for CodedSeed
        (@codec_seed_visitor_ext
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
            { $($visit_fns:tt)* }
        ) => {
            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                const _D: bool = false;
                impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                    IpldVisitorExt<'de> for
                    CodecSeed<_C, _D, SelectorSeed<'_a, Ctx, $ty>, $ty>
                where
                    Ctx: Context,
                    $($bounds)*
                {
                    $($visit_fns)*
                }
            };

            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                const _D: bool = true;
                // impl<'a, 'de, const _C: u64, $($generics)*>
                //     IpldVisitorExt<'de> for
                //     CodecSeed<_C, _D, std::marker::PhantomData<$ty>>
                // where
                //     $($generics)*
                // {
                //     $($visit_fns)*
                // }
            };
        };

    // CodecDeserializeSeed

        // impl CodecDeserializeSeed for SelectorSeed
        (@selector_seed_codec_deseed
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
            { $($deseed_fn:tt)* }
        ) => {
            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                const _D: bool = false;
                // impl<'_a, 'de, Ctx, $($generics)*>
                //     CodecDeserializeSeed<'de> for
                //     SelectorSeed<'_a, Ctx, $ty>
                // where
                //     Ctx: Context,
                //     $($bounds)*
                // {
                //     type Value = ();
                //     $($deseed_fn)*
                //     // CodecSeed::<C, _D, _>(self).deserialize(deserializer)
                // }

                impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                    DeserializeSeed<'de> for
                    CodecSeed<_C, _D, SelectorSeed<'_a, Ctx, $ty>, $ty>
                where
                    Ctx: Context,
                    // Self: Visitor<'de>,
                {
                    // type Value = <Self as Visitor<'de>>::Value;
                    type Value = ();
                    $($deseed_fn)*
                }
            };

            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                const _D: bool = true;
                // impl<'a, 'de, $($generics)*>
                //     $crate::dev::CodecDeserializeSeed<'de> for
                //     std::marker::PhantomData<$ty>
                // {
                //     type Value = $ty;
                //     $($deseed_fn)*
                // }

                // impl<'de, const _C: u64 $($generics)*>
                //     $crate::dev::DeserializeSeed<'de> for
                //     $crate::dev::CodecSeed<_C, _D, std::marker::PhantomData<$ty>>
                // where
                //     Self: $crate::dev::Visitor<'de>,
                // {
                //     type Value = <Self as $crate::dev::Visitor<'de>>::Value;
                //     $($deseed_fn)*
                // }
            };
        };
        // impl CodecDeserializeSeed for SelectorSeed, using deserialize_any
        (@selector_seed_codec_deseed @any
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
        ) => {
            $crate::dev::macros::impl_selector_seed_serde! {
                @selector_seed_codec_deseed { $($generics)* } { $($bounds)* } $ty
            {
                #[inline]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    macros::cfg_if! {
                        if #[cfg(feature = "dag-json")] {
                            if _C == DagJson::CODE {
                                DagJson::deserialize_any(deserializer, self)
                            } else {
                                deserializer.deserialize_any(self)
                            }
                        } else if #[cfg(featureu = "dag-cbor")] {
                            if _C == DagCbor::CODE {
                                DagCbor::deserialize_any(deserializer, self)
                            } else {
                                deserializer.deserialize_any(self)
                            }
                        } else {
                            deserializer.deserialize_any(self)
                        }
                    }
                }
            }}
        };

    // Select

        // impl Select for T, using the seed's select
        (@selector_seed_select
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
        ) => {
            const _: () = {
                use $crate::dev::*;

                impl<Ctx, $($generics)*> Select<Ctx> for $ty
                where
                    Ctx: Context,
                    $($bounds)*
                {
                    #[inline]
                    fn select(
                        params: Params<'_, Ctx, Self>,
                        ctx: &mut Ctx,
                    ) -> Result<(), Error> {
                        SelectorSeed::<'_, Ctx, Self>::select(params, ctx)
                    }

                    #[doc(hidden)]
                    fn __select_from_deserializer<'a, 'de, const C: u64, D>(
                        seed: SelectorSeed<'a, Ctx, Self>,
                        deserializer: D,
                    ) -> Result<(), D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        let seed = CodecSeed::<C, false, _, Self>::from(seed);
                        seed.deserialize(deserializer)
                    }

                    #[doc(hidden)]
                    fn __select_from_seq<'a, 'de, const C: u64, A>(
                        seed: SelectorSeed<'a, Ctx, Self>,
                        mut seq: A,
                    ) -> Result<Option<()>, A::Error>
                    where
                        A: SeqAccess<'de>,
                    {
                        let seed = CodecSeed::<C, false, _, Self>::from(seed);
                        seq.next_element_seed(seed)
                    }

                    #[doc(hidden)]
                    fn __select_from_map<'a, 'de, const C: u64, A>(
                        seed: SelectorSeed<'a, Ctx, Self>,
                        mut map: A,
                        is_key: bool,
                    ) -> Result<Option<()>, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let seed = CodecSeed::<C, false, _, Self>::from(seed);
                        if is_key {
                            map.next_key_seed(seed)
                        } else {
                            Ok(Some(map.next_value_seed(seed)?))
                        }
                    }
                }
            };
        };
        (@seed_from_params $params:ident $ctx:ident) => {{
            let Params {
                cid,
                selector,
                max_path_depth,
                max_link_depth,
                callback,
            } = $params;
            let mut state = State {
                max_path_depth,
                max_link_depth,
                ..Default::default()
            };

            let root = cid.ok_or_else(|| {
                Error::InvalidSelectionParams("selection must start against some cid")
            })?;
            let block = $ctx.block_reader(&root)?;

            let default_selector = Selector::DEFAULT;
            SelectorSeed {
                selector: &selector.unwrap_or(&default_selector),
                state: &mut state,
                callback,
                ctx: &mut $ctx,
            }
        }};

    // newtype impls

        (@selector_seed_codec_deseed_newtype
            { $($generics:tt)* } { $($bounds:tt)* }
            // TODO: this should probably by a ty
            $ty:ident as $inner_ty:ty
        ) => {
            $crate::dev::macros::impl_selector_seed_serde! {
                @selector_seed_codec_deseed { $($generics)* } { $($bounds)* } $ty
            {
                // #[inline]
                // fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                // where
                //     D: Deserializer<'de>,
                // {
                #[inline]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    if _D {
                        unimplemented!()
                    } else {
                        let inner_seed = macros::impl_selector_seed_serde! {
                            @selector_seed_wrap
                            // TODO: probably need to pass a closure
                            self { $ty => $inner_ty }
                        };

                        // cfg_if::cfg_if! {
                        //     if #[cfg(feature = "serde-codec")] {
                        //         // (&mut &mut &mut Decoder(deserializer)).deserialize_any(self)
                        //         inner_seed.deserialize((&mut &mut &mut Decoder(deserializer)))
                        //     } else {
                        //         // deserializer.deserialize_any(self)
                        //         inner_seed.deserialize(deserializer)
                        //     }
                        // }

                        // inner_seed.deserialize::<C, _>(deserializer)
                        inner_seed.deserialize(deserializer)
                    }
                }
            }}
        };
        // produces an seed for the inner_ty, providing the original cb with ty
        (@selector_seed_wrap $seed:ident { $constructor:expr => $inner_ty:ty }) => {{
            use $crate::dev::{DagSelection, Callback::*};

            let (selector, state, callback, ctx) = $seed.0.into_parts();
            let callback = match callback {
                SelectNode { cb, only_matched } => SelectNode { cb, only_matched },
                SelectDag { mut cb } => SelectDag {
                    cb: Box::new(move |selection, ctx| {
                        let inner_dag = selection.dag.downcast::<$inner_ty>()?;
                        let dag = $constructor(inner_dag.into());
                        cb(DagSelection { dag: dag.into(), ..selection }, ctx)
                    }),
                },
                _ => unreachable!(),
            };
            Self::from(SelectorSeed::from_parts(selector, state, callback, ctx))
        }};

    // (@empty $seed:ident $constructor:expr => $inner_ty:tt) => { unimplemented!() };
    // TODO: deprecate, since we want SelectorSeed to do all the heavy-lifting
    // (@select_newtype
    //     { $($generics:tt)* } { $($bounds:tt)* }
    //     $ty:ident as $inner_ty:ident
    // ) => {
    //     /// Delegates directly to the inner type's [`Select`] implmentation,
    //     /// wrapping the provided callbacks to ensure the expected types are
    //     /// selected.
    //     ///
    //     /// [`Select`]: crate::dev::Select
    //     impl<Ctx, $($generics)*> $crate::dev::Select<Ctx> for $ty
    //     where
    //         Ctx: $crate::dev::Context,
    //         $inner_ty: $crate::dev::Select<Ctx>,
    //         $($bounds)*
    //     {
    //         fn select(
    //             params: $crate::dev::Params<'_, Ctx, Self>,
    //             ctx: &mut Ctx,
    //         ) -> Result<(), $crate::dev::Error> {
    //             use $crate::dev::Callback::*;

    //             let params = $crate::dev::Params::<'_, Ctx, $inner_ty> {
    //                 cid: params.cid,
    //                 selector: params.selector,
    //                 max_path_depth: params.max_path_depth,
    //                 max_link_depth: params.max_link_depth,
    //                 callback: match params.callback {
    //                     SelectNode { cb, only_matched } => SelectNode { cb, only_matched },
    //                     SelectDag { mut cb } => SelectDag {
    //                         cb: Box::new(move |selection, ctx| {
    //                             let dag = selection.dag.downcast::<$inner_ty>()?;
    //                             let selection = $crate::dev::DagSelection {
    //                                 dag: Self(dag).into(),
    //                                 ..selection
    //                             };
    //                             cb(selection, ctx)
    //                         }),
    //                     },
    //                     _ => unreachable!(),
    //                 },
    //             };

    //             <$inner_ty>::select(params, ctx)
    //         }
    //     }
    // };

    // (visit_self
    //     { $($generics:tt)* } { $($bounds:tt)* }
    //     $type:ty
    //     { $($visit_fn:tt)* }
    //     { $($flush_fn:tt)* }
    // ) => {
    //     impl<Ctx, $($generics)*> $crate::dev::Visit<Ctx> for $type
    //     where
    //         Ctx: $crate::dev::Context,
    //         $($bounds)*
    //     {
    //         // #[inline]
    //         // fn r#match(
    //         //     selector: &$crate::dev::Selector,
    //         //     state: $crate::dev::SelectorState,
    //         //     ctx: &mut Ctx
    //         // ) -> Result<Option<Self>, $crate::dev::Error> {
    //         //     let deserializer = ctx.path_decoder(state.path())?;
    //         //     $crate::dev::SelectorSeed::<'_, Ctx, Self, Self>::from(selector, state, ctx)
    //         //         .deserialize(deserializer)
    //         //         .map_err($crate::dev::Error::decoder)
    //         // }
    //
    //         fn visit<F, T: $crate::dev::Representation>(
    //             &mut self,
    //             selector: &$crate::dev::Selector,
    //             state: $crate::dev::SelectorState,
    //             ctx: &mut Ctx,
    //             op: F,
    //         ) -> Result<(), $crate::dev::Error>
    //         where
    //             F: Fn(T, &mut Ctx) -> Result<Option<T>, $crate::dev::Error>,
    //         {
    //             unimplemented!()
    //         }
    //
    //         fn flush(
    //             &mut self,
    //             selector: &$crate::dev::Selector,
    //             state: $crate::dev::SelectorState,
    //             ctx: &mut Ctx,
    //         ) -> Result<(), $crate::dev::Error> {
    //             unimplemented!()
    //         }
    //     }
    // };
}

/*
/// Returns an iterator ...
pub fn seq_iter<'a: 'de, 'de, Ctx, A, T, U>(
    selector: &'a Selector,
    mut state: &'a mut SelectorState,
    callback: Params<'_, C, T, Option<U>>,
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
            match T::select::<Option<U>>(&selector, &mut state, callback, &mut ctx).transpose() {
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
            // if type_eq::<SelectorSeed<'_, C, T, U>, V>() {
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

// default impl<'de, 'a, C, T, U> DeserializeSeed<'de> for SelectorSeed<'a, C, T, U>
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

 */
