use crate::dev::*;
use std::{
    marker::PhantomData,
    ops::{Range, RangeBounds},
};

///
pub type List<T = Value> = Vec<T>;

// TODO: write the 4 Select impls, then the latter 3 for Vec<Link<T>>

impl<T: Representation> Representation for List<T> {
    const NAME: &'static str = concat!("List<", stringify!(T::NAME), ">");
    const SCHEMA: &'static str = concat!(
        "type ",
        stringify!(Self::NAME),
        " = [",
        stringify!(T::NAME),
        "]",
    );
    const KIND: Kind = Kind::List;
    const HAS_LINKS: bool = T::HAS_LINKS;

    fn has_links(&self) -> bool {
        self.iter().any(|e| e.has_links())
    }
}

// #[derive(Deserialize, Serialize)]
// #[serde(transparent)]
// struct MyList(List<Int>);
// impl Representation for MyList {
//     const NAME: &'static str = "MyList";
//     const SCHEMA: &'static str = "type MyList [Int]";
//     const KIND: Kind = Kind::List;
// }
// impl_ipld_list!(MyList => Int);

///
/// ? should be explicitly implemented for each concrete List<T>
#[macro_export]
macro_rules! impl_ipld_list {
    ($inner_ty:ty) => {
        $crate::dev::impl_ipld_list! {
            @visit_list {} {} [ $inner_ty ]
        }
    };

    (@visit_list
        { $($generics:tt)* } { $($bounds:tt)* } [ $inner_ty:ty ]
    ) => {
        impl<'a, 'de, C, T, $($generics)*> $crate::dev::Visitor<'de> for $crate::dev::ContextSeed<'a, C, List<$inner_ty>, T>
        where
            T: $crate::dev::Representation + Send + Sync + 'static,
            C: $crate::dev::Context,
            for<'b> $crate::dev::ContextSeed<'b, C, $inner_ty, T>: $crate::dev::DeserializeSeed<'de, Value = Option<T>>,
            $($bounds)*
        {
            type Value = Option<T>;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", <List<$inner_ty>>::NAME)
            }

            #[inline]
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: $crate::dev::SeqAccess<'de>,
            {
                match self.selector {
                    s if type_eq::<List<$inner_ty>, T>() && s.is_matcher() => {
                        type_cast_selection::<List<$inner_ty>, T, _, _>(|| self.into::<List<$inner_ty>, T>().visit_list(seq))
                    },
                    s if s.is_explore_index() => self.into::<List<$inner_ty>, T>().visit_list_index(seq),
                    s if s.is_explore_range() => self.into::<List<$inner_ty>, T>().visit_list_range(seq),
                    s if s.is_explore_all() => self.into::<List<$inner_ty>, T>().visit_list_full(seq),
                    _ => Err(A::Error::custom($crate::Error::unsupported_selector::<List<$inner_ty>, T>(self.selector))),
                }
            }
        }
    };
}

// impl_ipld_list! { Int }
// impl_ipld_list! { List<Int> }
// impl_ipld_list!(@list_self {T: Representation} {} List<T> [ T ] List::<T>::from);

// impl<'a, 'de, C, T, U> Visitor<'de> for ContextSeed<'a, C, List<T>, U>
// where
//     T: Representation + Send + Sync + 'static,
//     U: Representation + Send + Sync + 'static,
//     C: Context,
//     // List<T>: TypeEq2<false, T>,
//     // T: TypeEq2<false, List<T>>,
//     // for<'b> ContextSeed<'b, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
// {
//     type Value = Option<U>;

//     #[inline]
//     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(formatter, "{}", List::<T>::NAME)
//     }

//     #[inline]
//     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: SeqAccess<'de>,
//     {
//         match self.selector {
//             s if s.is_matcher() && type_eq::<List<T>, U>() => {
//                 type_cast_selection::<List<T>, U, _, _>(|| {
//                     self.into::<List<T>, U>().visit_list(seq)
//                 })
//             }
//             s if s.is_explore_index() => self.into::<List<T>, U>().visit_list_index(seq),
//             s if s.is_explore_range() => self.into::<List<T>, U>().visit_list_range(seq),
//             s if s.is_explore_all() => self.into::<List<T>, U>().visit_list_full(seq),
//             _ => Err(A::Error::custom(Error::unsupported_selector::<List<T>, U>(
//                 self.selector,
//             ))),
//         }
//     }
// }

