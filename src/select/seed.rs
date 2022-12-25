use std::{borrow::Borrow, ops::Deref};

use crate::dev::{macros::derive_more::From, *};
use maybestd::{cell::Cell, fmt, marker::PhantomData, mem, str::FromStr};

/// The selection mode of the selector, which determines what gets visited,
/// matched, sent and returned.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub enum SelectionMode {
    /// ...
    Decode,

    /// Selection will invoke the provided callback on all traversed [`Node`]s.
    CoverNode,

    /// Selection will invoke the provided callback on all matched [`Node`]s.
    MatchNode,

    /// Selection will invoke the provided callback on all matched [`Dag`]s.
    Select,

    ///
    SelectRef,

    /// Selection will invoke the provided callback on all matched [`Dag`]s,
    /// optionally mutating the dag updates matching dags with the output of a callback.
    /// Optionally flushes changes after each callback.
    Patch,
}

/// A helper type for guided decoding of a dag, using a selector to direct
/// and/or ignore fields or entire blocks, and a linked context to fetch more
/// blocks.
///
/// todo: sharing logic, parameterizing over DM_KIND and REPR_KIND
/// - the Visitor impl:
///     - is called by the REPR of the type
///         e.g. the block encodes a list, map, etc
///     - creates an iterator based on the REPR of the type
///         e.g. Map creates MapIterator from MapAccess
///         e.g. StringpairsMap creates a MapIterator from a string
///     - calls SelectorSeed::<selector> method based on DM_KIND of the type
///         e.g. List* calls match, explore_list_*
///         e.g.
/// ? - ergo, for ADLs:
///     ? - map the input selector to another tailored for the underlying types
pub struct SelectorSeed<'a, Ctx, T>
where
    T: Representation,
{
    pub(crate) selector: &'a Selector,
    pub(crate) state: &'a mut State,
    pub(crate) callback: Option<Callback<'a, Ctx>>,
    pub(crate) ctx: Ctx,
    _t: PhantomData<T>,
}

///
pub type EmptySeed<Ctx = (), T = Any> = SelectorSeed<'static, Ctx, T>;

// Blanket impl for all `CodecSeed`s that implement `Visitor` for a given `T`.
// Doing this allows us to more easily "escape" [`serde`]'s traits into our own
// methods for selection, requiring only that we implement [`Visitor`] for
// `CodecSeed<..., T>`.
repr_serde!(@def_walk);

/*
#[doc(hidden)]
pub trait SeedType<T, U = T>: Sized {
    // type Input;
    type Output;
    type Wrapped: SeedType<U>;
    const CAN_SELECT: bool;

    fn mode(&self) -> SelectionMode;
    fn selector(&self) -> &Selector;

    // fn select_node() -> Result<(), Error> {
    //     unimplemented!()
    // }
    // fn get_mut(&mut self) -> &mut T {
    //     unimplemented!()
    // }

    fn wrap<F>(self, conv: F) -> Option<Self::Wrapped> {
        unimplemented!()
    }
}
impl<T, U> SeedType<T, U> for PhantomData<T> {
    // type Input = ();
    type Output = T;
    type Wrapped = PhantomData<U>;
    const CAN_SELECT: bool = false;

    fn mode(&self) -> SelectionMode {
        SelectionMode::Match
    }
    fn selector(&self) -> &Selector {
        &DEFAULT_SELECTOR
    }
}
impl<'a, Ctx, T, U> SeedType<T, U> for SelectorSeed<'a, Ctx, T> {
    // type Input = &'a mut T;
    type Output = ();
    type Wrapped = SelectorSeed<'a, Ctx, U>;
    const CAN_SELECT: bool = true;

    fn mode(&self) -> SelectionMode {
        self._mode()
    }
    fn selector(&self) -> &Selector {
        self.selector
    }
}
 */

