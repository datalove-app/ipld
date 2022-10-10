use super::*;
use crate::dev::{macros::derive_more::From, *};
use std::{fmt, marker::PhantomData};

/// The selection mode of the selector, which determines what gets visited,
/// matched, sent and returned.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub enum SelectionMode {
    /// Selection will invoke the provided callback on all traversed [`Node`]s.
    CoverNode,

    /// Selection will invoke the provided callback on all matched [`Node`]s.
    MatchNode,

    /// Selection will invoke the provided callback on all matched [`Dag`]s.
    MatchDag,
    // /// Selection updates matching dags with the output of a callback.
    // /// Optionally flushes changes after each callback.
    // Patch,
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
pub struct SelectorSeed<'a, Ctx, T = Any> {
    pub(crate) selector: &'a Selector,
    pub(crate) state: &'a mut State,
    pub(crate) callback: Callback<'a, Ctx, T>,
    pub(crate) ctx: &'a mut Ctx,
}

pub type EmptySeed<T> = SelectorSeed<'static, (), T>;

/// A marked [`SelectorSeed`] that's aware of the codec of the block it's
/// currenly selecting against.
#[doc(hidden)]
#[derive(Debug)]
pub struct CodecSeed<const C: u64, S, T = Any, U = T, RK = <T as Representation>::ReprKind>(
    pub(crate) S,
    PhantomData<(T, U, RK)>,
)
where
    T: Representation<ReprKind = RK>,
    U: Representation,
    RK: TypedKind;

impl<const C: u64, S, T: Representation, U: Representation> CodecSeed<C, S, T, U> {
    // pub const RK: u32 = RK;
    // pub const fn is_select() -> bool {
    //     (!RK)
    // }
    // pub const fn repr_kind() -> Kind {
    //     T::REPR_KIND
    // }
    // pub const fn schema_kind() -> Kind {
    //     T::SCHEMA_KIND
    // }

    pub fn from(seed: S) -> Self {
        Self(seed, PhantomData)
    }

    const fn repr_kind() -> Kind
    where
        T: Representation,
    {
        if Kind::TypedInt.contains(T::REPR_KIND) {
            Int::REPR_KIND
        } else if Kind::TypedFloat.contains(T::REPR_KIND) {
            Float::REPR_KIND
        } else {
            T::REPR_KIND
        }
    }
}

/// Blanket impl for all `CodecSeed`s that implement `Visitor` for a given `T`.
/// Doing this allows us to focus selection implementations on what to do when
/// visiting a particular representation.
///
impl<'a, 'de, const C: u64, S, T, U, RK> DeserializeSeed<'de> for CodecSeed<C, S, T, U, RK>
where
    Self: Visitor<'de> + IpldVisitorExt<'de>,
    S: SeedType<T>,
    T: Representation<ReprKind = RK>,
    U: Representation,
    RK: TypedKind,
{
    type Value = <Self as Visitor<'de>>::Value;

    #[inline]
    fn deserialize<De>(self, deserializer: De) -> Result<Self::Value, De::Error>
    where
        De: Deserializer<'de>,
    {
        use Kind as K;

        if T::__IGNORED {
            return deserializer.deserialize_ignored_any(self);
        }

        match Self::repr_kind() {
            K::Null => deserializer.deserialize_unit(self),
            K::Bool => deserializer.deserialize_bool(self),
            K::Int8 => deserializer.deserialize_i8(self),
            K::Int16 => deserializer.deserialize_i16(self),
            K::Int32 => deserializer.deserialize_i32(self),
            K::Int64 => deserializer.deserialize_i64(self),
            K::Int128 => deserializer.deserialize_i128(self),
            K::Uint8 => deserializer.deserialize_u8(self),
            K::Uint16 => deserializer.deserialize_u16(self),
            K::Uint32 => deserializer.deserialize_u32(self),
            K::Uint64 => deserializer.deserialize_u64(self),
            K::Uint128 => deserializer.deserialize_u128(self),
            K::Float32 => deserializer.deserialize_f32(self),
            K::Float64 => deserializer.deserialize_f64(self),
            K::String => deserializer.deserialize_str(self),
            K::Bytes => {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::deserialize_bytes(deserializer, self);
                }
                deserializer.deserialize_bytes(self)
            }
            K::List => deserializer.deserialize_seq(self),
            K::Map => deserializer.deserialize_map(self),
            K::Link => {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::deserialize_cid(deserializer, self);
                }
                #[cfg(feature = "dag-cbor")]
                if C == DagCbor::CODE {
                    return DagCbor::deserialize_cid(deserializer, self);
                }
                deserializer.deserialize_any(self)
            }
            _ => {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::deserialize_any(deserializer, self);
                }
                #[cfg(feature = "dag-cbor")]
                if C == DagCbor::CODE {
                    return DagCbor::deserialize_any(deserializer, self);
                }
                deserializer.deserialize_any(self)
            }
        }
    }
}