impl<'a, C, T, U> ContextSeed<'a, C, List<T>, U>
where
    C: Context,
    T: Representation + Send + Sync + 'static,
    U: Representation + Send + Sync + 'static,
{
    // ///
    // pub fn select_list<'de, A>(self, mut seq: A) -> Result<Option<List<T>>, A::Error>
    // where
    //     A: SeqAccess<'de>,
    //     // for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = Option<T>>,
    //     T: Select<C>,
    // {
    //     let Self {
    //         selector,
    //         state,
    //         ctx,
    //         ..
    //     } = self;

    //     let mut decoder = SeqIterDecoder::<_, C, T, U> {
    //         seq,
    //         end: false,
    //         _t: PhantomData,
    //     };
    //     ctx.set_decoder(&mut decoder)
    //     // match T::select::<>
    // }

    ///
    pub fn visit_list<'de, A>(self, mut seq: A) -> Result<Option<List<T>>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = Option<T>>,
    {
        let Self {
            selector,
            state,
            ctx,
            ..
        } = self;

        let matcher = selector
            .assert_matcher::<List<T>, U>()
            .map_err(A::Error::custom)?;

        match state.mode() {
            SelectorState::NODE_MODE => {
                state
                    .send_matched(Node::List, matcher.label.clone())
                    .map_err(A::Error::custom)?;

                for i in 0.. {
                    let seed = ContextSeed::<C, T>::from(selector, state, ctx)
                        .descend_index(i)
                        .map_err(A::Error::custom)?;
                    if let None = seq.next_element_seed(seed)? {
                        break;
                    }
                }

                Ok(None)
            }
            SelectorState::DAG_MATCH_MODE | SelectorState::DAG_MODE => {
                let mut dag = Vec::with_capacity(seq.size_hint().unwrap_or(256));

                for i in 0.. {
                    let seed = ContextSeed::<C, T>::from(selector, state, ctx)
                        .descend_index(i)
                        .map_err(A::Error::custom)?;
                    match seq.next_element_seed(seed)? {
                        Some(Some(inner)) => dag.push(inner),
                        None => break,
                        Some(None) => unreachable!(),
                    };
                }

                match state.mode() {
                    SelectorState::DAG_MATCH_MODE => Ok(Some(dag)),
                    SelectorState::DAG_MODE => {
                        state
                            .send_dag(dag, matcher.label.clone())
                            .map_err(A::Error::custom)?;
                        Ok(None)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    ///
    pub fn visit_list_index<'de, A>(self, mut seq: A) -> Result<Option<U>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
    {
        let Self {
            selector,
            state,
            ctx,
            ..
        } = self;

        unimplemented!()
    }

    ///
    pub fn visit_list_range<'de, A>(self, mut seq: A) -> Result<Option<U>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
    {
        let Self {
            selector,
            state,
            ctx,
            ..
        } = self;

        unimplemented!()
    }

    ///
    pub fn visit_list_full<'de, A>(self, mut seq: A) -> Result<Option<U>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
    {
        let Self {
            selector,
            state,
            ctx,
            ..
        } = self;

        unimplemented!()
    }
}

// /// Specialized implementation for selecting self.
// impl<'de, 'a, C: Context, T: Representation> Visitor<'de>
//     for ContextSeed<'a, C, List<T>, List<T>>
// where
//     T: Send + Sync + 'static,
//     // ContextSeed<'a, C, T, T>: DeserializeSeed<'de>,
// {
//     unimplemented!()
// }

// impl<'de, 'a, C: Context, T: Representation> Visitor<'de> for ContextSeed<'a, C, List<T>, List<T>>
// where
//     T: Send + Sync,
//     // ContextSeed<'a, C, List<T>, T>: DeserializeSeed<'de>,
//     // ContextSeed<'a, C, List<T>, T>: DeserializeSeed<'de>,
//     ContextSeed<'a, C, T, T>: Visitor<'de, Value = Option<T>>,
//     T: TypeEq2<false, List<T>>,
//     List<T>: TypeEq2<false, T>,
// {
//     type Value = Option<List<T>>;
//
//     #[inline]
//     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(formatter, "{}", List::<T>::NAME)
//     }
//
//     #[inline]
//     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: SeqAccess<'de>,
//     {
//         let Self {
//             selector,
//             state,
//             ctx,
//             ..
//         } = self;
//         let matcher = selector
//             .try_matcher::<List<T>, List<T>>()
//             .map_err(A::Error::custom)?;
//
//         match state.mode() {
//             SelectorState::NODE_MODE => {
//                 state
//                     .send_matched(Node::List, matcher.label.clone())
//                     .map_err(A::Error::custom)?;
//
//                 for i in 0.. {
//                     let seed = ContextSeed::<'_, C, T, T>::from(selector, state, ctx)
//                         .descend_index(i)
//                         .map_err(A::Error::custom)?;
//                     if let None = seq.next_element_seed(seed)? {
//                         break;
//                     }
//                 }
//
//                 Ok(None)
//             }
//             SelectorState::DAG_MATCH_MODE | SelectorState::DAG_MODE => {
//                 let mut dag = Vec::with_capacity(seq.size_hint().unwrap_or(256));
//
//                 for i in 0.. {
//                     let seed = ContextSeed::<'_, C, T, T>::from(selector, state, ctx)
//                         .descend_index(i)
//                         .map_err(A::Error::custom)?;
//                     match seq.next_element_seed(seed)? {
//                         Some(Some(inner)) => dag.push(inner),
//                         None => break,
//                         Some(None) => unreachable!(),
//                     };
//                 }
//
//                 match state.mode() {
//                     SelectorState::DAG_MATCH_MODE => Ok(Some(dag)),
//                     SelectorState::DAG_MODE => {
//                         state
//                             .send_dag(dag, matcher.label.clone())
//                             .map_err(A::Error::custom)?;
//                         Ok(None)
//                     }
//                     _ => unreachable!(),
//                 }
//             }
//         }
//     }
// }

// fn match_self<'a, 'de, C, T, A>(
//     seed: ContextSeed<'a, C, List<T>, List<T>>,
//     mut seq: A,
// ) -> Result<Option<List<T>>, A::Error>
// where
//     C: Context,
//     T: Representation + Send + Sync + 'static,
//     A: SeqAccess<'de>,
//     for<'b> ContextSeed<'b, &'b mut C, T, T>: DeserializeSeed<'de, Value = Option<T>>,
// {
//     let ContextSeed {
//         selector,
//         state,
//         ctx,
//         ..
//     } = seed;
//     let matcher = selector
//         .try_matcher::<List<T>, List<T>>()
//         .map_err(A::Error::custom)?;
//
//     match state.mode() {
//         SelectorState::NODE_MODE => {
//             state
//                 .send_matched(Node::List, matcher.label.clone())
//                 .map_err(A::Error::custom)?;
//
//             for i in 0.. {
//                 let seed = ContextSeed::<'_, C, T, T>::from(selector, state, ctx)
//                     .descend_index(i)
//                     .map_err(A::Error::custom)?;
//                 if let None = seq.next_element_seed(seed)? {
//                     break;
//                 }
//             }
//
//             Ok(None)
//         }
//         SelectorState::DAG_MATCH_MODE | SelectorState::DAG_MODE => {
//             let mut dag = Vec::with_capacity(seq.size_hint().unwrap_or(256));
//
//             for i in 0.. {
//                 let seed = ContextSeed::<'_, C, T, T>::from(selector, state, ctx)
//                     .descend_index(i)
//                     .map_err(A::Error::custom)?;
//                 match seq.next_element_seed(seed)? {
//                     Some(Some(inner)) => dag.push(inner),
//                     None => break,
//                     Some(None) => unreachable!(),
//                 };
//             }
//
//             match state.mode() {
//                 SelectorState::DAG_MATCH_MODE => Ok(Some(dag)),
//                 SelectorState::DAG_MODE => {
//                     state
//                         .send_dag(dag, matcher.label.clone())
//                         .map_err(A::Error::custom)?;
//                     Ok(None)
//                 }
//                 _ => unreachable!(),
//             }
//         }
//     }
// }

// impl<'de, 'a, C: Context, T: Representation, U: Representation> Visitor<'de>
//     for ContextSeed<'a, C, List<T>, U>
// where
//     T: Send + Sync + 'static,
//     U: 'static,
//     // ContextSeed<'a, &'a mut C, List<T>, T>: DeserializeSeed<'de, Value = Option<T>>,
//     ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = Option<T>>,
//     // ContextSeed<'a, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
// {
//     type Value = Option<U>;

//     #[inline]
//     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(formatter, "{}", List::<T>::NAME)
//     }
// }

impl<'de, 'a, C: Context, T: Representation, U: Representation> DeserializeSeed<'de>
    for ContextSeed<'a, C, List<T>, U>
where
    // ContextSeed<'a, C, T, T>: DeserializeSeed<'de, Value = Option<T>>,
    ContextSeed<'a, C, List<T>, U>: Visitor<'de, Value = Option<U>>,
{
    type Value = Option<U>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

// impl_ipld! { @select_self {
//     T: Representation + Send + Sync + 'static,
// } {} List<T> {
//     fn select<U: Select<C>>(
//         selector: &Selector,
//         mut state: SelectorState,
//         ctx: &mut C
//     ) -> Result<Option<U>, Error> {
//         match &selector {
//             Selector::Matcher { .. }
//             | Selector::ExploreAll { .. }
//             | Selector::ExploreIndex { .. }
//             | Selector::ExploreRange { .. } => {
//                 ContextSeed::<'_, C, Self, U>::from(selector, &mut state, ctx).decode()
//             }
//             // Some(_) if U::type_eq::<List<T>>() => {
//             //     let deserializer = ctx.path_decoder(&state.path())?;
//             //     let seed = ContextSeed::<'_, C, U, U>::from(selector, state, ctx);
//             //     U::r#match(seed.into::<U, U>(), deserializer).map_err(Error::decoder)
//             // }
//             _ => Err(Error::unsupported_selector::<Self, U>(&selector)),
//         }
//     }
// }}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

///
#[macro_export]
macro_rules! impl_ipld_list_old {
    (@visitor_self $concrete_ty:ty => $inner_ty:ty) => {
        // impl_ipld! { @visitor {} {} $concrete_ty => $concrete_ty {
        //     #[inline]
        //     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         write!(formatter, "{}", <$concrete_ty>::NAME)
        //     }
        //
        //     #[inline]
        //     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        //     where
        //         A: $crate::dev::SeqAccess<'de>,
        //     {
        //         self.into::<List<$inner_ty>, List<$inner_ty>>()
        //             .visit_seq(seq)
        //             .map(|opt| opt.map(Self))
        //     }
        // }}

        impl_ipld! { @visitor {} {} List<$inner_ty> => List<$inner_ty> {
            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", <$concrete_ty>::NAME)
            }

            #[inline]
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: $crate::dev::SeqAccess<'de>,
            {
                let Self {
                    selector, state, ..
                } = self;

                // match selector {
                //     Selector::Matcher(Matcher { label, .. }) => {
                //         state.add_matched(s.clone().into(), label.clone())
                //             .map_err(E::custom)?;

                //         Ok(Some(s))
                //     },
                //     selector => Err(Error::unsupported_selector::<String>(selector)).map_err(E::custom)
                // }

                unimplemented!()
            }
        }}
    };
    (@visitor_inner $concrete_ty:ty => $inner_ty:ty) => {
        // impl_ipld! { @visitor {} {} $concrete_ty => $inner_ty {
        //     #[inline]
        //     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         write!(formatter, "{}", <$concrete_ty>::NAME)
        //     }
        //
        //     #[inline]
        //     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        //     where
        //         A: $crate::dev::SeqAccess<'de>,
        //     {
        //         self.into::<List<$inner_ty>, $inner_ty>()
        //     }
        // }}

        impl_ipld! { @visitor {} {} List<$inner_ty> => $inner_ty {
            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", <$concrete_ty>::NAME)
            }

            #[inline]
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: $crate::dev::SeqAccess<'de>,
            {
                let Self {
                    selector, state, ..
                } = self;

                // match selector {
                //     Selector::Matcher(Matcher { label, .. }) => {
                //         state.add_matched(s.clone().into(), label.clone())
                //             .map_err(E::custom)?;

                //         Ok(Some(s))
                //     },
                //     selector => Err(Error::unsupported_selector::<String>(selector)).map_err(E::custom)
                // }

                unimplemented!()
            }
        }}
    };
    (@visitor_generic $concrete_ty:ty => $inner_ty:ty) => {
        // impl_ipld! { @visitor {} {} $conc// primitive_select::<'_, '_, C, T>(selector, state, ctx)

        // impl_ipld! { @visitor {} {} List<$inner_ty> => T {
        // default impl<'de, 'a, C: $crate::dev::Context, T: $crate::dev::Representation> $crate::dev::Visitor<'de> for ContextSeed<'a, C, List<$inner_ty>, T>
        // where
        //     $inner_ty: Select<C, T>,
        // {
        //     type Value = Option<T>;
        //
        //     #[inline]
        //     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         write!(formatter, "{}", <$concrete_ty>::NAME)
        //     }
        //
        //     #[inline]
        //     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        //     where
        //         A: $crate::dev::SeqAccess<'de>,
        //     {
        //         let Self {
        //             selector, state, ..
        //         } = self;
        //
        //         // match selector {
        //         //     Selector::Matcher(Matcher { label, .. }) => {
        //         //         state.add_matched(s.clone().into(), label.clone())
        //         //             .map_err(E::custom)?;
        //
        //         //         Ok(Some(s))
        //         //     },
        //         //     selector => Err(Error::unsupported_selector::<String>(selector)).map_err(E::custom)
        //         // }
        //
        //         unimplemented!()
        //     }
        // }

    };
    (@deseed $concrete_ty:ty => $inner_ty:ty) => {
        // impl_ipld! { @deseed {} {} $concrete_ty => $concrete_ty {
        //     #[inline]
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: $crate::dev::Deserializer<'de>,
        //     {
        //         deserializer.deserialize_seq(self.into::<List<$inner_ty>, List<$inner_ty>>())
        //             .map(|opt| opt.map(Self))
        //     }
        // }}

        // impl_ipld! { @deseed {} {} $concrete_ty => $inner_ty {
        //     #[inline]
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: $crate::dev::Deserializer<'de>,
        //     {
        //         deserializer.deserialize_seq(self.into::<List<$inner_ty>, $inner_ty>())
        //     }
        // }}

        // impl_ipld! { @deseed
        //     {T: Representation} {$inner_ty: Select<C, T>}
        //     $concrete_ty => T
        // {
        //     #[inline]
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: $crate::dev::Deserializer<'de>,
        //     {
        //         deserializer.deserialize_seq(self.into::<List<$inner_ty>, T>())
        //     }
        // }}

        // impl_ipld! { @deseed {} {} List<$inner_ty> => List<$inner_ty> {
        //     #[inline]
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: $crate::dev::Deserializer<'de>,
        //     {
        //         deserializer.deserialize_seq(self)
        //     }
        // }}

        // impl_ipld! { @deseed {} {} List<$inner_ty> => $inner_ty {
        //     #[inline]
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: $crate::dev::Deserializer<'de>,
        //     {
        //         deserializer.deserialize_seq(self)
        //     }
        // }}

        // impl_ipld! { @deseed
        //     {T: Representation} {$inner_ty: Select<C, T>}
        //     List<$inner_ty> => T
        // {
        // default impl<'de, 'a, C: $crate::dev::Context, T: $crate::dev::Representation> $crate::dev::DeserializeSeed<'de> for ContextSeed<'a, C, List<$inner_ty>, T>
        // where
        //     ContextSeed<'a, C, $inner_ty, T>: DeserializeSeed<'de>,
        //     // $inner_ty: Select<C, T>,
        //     // T: Select<C, T>,
        // {
        //     type Value = Option<T>;
        //
        //     #[inline]
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: $crate::dev::Deserializer<'de>,
        //     {
        //         deserializer.deserialize_seq(self)
        //     }
        // }
    };
    (@select $concrete_ty:ty => $inner_ty:ty) => {
        // main impl

        // impl_ipld! { @select {} {} $concrete_ty => $concrete_ty {
        //     #[inline]
        //     fn select(
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         ctx: &mut C,
        //     ) -> Result<Option<Self>, $crate::dev::Error> {
        //         <List<$inner_ty> as Select<C, List<$inner_ty>>>::select(selector, state, ctx)
        //             .map(|opt| opt.map(Self))
        //     }
        //
        //     #[inline]
        //     fn patch(
        //         self_: &mut Self,
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         dag: Self,
        //         ctx: &mut C,
        //     ) -> Result<(), $crate::dev::Error> {
        //         <List<$inner_ty> as Select<C, List<$inner_ty>>>::patch(&mut self_.0, selector, state, dag.0, ctx)
        //     }
        // }}
        //
        // impl_ipld! { @select {} {} $concrete_ty => $inner_ty {
        //     #[inline]
        //     fn select(
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         ctx: &mut C,
        //     ) -> Result<Option<$inner_ty>, $crate::dev::Error> {
        //         <$inner_ty as Select<C, List<$inner_ty>>>::select(selector, state, ctx)
        //     }
        //
        //     #[inline]
        //     fn patch(
        //         self_: &mut $concrete_ty,
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         dag: $inner_ty,
        //         ctx: &mut C,
        //     ) -> Result<(), $crate::dev::Error> {
        //         <$inner_ty as Select<C, List<$inner_ty>>>::patch(&mut self_.0, selector, state, dag, ctx)
        //     }
        // }}

        // impl<C: $crate::dev::Context, T: $crate::dev::Representation> $crate::dev::Select<C, T> for $concrete_ty
        // {
        //     #[inline]
        //     fn select(
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         ctx: &mut C,
        //     ) -> Result<Option<T>, $crate::dev::Error> {
        //         <List<$inner_ty> as Select<C, T>>::select(selector, state, ctx)
        //     }
        //
        //     #[inline]
        //     fn patch(
        //         &mut self,
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         dag: T,
        //         ctx: &mut C,
        //     ) -> Result<(), $crate::dev::Error> {
        //         <List<$inner_ty> as Select<C, T>>::patch(&mut self.0, selector, state, dag, ctx)
        //     }
        // }

        // list impl

        // impl_ipld! { @select {} {} List<$inner_ty> => List<$inner_ty> {
        //     #[inline]
        //     fn select(
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         ctx: &mut C,
        //     ) -> Result<Option<Self>, $crate::dev::Error> {
        //         // primitive_select::<'_, '_, C, Self>(selector, state, ctx)
        //         unimplemented!()
        //     }
        //
        //     fn patch(
        //         self_: &mut List<$inner_ty>,
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         dag: Self,
        //         ctx: &mut C,
        //     ) -> Result<(), $crate::dev::Error> {
        //         // primitive_patch::<C, List<$inner_ty>>(self, selector, state, dag, ctx)
        //         unimplemented!()
        //     }
        // }}
        //
        // impl_ipld! { @select {} {} List<$inner_ty> => $inner_ty {
        //     #[inline]
        //     fn select(
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         ctx: &mut C,
        //     ) -> Result<Option<$inner_ty>, $crate::dev::Error> {
        //         // primitive_select::<'_, '_, C, $inner_ty>(selector, state, ctx)
        //         unimplemented!()
        //     }
        //
        //     fn patch(
        //         self_: &mut List<$inner_ty>,
        //         selector: &$crate::dev::Selector,
        //         state: $crate::dev::SelectorState,
        //         dag: $inner_ty,
        //         ctx: &mut C,
        //     ) -> Result<(), $crate::dev::Error> {
        //         // primitive_patch::<C, List<$inner_ty>>(self, selector, state, dag, ctx)
        //         unimplemented!()
        //     }
        // }}

        impl<C: $crate::dev::Context, T: $crate::dev::Representation> $crate::dev::Select<C> for List<T>
        where
            // T: Select<C, T>,
            // T: Select<C, $inner_ty>,
            // C: 'a,
            // ContextSeed<'a, C, List<$inner_ty>, T>: for<'de> DeserializeSeed<'de, Value = Option<T>>,
        {
            #[inline]
            fn select<U: Select<C>>(
                selector: &$crate::dev::Selector,
                state: $crate::dev::SelectorState,
                ctx: &mut C,
            ) -> Result<U, $crate::dev::Error> {
                // let deserializer = ctx.path_decoder(&state.path())?;
                // ContextSeed::<'_, C, List<$inner_ty>, T>::from(selector, state, ctx)
                //     .deserialize(deserializer).map_err(Error::decoder)
            }

            // default fn patch(
            //     &mut self,
            //     selector: &$crate::dev::Selector,
            //     state: $crate::dev::SelectorState,
            //     dag: T,
            //     ctx: &mut C,
            // ) -> Result<(), $crate::dev::Error> {
            //     // primitive_patch::<C, List<$inner_ty>>(self, selector, state, dag, ctx)
            //     unimplemented!()
            // }
        }
    };
    ($concrete_ty:ty => $inner_ty:ty) => {
        impl_ipld_list!(@visitor_self $concrete_ty => $inner_ty);
        impl_ipld_list!(@visitor_inner $concrete_ty => $inner_ty);
        // impl_ipld_list!(@visitor_generic $concrete_ty => $inner_ty);
        impl_ipld_list!(@deseed $concrete_ty => $inner_ty);
        // impl_ipld_list!(@select $concrete_ty => $inner_ty);

    };
}

// mod impl_self {
//     use crate::dev::*;
//
//     impl_ipld! { @visitor {T: Representation} List<T> => List<T> {
//     // default impl<'de, 'a, C, T> Visitor<'de> for ContextSeed<'a, C, List<T>, List<T>>
//     // where
//     //     C: Context,
//     //     T: Representation,
//     // {
//     //     type Value = Option<List<T>>;
//         #[inline]
//         fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             write!(formatter, "A list of `{}`", T::NAME)
//         }
//
//         #[inline]
//         fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
//         where
//             A: SeqAccess<'de>,
//         {
//             let Self {
//                 selector, state, ..
//             } = self;
//
//             // match selector {
//             //     Selector::Matcher(Matcher { label, .. }) => {
//             //         state.add_matched(s.clone().into(), label.clone())
//             //             .map_err(E::custom)?;
//
//             //         Ok(Some(s))
//             //     },
//             //     selector => Err(Error::unsupported_selector::<String>(selector)).map_err(E::custom)
//             // }
//
//             unimplemented!()
//         }
//     }}
//
//     // impl_ipld! { @deseed {
//     //     T: Representation
//     // } List<T> => List<T> {
//     //     #[inline]
//     //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     //     where
//     //         D: Deserializer<'de>,
//     //     {
//     //         deserializer.deserialize_seq(self)
//     //     }
//     // }}
//     //
//     // impl_ipld! { @select {
//     //     T: Representation
//     // } List<T> => Self {
//     //     #[inline]
//     //     fn select(
//     //         selector: &Selector,
//     //         state: SelectorState,
//     //         ctx: &mut C,
//     //     ) -> Result<Option<Self>, Error> {
//     //         // primitive_select::<'_, '_, C, Self>(selector, state, ctx)
//     //         unimplemented!()
//     //     }
//     //
//     //     fn patch(
//     //         &mut self,
//     //         selector: &Selector,
//     //         state: SelectorState,
//     //         dag: Self,
//     //         ctx: &mut C,
//     //     ) -> Result<(), Error> {
//     //         // primitive_patch::<C, List<T>>(self, selector, state, dag, ctx)
//     //         unimplemented!()
//     //     }
//     // }}
// }
//
// mod impl_generic {
//     use crate::dev::*;
//
//     default impl<'de, 'a, C, T, U> Visitor<'de> for ContextSeed<'a, C, List<T>, U>
//     where
//         C: Context,
//         // List<T>: Select<C, T>,
//         T: Representation,
//         // ContextSeed<'a, C, T, U>: DeserializeSeed<'de, Value = ()>,
//         U: Representation,
//     {
//         type Value = ();
//
//         #[inline]
//         fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             write!(formatter, "A list of {}", T::NAME)
//         }
//
//         #[inline]
//         fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
//         where
//             A: SeqAccess<'de>,
//         {
//             let Self {
//                 selector, state, ..
//             } = self;
//
//             // match selector {
//             //     Selector::Matcher(Matcher { label, .. }) => {
//             //         state.add_matched(Node::List, label.clone())
//             //             .map_err(E::custom)?;
//
//             //         Ok(Some(s))
//             //     },
//             //     selector => Err(Error::unsupported_selector::<String>(selector)).map_err(E::custom)
//             // }
//
//             unimplemented!()
//         }
//     }
//
//     impl_ipld! { @deseed {
//         T: Representation,
//         // ContextSeed<'a, C, T, U>: serde::de::DeserializeSeed<'de, Value = ()>,
//         U: Representation,
//     } List<T> => U {
//         #[inline]
//         fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//         where
//             D: Deserializer<'de>,
//         {
//             deserializer.deserialize_seq(self)
//         }
//     }}
//
//     impl_ipld! { @select {
//         T: Representation,
//         // ContextSeed<'a, C, T, U>: serde::de::DeserializeSeed<'de, Value = ()>,
//         U: Representation,
//     } List<T> => U {
//         fn select(
//             selector: &Selector,
//             state: SelectorState,
//             ctx: &mut C,
//         ) -> Result<(), Error> {
//             let deserializer = ctx.path_decoder(state.path())?;
//             ContextSeed::<'_, C, T, U>::deserialize((selector, state, ctx).into(), deserializer)
//                 .map_err(Error::decoder)
//         }
//
//         fn patch(
//             &mut self,
//             selector: &Selector,
//             state: SelectorState,
//             dag: U,
//             ctx: &mut C,
//         ) -> Result<(), Error> {
//             // primitive_patch::<C, List<T>>(self, selector, state, dag, ctx)
//             unimplemented!()
//         }
//     }}
// }
//
// // exploring all, range, index
// impl<C, T> Select<C, Self> for List<T>
// where
//     T: Select<C, T>,
//     C: Context,
// {
//     fn select(seed: SelectionProgress, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
//
//     // fn select_dag(seed: SelectionProgress, ctx: &mut C) -> Result<Self, Error> {
//     //     unimplemented!()
//     // }
//
//     fn patch(&mut self, seed: SelectionProgress, dag: Self, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
// }
//
// // exploring index
// // impl<T, U, C> Select<U, C> for List<T>
// // where
// //     T: Select<U, C>,
// //     U: Select<U, C>,
// //     C: Context,
// impl<C, T> Select<C, T> for List<T>
// where
//     T: Select<C, T>,
//     C: Context,
// {
//     fn select(seed: SelectionProgress, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
//
//     fn select_dag(seed: SelectionProgress, ctx: &mut C) -> Result<T, Error> {
//         unimplemented!()
//     }
//
//     fn patch(&mut self, seed: SelectionProgress, dag: T, ctx: &mut C) -> Result<(), Error> {
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
// struct SelectorListVisitor<T> {
//     pub seed: SelectorSeed,
//     pub start: Option<Int>,
//     pub end: Option<Int>,
//     pub next: Selector,
//     _type: PhantomData<T>,
// }
//
// async fn select_list<'de, S, C, T>(
//     selector: S,
//     ctx: &mut C,
//     visitor: SelectorListVisitor<T>,
// ) -> Result<SelectorSeed, Error>
// where
//     S: Into<SelectorSeed>,
//     C: Context,
//     T: Select<C>,
//     List<T>: Select<C>,
// {
//     impl<'de, T> Visitor<'de> for SelectorListVisitor<T>
//     where
//         T: Representation,
//         List<T>: Representation,
//     {
//         type Value = SelectorSeed;
//
//         fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             formatter.write_str("an IPLD Data Model List")
//         }
//
//         //     fn visit_seq<S>(mut self, mut seq: S) -> Result<Self::Value, S::Error>
//         //     where
//         //         S: SeqAccess<'de>,
//         //     {
//         //         // ignore starting elements
//         //         let start = self.start.unwrap_or(0);
//         //         for _ in 0..start {
//         //             if seq.next_element::<IgnoredAny>()?.is_none() {
//         //                 return Err(S::Error::custom(Error::SelectorRange(
//         //                     "list contains too few elements",
//         //                 )));
//         //             }
//         //         }
//
//         //         // select within target index/range
//         //         if let Some(end) = self.end {
//         //             for idx in start..end {
//         //                 self.seed = deserialize_selector_list_element(self.seed, seq, idx, self.next)?
//         //                     .ok_or(S::Error::custom(Error::SelectorRange(
//         //                         "missing expected element",
//         //                     )))?;
//         //             }
//         //         } else {
//         //             for idx in start.. {
//         //                 if let Some(seed) =
//         //                     deserialize_selector_list_element(self.seed, seq, idx, self.next)?
//         //                 {
//         //                     self.seed = seed;
//         //                 }
//         //             }
//         //         }
//
//         //         // ignore any remaining elements
//         //         while let Some(_) = seq.next_element::<IgnoredAny>()? {}
//
//         //         Ok(self.seed)
//         //     }
//     }
//
//     unimplemented!()
// }

// fn deserialize_selector_list_element<'de, S, T>(
//     seed: SelectorSeed,
//     mut seq: S,
//     idx: Int,
//     next: Selector,
// ) -> Result<Option<SelectorSeed>, S::Error>
// where
//     S: SeqAccess<'de>,
//     T: Representation,
//     List<T>: Representation,
// {
//     // let new_seed = seed
//     //     .descend::<T, _>(next, idx.to_string(), false)
//     //     .map_err(S::Error::custom)?;
//     // Ok(seq
//     //     .next_element_seed(new_seed)?
//     //     .map(|seed| new_seed.ascend(seed.selector, false)))
//
//     unimplemented!()
// }

// // TODO: add the rest of the selectors
// impl_root_select!(Matcher, ExploreAll, ExploreIndex, ExploreRange, {
//     default impl<Ctx, T> Select<Selector, Ctx> for Vec<T>
//     where
//         Ctx: Context,
//         T: Representation + 'static
// });