// impl<'de, const C: u64, T> CodecSeed<C, true, PhantomData<T>, T>
// // where
// //     Self: Visitor<'de>,
// {
//     #[inline]
//     #[doc(hidden)]
//     pub fn empty() -> Self {
//         CodecSeed(PhantomData, PhantomData)
//     }
// }
//
// impl<'a, const C: u64, Ctx, T> CodecSeed<C, SelectorSeed<'a, Ctx, T>> {
//     fn from(inner: SelectorSeed<'a, Ctx, T>) -> Self {
//         Self(inner)
//     }
// }
// impl<const C: u64, T> CodecSeed<C, true, PhantomData<T>> {
//     fn from(inner: PhantomData<T>) -> Self {
//         Self(inner)
//     }
// }
// impl<'a, const C: u64, Ctx, T> From<SelectorSeed<'a, Ctx, T>>
//     for CodecSeed<C, false, SelectorSeed<'a, Ctx, T>, T>
// {
//     fn from(inner: SelectorSeed<'a, Ctx, T>) -> Self {
//         Self(inner, PhantomData)
//     }
// }
// impl<const C: u64, T> From<PhantomData<T>> for CodecSeed<C, true, PhantomData<T>, T> {
//     fn from(inner: PhantomData<T>) -> Self {
//         Self(inner, PhantomData)
//     }
// }
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

impl<'a, Ctx, T> fmt::Debug for SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectorSeed")
            .field("selector", &self.selector)
            .field("state", &self.state)
            .field("callback", &self.callback)
            .finish()
    }
}

