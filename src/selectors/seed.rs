use super::*;
use crate::dev::*;

/// A helper type for guided decoding of a dag, using a selector to direct
/// and/or ignore fields or entire blocks, and a linked context to fetch more
/// blocks.
#[derive(Debug)]
pub struct ContextSeed<'a, C, T = Any>
where
    C: Context,
    T: Representation,
{
    pub(crate) selector: &'a Selector,
    pub(crate) state: &'a mut SelectionState,
    pub(crate) callback: SelectionCallback<'a, C, T>,
    pub(crate) ctx: &'a mut C,
}

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

impl<'a, C, T> ContextSeed<'a, C, T>
where
    C: Context,
    T: Representation,
{
    #[inline]
    pub(crate) const fn mode(&self) -> SelectionMode {
        self.callback.mode()
    }

    #[inline]
    pub(crate) const fn is_node(&self) -> bool {
        self.callback.is_node()
    }

    #[inline]
    pub(crate) const fn is_dag(&self) -> bool {
        self.callback.is_dag()
    }

    ///
    #[inline]
    pub(crate) fn from(
        selector: &'a Selector,
        state: &'a mut SelectionState,
        callback: SelectionCallback<'a, C, T>,
        ctx: &'a mut C,
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
        &'a mut SelectionState,
        SelectionCallback<'a, C, T>,
        &'a mut C,
    ) {
        (self.selector, self.state, self.callback, self.ctx)
    }

    ///
    #[inline]
    pub fn read<'de>(self, cid: &Cid) -> Result<(), Error>
    where
        Self: DeserializeSeed<'de, Value = ()>,
    {
        let mut codec = Multicodec::try_from(cid)?;
        let block_reader = self.ctx.block_reader(cid)?;
        codec.read_with_seed(self, block_reader)
    }

    // ///
    // #[inline]
    // pub fn encode(self, dag: &T) -> Result<(), Error> {
    //     // let mut encoder = self.ctx.encoder();
    //     // let encoder_mut = encoder.as_mut();
    //     // dag.serialize(encoder_mut).map_err(Error::encoder)?;
    //     // Ok(())

    //     unimplemented!()
    // }
}

// dag selection methods
impl<'a, C, T> ContextSeed<'a, C, T>
where
    C: Context,
    T: Representation,
{
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
        state: &'b mut SelectionState,
        callback: &mut SelectionCallback<'a, C, T>,
        ctx: &'b mut C,
        field: Field<'a>,
        match_cb: Option<Box<dyn MatchDagOp<U, C> + 'b>>,
    ) -> Result<ContextSeed<'b, C, U>, Error>
    where
        'a: 'b,
        U: Representation,
    {
        let next = selector
            .next(Some(&field))
            .ok_or_else(|| Error::missing_next_selector(selector))?;
        let callback = match (match_cb, callback) {
            //
            (None, SelectionCallback::SelectNode { cb, only_matched }) => {
                SelectionCallback::SelectNode {
                    cb: cb.clone(),
                    only_matched: *only_matched,
                }
            }
            //
            (None, SelectionCallback::SelectDag { cb }) => {
                SelectionCallback::SelectDag { cb: cb.clone() }
            }
            // matching the field
            (Some(field_cb), _) => SelectionCallback::MatchDag { cb: field_cb },
            _ => unreachable!(),
        };

        state.descend::<U>(field)?;
        Ok(ContextSeed::from(next, state, callback, ctx))
    }
}

// patch methods
impl<'a, C, T> ContextSeed<'a, C, T>
where
    C: Context,
    T: Representation,
{
    /*

    ///
    #[inline]
    pub fn to_field_patch_seed<'b, V, P>(
        self,
        field: P,
        parent_dag: &'b mut V,
    ) -> Result<(ContextSeed<'b, C, V, U>, &'b Selector), Error>
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

        let seed = ContextSeed {
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
            SelectionParams::Patch { op, flush, .. } => {
                // op(dag, self.ctx)?;
                op(self.ctx)?;

                if *flush {
                    // self.encode(&dag)?;
                }
                Ok(())
            }
            _ => Err(Error::InvalidSelectionMode(
                "`SelectionParams` not in patch mode",
            )),
        }
    }

     */
}

/*
/// Returns an iterator ...
pub fn seq_iter<'a: 'de, 'de, C, A, T, U>(
    selector: &'a Selector,
    mut state: &'a mut SelectorState,
    callback: SelectionParams<'_, C, T, Option<U>>,
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

 */