#[doc(hidden)]
pub trait SeedType<T> {
    type Output;
    const CAN_SELECT: bool;
}
impl<T> SeedType<T> for PhantomData<T> {
    type Output = T;
    const CAN_SELECT: bool = false;
}
impl<'a, Ctx, T> SeedType<T> for SelectorSeed<'a, Ctx, T> {
    type Output = ();
    const CAN_SELECT: bool = true;
}

impl<'a, const C: u64, Ctx, T> CodecSeed<C, SelectorSeed<'a, Ctx, T>, T>
where
    T: Representation,
{
    // ///
    // #[inline]
    // pub(crate) fn from_parts(
    //     selector: &'a Selector,
    //     state: &'a mut State,
    //     callback: Callback<'a, Ctx, T>,
    //     ctx: &'a mut Ctx,
    // ) -> Self {
    //     Self(
    //         SelectorSeed {
    //             selector,
    //             state,
    //             callback,
    //             ctx,
    //             // visitor,
    //             // _t: PhantomData,
    //         },
    //         PhantomData,
    //     )
    // }

    // ///
    // #[inline]
    // #[doc(hidden)]
    // pub(crate) fn into_parts(
    //     self,
    // ) -> (
    //     &'a Selector,
    //     &'a mut State,
    //     Callback<'a, Ctx, T>,
    //     &'a mut Ctx,
    // ) {
    //     (self.0.selector, self.0.state, self.0.callback, self.0.ctx)
    // }
}

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

// ///
// #[doc(hidden)]
// pub type CodedSelectorSeed<'a, const C: u64, Ctx, T> =
//     CodecSeed<C, SelectorSeed<'a, Ctx, T>, T>;

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
//     fn deserialize<const C: u64, RK>(self, deserializer: RK) -> Result<Self::Value, RK::Error>
//     where
//         // CodecSeed<C, false, Self>: DeserializeSeed<'de, Value = ()>,
//         RK: Deserializer<'de>;
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
// //     fn deserialize<const C: u64, RK>(self, deserializer: RK) -> Result<Self::Value, RK::Error>
// //     where
// //         RK: Deserializer<'de>,
// //     {
// //         <T as Representation>::deserialize::<C, RK>(deserializer)
// //     }
// // }
// //
// // impl<'a, 'de, Ctx, T> CodecDeserializeSeed<'de> for SelectorSeed<'a, Ctx, T>
// // // where
// // //     CodecSeed<C, false, Self>: DeserializeSeed<'de, Value = ()>,
// // {
// //     const RK: u32 = false;
// //     type Value = ();
// //     fn deserialize<const C: u64, De>(self, deserializer: De) -> Result<(), De::Error>
// //     where
// //         // CodecSeed<C, { <SelectorSeed<'a, Ctx, T> as CodecDeserializeSeed<'de>>::RK }, Self>:
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

// TODO: be lazier: provide a mode method that lets us skip any dag-deserialization work, while also letting us just call what we want without a lot of branching
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
            Callback::SelectNode { .. } if self.selector.is_matcher() => SelectionMode::MatchNode,
            Callback::SelectNode { .. } => SelectionMode::CoverNode,
            Callback::SelectDag { .. } | Callback::MatchDag { .. } => SelectionMode::MatchDag,
            // Self::Patch { .. } => SelectionMode::Patch,
        }
    }

    ///
    #[inline]
    pub const fn is_node_select(&self) -> bool {
        match self.callback {
            Callback::SelectNode { .. } => true,
            _ => false,
        }
    }

    ///
    #[inline]
    pub const fn is_dag_select(&self) -> bool {
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
        }
    }

    ///
    pub fn wrap<U, F>(self, conv: F) -> SelectorSeed<'a, Ctx, U>
    where
        Ctx: 'a,
        U: Representation + 'static,
        T: 'static,
        // T: From<U> + 'static,
        F: Fn(U) -> T + Clone + 'a,
    {
        let Self {
            selector,
            state,
            callback,
            ctx,
        } = self;
        SelectorSeed {
            selector,
            state,
            callback: callback.wrap::<U, F>(conv),
            ctx,
        }
    }
}