// TODO: be lazier: provide a mode method that lets us skip any dag-deserialization work, while also letting us just call what we want without a lot of branching
impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    T: Representation,
{
    ///
    #[inline]
    pub const fn _mode(&self) -> SelectionMode {
        match (&self.callback, self.selector.is_matcher()) {
            (None, _) => SelectionMode::Decode,
            (Some(Callback::SelectNode { .. }), true) => SelectionMode::MatchNode,
            (Some(Callback::SelectNode { .. }), _) => SelectionMode::CoverNode,
            // (Callback::SelectDag { .. }, true) | (Callback::MatchDag { .. }, _) => {
            //     SelectionMode::Match
            // }
            (Some(Callback::SelectDag { .. }), _) => SelectionMode::Select,
            (Some(Callback::SelectRef { .. }), _) => SelectionMode::SelectRef,
            (Some(Callback::Patch { .. }), _) => SelectionMode::Patch,
        }
    }

    ///
    #[inline]
    pub const fn _selector(&self) -> &Selector {
        &self.selector
    }

    ///
    #[inline]
    pub const fn is_decode(&self) -> bool {
        match self._mode() {
            SelectionMode::Decode => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub const fn is_select_node(&self) -> bool {
        match self._mode() {
            SelectionMode::CoverNode | SelectionMode::MatchNode => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub const fn is_select_dag(&self) -> bool {
        match self._mode() {
            SelectionMode::Select | SelectionMode::Decode => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub const fn is_select_ref(&self) -> bool {
        match self._mode() {
            SelectionMode::SelectRef => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub const fn is_patch(&self) -> bool {
        match self._mode() {
            SelectionMode::Patch => true,
            _ => false,
        }
    }
}

// impl<'a, Ctx, T> Drop for SelectorSeed<'a, Ctx, T>
// where
//     T: Representation,
// {
//     fn drop(&mut self) {
//         self.state.ascend::<T>();
//         mem::drop(self.callback.take());
//     }
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
    pub fn from_parts(
        selector: &'a Selector,
        state: &'a mut State,
        callback: Option<Callback<'a, Ctx>>,
        ctx: Ctx,
    ) -> Self {
        Self {
            selector,
            state,
            callback,
            ctx,
            _t: PhantomData,
        }
    }

    ///
    #[inline]
    #[doc(hidden)]
    pub(crate) fn into_parts(
        self,
    ) -> (&'a Selector, &'a mut State, Option<Callback<'a, Ctx>>, Ctx) {
        (self.selector, self.state, self.callback, self.ctx)
    }

    // ///
    // pub fn wrap<U, F>(self, conv: F) -> SelectorSeed<'a, Ctx, U>
    // where
    //     Ctx: 'a,
    //     T: 'static,
    //     U: Representation + 'static,
    //     // T: From<U> + 'static,
    //     F: Fn(U) -> T + Clone + 'a,
    // {
    //     let Self {
    //         selector,
    //         state,
    //         callback,
    //         ctx,
    //     } = self;
    //     SelectorSeed {
    //         selector,
    //         state,
    //         callback: callback.map(|c| c.wrap::<U, F>(conv)),
    //         ctx,
    //     }
    // }

    // ///
    // fn wrap_advanced<U, F>(self, conv: F) -> SelectorSeed<'a, Ctx, U>
    // where
    //     Ctx: 'a,
    //     T: 'static,
    //     U: Representation + 'static,
    //     // T: From<U> + 'static,
    //     F: Fn(U) -> T + Clone + 'a,
    // {
    //     let Self {
    //         selector,
    //         state,
    //         callback,
    //         ctx,
    //     } = self;
    //     SelectorSeed {
    //         selector,
    //         state,
    //         callback: callback.wrap::<U, F>(conv),
    //         ctx,
    //     }
    // }
}

// dag selection methods
impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation,
{
    ///
    pub fn select_node(&mut self, node: SelectedNode) -> Result<AstResult<T>, Error> {
        let path = self.state.path();
        match (self._mode(), &mut self.callback, self.selector.as_matcher()) {
            (SelectionMode::MatchNode, Some(cb), Some(m)) => {
                cb.match_node(path, node, m.label.as_deref(), &mut self.ctx)?;
                Ok(AstResult::Ok)
            }
            (SelectionMode::CoverNode, Some(cb), _) => {
                cb.cover_node(path, node, &mut self.ctx)?;
                Ok(AstResult::Ok)
            }
            _ => unreachable!(),
        }
    }

    ///
    pub fn select_dag(&mut self, dag: T) -> Result<AstResult<T>, Error>
    where
        T: 'static,
    {
        let path = self.state.path();
        match (self._mode(), &mut self.callback, self.selector.as_matcher()) {
            (SelectionMode::Decode, _, _) => Ok(AstResult::Value(dag)),
            (SelectionMode::Select, Some(cb), Some(m)) => {
                cb.match_dag(path, dag, m.label.as_deref(), &mut self.ctx)?;
                Ok(AstResult::Ok)
            }
            _ => self.select_ref(&dag),
        }
    }

    ///
    pub fn select_ref(&mut self, dag: &T) -> Result<AstResult<T>, Error>
    where
        T: 'static,
    {
        let path = self.state.path();
        match (self._mode(), &mut self.callback, self.selector.as_matcher()) {
            (SelectionMode::SelectRef, Some(cb), Some(m)) => {
                cb.match_ref(path, dag, m.label.as_deref(), &mut self.ctx)?;
                Ok(AstResult::Ok)
            }
            (SelectionMode::MatchNode | SelectionMode::CoverNode, _, _) => {
                self.select_node(dag.to_selected_node())
            }
            _ => unreachable!(),
        }
    }

    ///
    pub fn patch_dag(&mut self, dag: &mut T) -> Result<AstResult<T>, Error>
    where
        T: 'static,
    {
        let path = self.state.path();
        match (self._mode(), &mut self.callback, self.selector.as_matcher()) {
            (SelectionMode::Patch, Some(cb), Some(m)) => {
                cb.patch_dag(path, dag, m.label.as_deref(), &mut self.ctx)?;
                Ok(AstResult::Ok)
            }
            _ => unreachable!(),
        }
    }

    // pub(crate) fn select_index_dag<'b, U>(
    //     &mut self,
    //     field: &Field<'_>,
    // ) -> Result<Option<SelectorSeed<'b, Ctx, U>>, Error>
    // where
    //     'a: 'b,
    //     U: Representation,
    // {
    //     match self.selector.next(Some(field)) {
    //         None => Ok(None),
    //         Some(s) => Ok(Some(SelectorSeed::from_parts(
    //             s,
    //             self.state.descend::<U>(&field)?,
    //             self.callback.clone(),
    //             self.ctx,
    //         ))),
    //     }
    // }

    // pub(crate) fn select_index_ref(&mut self, )

    // pub(crate) fn patch_index_dag(&mut self, )

    // pub fn cover_node(&mut self, node: SelectedNode) -> Result<AstResult< T>, Error> {}

    // ///
    // pub fn handle_node(&mut self, node: SelectedNode) -> Result<(), Error> {
    //     if let Some(matcher) = self.selector.as_matcher() {
    //         self.callback
    //             .select_node(self.state.path(), node, matcher.label.as_deref(), self.ctx)
    //     } else {
    //         self.callback.cover_node(self.state.path(), node, self.ctx)
    //     }
    // }

    // ///
    // pub fn handle_dag(&mut self, dag: T) -> Result<(), Error>
    // where
    //     T: Representation + 'static,
    // {
    //     let matcher = self.selector.try_as_matcher()?;
    //     self.callback
    //         .select_dag((self.state.path(), dag, matcher.label.as_deref()), self.ctx)
    // }

    ///
    #[inline]
    pub fn to_field_select_seed<'b, U>(
        selector: &'b Selector,
        state: &'b mut State,
        callback: Option<Callback<'a, Ctx>>,
        ctx: Ctx,
        field: &Field<'b>,
        // match_cb: Option<Box<dyn MatchDagOp<U, Ctx> + 'b>>,
        // match_cb: Option<&'b mut dyn MatchDagOp<U, Ctx>>,
        // match_cb: Option<F>
    ) -> Result<Option<SelectorSeed<'b, Ctx, U>>, Error>
    where
        'a: 'b,
        Ctx: 'b,
        U: Representation,
        // F: FnOnce(U, Ctx) -> Result<(), Error>,
    {
        match selector.next(Some(field)) {
            None => Ok(None),
            Some(s) => Ok(Some(SelectorSeed::from_parts(
                s,
                state.descend::<U>(&field)?,
                callback.clone(),
                ctx,
            ))),
        }
    }

    /*
    /// Execute the next [`Selector`] against the next element of the
    /// [`ListIterator`].
    ///
    /// TODO: switch on mode, calling the right iter method
    ///
    /// T creates seed (+ iter)
    /// seed + iter -> select/patch_dm(iter) -> handle_index
    ///     - if selector for current index exists, descend
    ///     - else, ignore
    pub fn handle_index<'b, const MC: u64, U>(
        &'a mut self,
        iter: &'b mut impl ListIterator<'b, U>,
        // new_cb: Either<(), (&mut U, Box<dyn PatchDagOp<U, Ctx>>)>
        // match_cb: Option<Box<dyn MatchDagOp<U, Ctx> + 'b>>,
        // match_cb: Option<&'b mut dyn MatchDagOp<U, Ctx>>,
    ) -> Result<Option<AstResult<U>>, Error>
    where
        'a: 'b,
        U: Select<Ctx>,
    {
        // match self._mode() {
        //     SelectionMode::Decode => {

        //     },
        //     SelectionMode::CoverNode => unimplemented!(),
        //     SelectionMode::MatchNode => unimplemented!(),
        //     SelectionMode::Select => unimplemented!(),
        //     SelectionMode::SelectRef => unimplemented!(),
        //     SelectionMode::Patch => unimplemented!(),
        // }

        let res = iter.next_element_seed::<MC, Ctx, _>(|idx| {
            let field = Field::Index(idx);
            match self.selector.next(Some(&field)) {
                None => Ok(None),
                Some(selector) => Ok(Some(SelectorSeed {
                    selector,
                    state: self.state.descend::<U>(&field)?,
                    callback: self.callback.clone(),
                    ctx: self.ctx,
                    _t: PhantomData,
                })),
            }
        })?;
        match res {
            Some(AstResult::Ok) => Ok(Some(AstResult::Ok)),
            Some(AstResult::Value(v)) => Ok(Some(AstResult::Value(v))),
            // Some(AstResult::Continue) => Ok(Some(AstResult::Continue)),
            // Some(AstResult::Break) => Ok(Some(AstResult::Break)),
            // Some(Err(e)) => Err(e),
            _ => unimplemented!(),
        }
    }

    ///
    pub fn handle_field<'b, const MC: u64, K, V>(
        &'b mut self,
        iter: &mut impl MapIterator<K, V>,
        // match_cb: Option<Box<dyn MatchDagOp<V, Ctx> + 'b>>,
    ) -> Result<Option<AstResult<V>>, Error>
    where
        // 'a: 'b,
        K: StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Select<Ctx>,
    {
        let res =
            iter.next_entry_seed::<MC, Ctx, _>(|field| match self.selector.next(Some(&field)) {
                None => Ok(None),
                Some(selector) => Ok(Some(SelectorSeed {
                    selector,
                    state: self.state.descend::<V>(&field)?,
                    callback: self.callback.clone(),
                    ctx: self.ctx,
                    _t: PhantomData,
                })),
            })?;
        match res {
            Some(AstResult::Ok) => Ok(Some(AstResult::Ok)),
            Some(AstResult::Value(v)) => Ok(Some(AstResult::Value(v))),
            // Some(AstResult::Continue) => Ok(Some(AstResult::Continue)),
            // Some(AstResult::Break) => Ok(Some(AstResult::Break)),
            // Some(Err(e)) => Err(e),
            _ => unimplemented!(),
        }
    }
     */
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
#[doc(hidden)]
#[macro_export]
macro_rules! repr_serde {
    ///////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////

    // impls Visitor for $seed<const C, S: SeedType<T>, T>
    (@visitors for $T:ty { $($visit_fns:tt)* }) => {
        repr_serde!(@visitors for $T {} {} @serde { $($visit_fns)* });
    };
    (@visitors for $T:ty where
        @serde { $($visit_fns:tt)* }
        @link { $($visit_link_fns:tt)* }
    ) => {
        repr_serde!(@visitors for $T {} {}
            @serde { $($visit_fns)* } @link { $($visit_link_fns)* }
        );
    };
    (@visitors for $T:ty { $($generics:tt)* } { $($bounds:tt)* }
        @serde { $($visit_fns:tt)* }
    ) => {
        repr_serde!(@visitors for $T { $($generics)* } { $($bounds)* }
            @serde { $($visit_fns)* } @link {}
        );
    };
    (@visitors for $T:ty { $($generics:tt)* } { $($bounds:tt)* }
        @serde { $($visit_fns:tt)* }
        @link { $($visit_link_fns:tt)* }
    ) => {
        #[doc(hidden)]
        impl<'__a: 'de, 'de, const MC: u64, Ctx, $($generics)*>
            $crate::dev::Visitor<'de> for
            // Seed<MC, $crate::dev::SelectorSeed<'__a, Ctx, $T>, $T>
            AstWalk<'__a, MC, Ctx, $T>
        where
            Ctx: $crate::dev::Context,
            // $T: '__a,
            $($bounds)*
        {
            // type Value = <$crate::dev::SelectorSeed<'__a, Ctx, $T> as SeedType<$T>>::Output;
            type Value = AstResult<$T>;
            $($visit_fns)*
        }

        #[doc(hidden)]
        impl<'__a: 'de, 'de, const MC: u64, Ctx, $($generics)*>
            LinkVisitor<'de, MC> for
            // Seed<MC, $crate::dev::SelectorSeed<'__a, Ctx, $T>, $T>
            AstWalk<'__a, MC, Ctx, $T>
        where
            Ctx: $crate::dev::Context,
            // $T: '__a,
            $($bounds)*
        {
            $($visit_link_fns)*
        }
    };

    ///////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////

    // impls Select for T, where $seed<C, S, T>: DeserializeSeed
    (@select for $T:ty) => { repr_serde!(@select for $T {} {}); };
    (@select for $T:ty { $($generics:tt)* } { $($bounds:tt)* }) => {
        impl<Ctx, $($generics)*> $crate::dev::Select<Ctx> for $T
        where
            $T: 'static,
            Ctx: $crate::dev::Context,
            $($bounds)*
        {
            #[doc(hidden)]
            type Walker<'__a, const MC: u64> = AstWalk<'__a, MC, Ctx, Self> where Ctx: '__a;

            // #[doc(hidden)]
            // #[inline]
            // fn __select_de<'a, 'de, const MC: u64, D>(
            //     seed: $crate::dev::SelectorSeed<'a, Ctx, Self>,
            //     deserializer: D,
            // ) -> Result<(), D::Error>
            // where
            //     D: $crate::dev::Deserializer<'de>,
            // {
            //     // Seed::<MC, _, Self>::from(seed).deserialize(deserializer)
            //     AstWalk::<'a, MC, Ctx, $T>::from(seed).deserialize(deserializer)?;
            //     Ok(())
            // }
        }
    };

    ///////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////

    // defines a new Seed to be used as a DeserializeSeed and Visitor
    (@def_seed) => {
        use $crate::maybestd::marker::PhantomData as Phantom;

        #[doc(hidden)]
        pub(crate) struct Seed<const C: u64, S, T>(S, Phantom<T>);
        #[doc(hidden)]
        pub(crate) trait ISeed<S>: AsRef<S> + AsMut<S> + From<S> {
            const CAN_SELECT: bool;
            type Inner;
            fn into_inner(self) -> S;
            fn mode(&self) -> SelectionMode;
            fn selector(&self) -> &Selector;
        }
        impl<const C: u64, S: SeedType<T>, T> ISeed<S> for Seed<C, S, T> {
            const CAN_SELECT: bool = S::CAN_SELECT;
            type Inner = T;
            #[inline(always)]
            fn into_inner(self) -> S {
                self.0
            }
            #[inline(always)]
            fn selector(&self) -> &Selector {
                self.0.selector()
            }
            #[inline(always)]
            fn mode(&self) -> SelectionMode {
                self.0.mode()
            }
        }

        impl<const C: u64, S, T> AsRef<S> for Seed<C, S, T> {
            #[inline(always)]
            fn as_ref(&self) -> &S {
                &self.0
            }
        }
        impl<const C: u64, S, T> AsMut<S> for Seed<C, S, T> {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut S {
                &mut self.0
            }
        }
        impl<const C: u64, S, T> From<S> for Seed<C, S, T> {
            #[inline(always)]
            fn from(seed: S) -> Self {
                Self(seed, Phantom)
            }
        }

        impl<'de, const C: u64, S, T: Representation> DeserializeSeed<'de> for Seed<C, S, T>
        where
            Self: LinkVisitor<'de, C>,
            S: SeedType<T>,
            T: Representation,
        {
            type Value = <Self as Visitor<'de>>::Value;

            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                T::deserialize_with_visitor::<C, D, _>(deserializer, self)
            }
        }

        $crate::repr_serde!(@def_walk);
    };
    (@def_walk) => {
        /// A visitor that walks the AST
        // #[derive(Debug)]
        #[doc(hidden)]
        pub struct AstWalk<'a, const MC: u64 = IDENTITY, Ctx = (), T = Any>
        where
            Ctx: Context + 'a,
            T: Representation,
        {
            inner: Option<SelectorSeed<'a, Ctx, T>>,
        }

        impl<'a, const MC: u64, Ctx, T> AstWalk<'a, MC, Ctx, T>
        where
            Ctx: Context + 'a,
            T: Representation,
        {
            #[inline(always)]
            pub fn into_inner(self) -> SelectorSeed<'a, Ctx, T> {
                self.inner.unwrap()
            }
            #[inline(always)]
            pub fn selector(&self) -> &Selector {
                self.inner.as_ref().unwrap()._selector()
            }
            #[inline(always)]
            pub fn mode(&self) -> SelectionMode {
                self.inner.as_ref().map(|s| s._mode()).unwrap_or(SelectionMode::Decode)
            }
            #[inline(always)]
            pub fn from_parts(
                selector: &'a Selector,
                state: &'a mut State,
                callback: Option<Callback<'a, Ctx>>,
                ctx: Ctx,
            ) -> Self {
                let inner = Some(SelectorSeed::from_parts(
                    selector,
                    state,
                    callback,
                    ctx,
                ));
                Self { inner }
            }
            // pub fn decode<'de, __D>(deserializer: __D) -> Result<T, __D::Error>
            // where
            //     __D: Deserializer<'a>,
            //     Self: LinkVisitor<'de, MC, Value = AstResult<T>>,
            //     T: 'de,
            // {
            //     let res = DeserializeSeed::deserialize(AstWalk::<'de, MC, (), T>::default(), deserializer)?;
            //     Ok(res.unwrap_val())
            // }
            //
            // #[inline]
            // pub(crate) fn select_in<'de, D>(
            //     self,
            //     deserializer: D,
            // ) -> Result<(), Error>
            // where
            //     'a: 'de,
            //     Self: LinkVisitor<'de, MC>,
            //     D: Deserializer<'de>,
            // {
            //     deserialize_with_visitor::<'de, MC, D, Self, T>(deserializer, self)
            //         .map_err(|_| Error::SelectionFailure(T::NAME.into()))?;
            //     Ok(())
            // }
            //
            // #[inline]
            // pub(crate) fn select<'de>(
            //     self,
            //     dag: &'de T,
            // ) -> Result<(), Error>
            // where
            //     'a: 'de,
            // {
            //     Ok(())
            // }
            //
            // #[inline]
            // pub(crate) fn patch<'de>(
            //     self,
            //     dag: &'de mut T,
            // ) -> Result<(), Error>
            // where
            //     'a: 'de,
            // {
            //     Ok(())
            // }
        }

        impl<'a, const MC: u64, Ctx, T> AsRef<SelectorSeed<'a, Ctx, T>> for AstWalk<'a, MC, Ctx, T>
        where
            Ctx: Context + 'a,
            T: Representation,
        {
            #[inline(always)]
            fn as_ref(&self) -> &SelectorSeed<'a, Ctx, T> {
                self.inner.as_ref().unwrap()
            }
        }
        impl<'a, const MC: u64, Ctx, T> AsMut<SelectorSeed<'a, Ctx, T>> for AstWalk<'a, MC, Ctx, T>
        where
            Ctx: Context + 'a,
            T: Representation,
        {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut SelectorSeed<'a, Ctx, T> {
                self.inner.as_mut().unwrap()
            }
        }
        impl<'a, const MC: u64, Ctx, T> Default for AstWalk<'a, MC, Ctx, T>
        where
            Ctx: Context + 'a,
            T: Representation,
        {
            fn default() -> Self {
                Self { inner: None }
            }
        }
        impl<'a, const MC: u64, Ctx, T> From<SelectorSeed<'a, Ctx, T>> for AstWalk<'a, MC, Ctx, T>
        where
            Ctx: Context + 'a,
            T: Representation,
        {
            #[inline(always)]
            fn from(inner: SelectorSeed<'a, Ctx, T>) -> Self {
                Self { inner: Some(inner) }
            }
        }

        #[doc(hidden)]
        impl<'a: 'de, 'de, const MC: u64, Ctx, T> DeserializeSeed<'de> for AstWalk<'a, MC, Ctx, T>
        where
            Self: LinkVisitor<'de, MC, Value = AstResult<T>>,
            Ctx: Context + 'a,
            T: Representation + 'de,
        {
            type Value = <Self as Visitor<'de>>::Value;

            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                T::deserialize_with_visitor::<MC, D, _>(deserializer, self)
            }
        }
    };
}

/// ideas for refactor
///
/// notes:
/// - serde == ||basic-data-model|| access to ||encoded|| data
/// - ipld data model ==
/// - ipld node == ||basic-data-model|| access to ||abstract-data-model|| data
///
/// ! reqs:
/// 0. select/patch against blocks
/// 1. easy support for selective-deserialization
/// 1b. share selection impl between block (select) and dag (select_in)
///     ==> AstWalk<T>.deserialize(block_de)
///     ==> AstWalk<T>.deserialize(self.into_deserializer())
///         e.g. tuple struct -> tupledeserializer<T>, so it knows fields
///         => each IntoDeserializer must support T::REPR (for select(_in))
///             (? DM, SCHEMA?)
///         => [opt] Into(Map/Seq/Enum/Struct)Access for Ref(Mut)<T>
/// 1c. share Visitor logic between decode (Phantom) and select (Seed)
///     - Phantom<T>.deserialize(...) vs AstWalk<T>.deserialize(...)
///         i.e. AstWalk.seed.select(T::from(val)) vs return T::from(val)
///     - each visit fn:
///         - is given a source of value(s) from a REPRESENTATION
///         - MUST decide to:
///             1. drain source (by SCHEMA) and return T
///             2. selectively drain source (by SCHEMA) and return None
///             3. ...?
///     ? ==> SelectorSeed.explore_list(seq)
///         -
///
/// ! nice to haves:
/// 1. serde-compatible
/// 2.
/// 3. ? (de)serialize as another repr
///     - aka follow another type's (de)serialize instructions
///     ... U::deserialize(t.into_deserializer())
///
/// ! dream features:
/// 1. provide hooks forcompat with rkyv / other frameworks
pub trait Walk<'de, const MC: u64 = IDENTITY, T = Any>
where
    // T: Representation + 'de,
    Self: Default
        // + LinkVisitor<'de, MC, Value = AstResult<T>>
        + DeserializeSeed<'de, Value = AstResult<T>>,
{
}

impl<'de, const MC: u64, T, W> Walk<'de, MC, T> for W where
    // T: Representation + 'de,
    W: Default
        // + LinkVisitor<'de, MC, Value = AstResult<T>>
        + DeserializeSeed<'de, Value = AstResult<T>>
{
}

// impl<'de, 'a: 'de, const FULL: bool, W, T> DeserializeSeed<'de> for W
// where
//     T: Representation,
//     W: Walk<'de, 'a, T, FULL>,
// {
//     unimplemented!()
// }

// ///
// #[derive(Debug, Default)]
// pub struct AstWalk<'a, const MC: u64 = IDENTITY, Ctx = (), T = Any>
// where
//     Ctx: Context,
//     T: Representation,
// {
//     inner: Option<SelectorSeed<'a, Ctx, T>>,
//     // continuation:
// }

///
#[derive(Debug)]
pub enum AstResult<T> {
    ///
    Ok,
    ///
    Value(T),
    ///
    Continue(Selector, State),
    // ///
    // Ref(&'a T),
    // ///
    // RefMut(&'a mut T),
    // Continue(
    //     (&Selector, State) // SelectorSeed<'a, Ctx, T>
    // ),
}

impl<T> AstResult<T> {
    #[inline(always)]
    pub(crate) fn unwrap_val(self) -> T {
        match self {
            AstResult::Value(val) => val,
            _ => unreachable!("AstResult return type should be known"),
        }
    }
}

// impl<'a, T> Try for AstResult<'a, T> {
//     type Ok = T;
//     type Error = Error;

//     #[inline(always)]
//     fn into_result(self) -> Result<Self::Ok, Self::Error> {
//         match self {
//             AstResult::Ok => Ok(()),
//             AstResult::Value(val) => Ok(val),
//             _ => unreachable!("AstResult return type should be known"),
//         }
//     }

//     #[inline(always)]
//     fn from_error(_: Self::Error) -> Self {
//         unreachable!("AstResult return type should be known")
//     }

//     #[inline(always)]
//     fn from_ok(val: Self::Ok) -> Self {
//         AstResult::Value(val)
//     }
// }

// impl<'de, 'a: 'de, const MC: u64, Ctx, T> DeserializeSeed<'de> for AstWalk<'a, MC, Ctx, T>
// where
//     Self: LinkVisitor<'de, MC, Value = AstResult<T>>,
//     T: Representation,
// {
//     type Value = AstResult<T>;

//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         T::deserialize_with_visitor::<MC, D, _>(deserializer, self)
//     }
// }

// impl<'de, const C: u64, S, T: Representation> DeserializeSeed<'de> for S
// where
//     S: Visitor<'de, Value = > + LinkVisitor<'de>,
//     T: Representation,
// {
//     type Value = <Self as Visitor<'de>>::Value;

//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         T::deserialize_with_visitor::<C, D, _>(deserializer, self)
//     }
// }