// dag selection methods
impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation,
{
    ///
    pub fn select_node(&mut self, node: SelectedNode) -> Result<(), Error> {
        if let Some(matcher) = self.selector.as_matcher() {
            self.callback.select_node(
                self.state.path(),
                node,
                true,
                matcher.label.as_deref(),
                self.ctx,
            )
        } else {
            self.callback
                .select_node(self.state.path(), node, false, None, self.ctx)
        }
    }

    ///
    pub fn select_dag(&mut self, dag: T) -> Result<(), Error>
    where
        T: Representation + 'static,
    {
        let matcher = self.selector.try_as_matcher()?;
        self.callback
            .select_dag((self.state.path(), dag, matcher.label.as_deref()), self.ctx)
    }

    ///
    fn to_field_select_seed<'b, U>(
        &'b mut self,
        field: &Field<'b>,
        match_cb: Option<Box<dyn MatchDagOp<U, Ctx> + 'b>>,
        // match_cb: Option<&'b mut dyn MatchDagOp<U, Ctx>>,
        // match_cb: Option<F>
    ) -> Result<SelectorSeed<'b, Ctx, U>, Error>
    where
        // 'a: 'b,
        // Ctx: 'a,
        U: Representation,
        // F: FnOnce(U, Ctx) -> Result<(), Error>,
    {
        let selector = self.selector.try_next(Some(field))?;
        self.state.descend::<U>(&field)?;
        Ok(SelectorSeed {
            selector,
            state: self.state,
            callback: self.callback.wrap_match::<U>(match_cb),
            ctx: self.ctx,
        })
    }

    /// Execute the next [`Selector`] against the next element of the
    /// [`ListIterator`].
    pub fn select_index<'b, const C: u64, U>(
        &'b mut self,
        index: usize,
        match_cb: Option<Box<dyn MatchDagOp<U, Ctx> + 'b>>,
        // match_cb: Option<&'b mut dyn MatchDagOp<U, Ctx>>,
        iter: &mut impl ListIterator<U>,
    ) -> Result<bool, Error>
    where
        U: Select<Ctx>,
    {
        let res = iter.next_seed::<C, Ctx>({
            let field = index.into();
            self.state.descend::<U>(&field)?;
            SelectorSeed {
                selector: self.selector.try_next(Some(&field))?,
                state: self.state,
                callback: self.callback.wrap_match::<U>(match_cb),
                ctx: self.ctx,
            }
        })?;
        self.state.ascend::<U>()?;

        Ok(res)
    }

    ///
    pub fn select_field<'b, const C: u64, K, V>(
        &'b mut self,
        // key: &K,
        match_cb: Option<Box<dyn MatchDagOp<V, Ctx> + 'b>>,
        iter: &mut impl MapIterator<K, V>,
    ) -> Result<(), Error>
    where
        K: StringRepresentation,
        V: Select<Ctx>,
    {
        iter.next_value_seed::<C, Ctx>({
            let field = iter.field();
            self.state.descend::<V>(&field)?;
            SelectorSeed {
                selector: self.selector.try_next(Some(&field))?,
                state: self.state,
                callback: self.callback.wrap_match::<V>(match_cb),
                ctx: self.ctx,
            }
        })?;
        self.state.ascend::<V>()?;

        Ok(())
    }
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
    // visitor for CodedSeed

    // impl Visitor for CodedSeed
    (@codec_seed_visitor
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ident $(<
            // match one or more lifetimes separated by a comma
            $( $ty_generics:ident ),+
        >)?
        { $($visit_fns:tt)* }
    ) => {
        const _: () = {
            #[allow(unused_imports)]
            use $crate::dev::*;

            const _RK: u32 = <$ty as Representation>::REPR_KIND.bits();
            impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                Visitor<'de> for
                CodecSeed<_C,
                    SelectorSeed<'_a, Ctx, $ty $(<$($ty_generics),+>)?>,
                    $ty $(<$($ty_generics),+>)?
                >
            where
                Ctx: Context,
                $($bounds)*
            {
                type Value = ();
                $($visit_fns)*
            }
        };

    };
    // impl IpldVisitorExt for CodedSeed
    (@codec_seed_visitor_ext
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ident $(<
            // match one or more lifetimes separated by a comma
            $( $ty_generics:ident ),+
        >)?
        { $($visit_fns:tt)* }
    ) => {
        const _: () = {
            #[allow(unused_imports)]
            use $crate::dev::*;

            const _RK: u32 = <$ty as Representation>::REPR_KIND.bits();
            impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                IpldVisitorExt<'de> for
                CodecSeed<_C,
                    SelectorSeed<'_a, Ctx, $ty $(<$($ty_generics),+>)?>,
                    $ty $(<$($ty_generics),+>)?
                >
            where
                Ctx: Context,
                $($bounds)*
            {
                $($visit_fns)*
            }
        };
    };

    // impl Visitor for CodedSeed by REPR_KIND

    (@codec_seed_visitor_rk $rk:ident $ty:ident $marker_ty:ty
        { $($generics:tt)* } { $($bounds:tt)* }
        // $ty:ident
        //  $(<
        //     // match one or more lifetimes separated by a comma
        //     $( $ty_generics:ident ),+
        // >)?
        { $($visit_fns:tt)* }
    ) => {
        const _: () = {
            #[allow(unused_imports)]
            use $crate::dev::*;

            impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                Visitor<'de> for
                CodecSeed<_C, SelectorSeed<'_a, Ctx, $ty>, $ty, $marker_ty, type_kinds::$rk>
            where
                $ty: Representation<ReprKind = type_kinds::$rk> + Select<Ctx>,
                Ctx: Context,
                $($bounds)*
            {
                type Value = ();
                $($visit_fns)*
            }
        };

        // impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} $ty {} }
    };
    // impl Visitor for CodedSeed by REPR_KIND
    (@codec_seed_visitor_ext_rk $rk:ident $ty:ident $marker_ty:ty
        { $($generics:tt)* } { $($bounds:tt)* }
        // $ty:ident
        //  $(<
        //     // match one or more lifetimes separated by a comma
        //     $( $ty_generics:ident ),+
        // >)?
        { $($visit_fns:tt)* }
    ) => {
        const _: () = {
            #[allow(unused_imports)]
            use $crate::dev::*;

            impl<'_a, 'de, const _C: u64, Ctx, $($generics)*>
                IpldVisitorExt<'de> for
                CodecSeed<_C, SelectorSeed<'_a, Ctx, $ty>, $ty, $marker_ty, type_kinds::$rk>
            where
                $ty: Representation<ReprKind = type_kinds::$rk> + Select<Ctx>,
                Ctx: Context,
                $($bounds)*
            {
                $($visit_fns)*
            }
        };
    };

    // Select

        // impl Select for T, using the seed's select
        (@selector_seed_select
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
        ) => {
            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                impl<Ctx, $($generics)*> Select<Ctx> for $ty
                where
                    Ctx: Context,
                    $($bounds)*
                {
                    // #[inline]
                    // fn select(
                    //     params: Params<'_, Ctx, Self>,
                    //     ctx: &mut Ctx,
                    // ) -> Result<(), Error> {
                    //     SelectorSeed::<'_, Ctx, Self>::select(params, ctx)
                    // }

                    #[doc(hidden)]
                    fn __select_de<'a, 'de, const C: u64, D>(
                        seed: SelectorSeed<'a, Ctx, Self>,
                        deserializer: D,
                    ) -> Result<(), D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        let seed = CodecSeed::<C, _, Self, Self, Self::ReprKind>::from(seed);
                        seed.deserialize(deserializer)
                    }

                    // #[doc(hidden)]
                    // fn __select_seq<'a, 'de, const C: u64, A>(
                    //     seed: SelectorSeed<'a, Ctx, Self>,
                    //     mut seq: A,
                    // ) -> Result<Option<()>, A::Error>
                    // where
                    //     A: SeqAccess<'de>,
                    // {
                    //     let seed = CodecSeed::<C, false, _, Self>::from(seed);
                    //     seq.next_element_seed(seed)
                    // }

                    // #[doc(hidden)]
                    // fn __select_map<'a, 'de, const C: u64, A>(
                    //     seed: SelectorSeed<'a, Ctx, Self>,
                    //     mut map: A,
                    //     is_key: bool,
                    // ) -> Result<Option<()>, A::Error>
                    // where
                    //     A: MapAccess<'de>,
                    // {
                    //     let seed = CodecSeed::<C, false, _, Self>::from(seed);
                    //     if is_key {
                    //         map.next_key_seed(seed)
                    //     } else {
                    //         Ok(Some(map.next_value_seed(seed)?))
                    //     }
                    // }
                }
            };
        };
        /*
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
         */

    // newtype impls
        (@select_newtype
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty { $constructor:expr => $inner_ty:ty }
        ) => {
            const _: () = {
                #[allow(unused_imports)]
                use $crate::dev::*;

                impl<Ctx, $($generics)*> Select<Ctx> for $ty
                where
                    Ctx: Context,
                    $($bounds)*
                {
                    #[doc(hidden)]
                    #[inline]
                    fn __select<'a>(
                        seed: SelectorSeed<'a, Ctx, Self>,
                    ) -> Result<(), Error> {
                        let seed = seed.wrap::<$inner_ty, _>($constructor);
                        <$inner_ty>::__select(seed)
                    }

                    // #[doc(hidden)]
                    // #[inline]
                    // fn __select_de<'a, 'de, const C: u64, D>(
                    //     seed: SelectorSeed<'a, Ctx, Self>,
                    //     deserializer: D,
                    // ) -> Result<(), D::Error>
                    // where
                    //     D: Deserializer<'de>,
                    // {
                    //     let seed = seed.wrap::<$inner_ty, _>($constructor);
                    //     <$inner_ty>::__select_de::<C, D>(seed, deserializer)
                    // }

                    // #[doc(hidden)]
                    // #[inline]
                    // fn __select_seq<'a, 'de, const C: u64, A>(
                    //     seed: SelectorSeed<'a, Ctx, Self>,
                    //     seq: A,
                    // ) -> Result<Option<()>, A::Error>
                    // where
                    //     A: SeqAccess<'de>,
                    // {
                    //     let seed = seed.wrap::<$inner_ty, _>($constructor);
                    //     <$inner_ty>::__select_seq::<C, D>(seed, seq)
                    // }

                    // #[doc(hidden)]
                    // #[inline]
                    // fn __select_map<'a, 'de, const C: u64, A>(
                    //     seed: SelectorSeed<'a, Ctx, Self>,
                    //     map: A,
                    //     is_key: bool,
                    // ) -> Result<Option<()>, A::Error>
                    // where
                    //     A: MapAccess<'de>,
                    // {
                    //     let seed = seed.wrap::<$inner_ty, _>($constructor);
                    //     <$inner_ty>::__select_map::<C, D>(seed, map, is_key)
                    // }
                }
            };
        };

        // TODO: instead of transmuting cb in DeSeed, transmute it in Select
        (@selector_seed_codec_deseed_newtype
            { $($generics:tt)* } { $($bounds:tt)* }
            // TODO: this should probably by a ty
            $ty:ident as $inner_ty:ty
        ) => {
            $crate::dev::macros::impl_selector_seed_serde! {
                @selector_seed_codec_deseed { $($generics)* } { $($bounds)* } $ty
            {
                #[inline]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    if _RK {
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

            let SelectorSeed { selector, state, callback, ctx } = $seed.0;
            let callback = match callback {
                SelectDag { mut cb } => SelectDag {
                    cb: Box::new(move |selection, ctx| {
                        let inner_dag = selection.dag.downcast::<$inner_ty>()?;
                        let dag = $constructor(inner_dag.into());
                        cb(DagSelection { dag: dag.into(), ..selection }, ctx)
                    }),
                },
                cb => cb.clone(),
                _ => unreachable!(),
            };
            Self::from(SelectorSeed { selector, state, callback, ctx })
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
