use crate::dev::*;
use macros::impl_selector_seed_serde;
use serde::de::value::SeqAccessDeserializer;
use std::{
    cell::RefCell,
    fmt,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

///
pub type List<T = Any> = Vec<T>;

impl<T: Representation> Representation for List<T> {
    const NAME: &'static str = "List";
    const SCHEMA: &'static str = concat!("type List [", stringify!(T::NAME), "]");
    const DATA_MODEL_KIND: Kind = Kind::List;
    const HAS_LINKS: bool = T::HAS_LINKS;

    fn has_links(&self) -> bool {
        self.iter().any(Representation::has_links)
    }

    #[inline]
    #[doc(hidden)]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for elem in self {
            seq.serialize_element(&SerializeWrapper::<'_, C, _>(elem))?;
        }
        seq.end()
    }

    #[inline]
    #[doc(hidden)]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ListVisitor<const C: u64, T>(PhantomData<T>);
        impl<const C: u64, T> Default for ListVisitor<C, T> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }
        impl<'de, const C: u64, T: Representation> Visitor<'de> for ListVisitor<C, T> {
            type Value = List<T>;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A list of `{}`", T::NAME)
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut list = List::with_capacity(seq.size_hint().unwrap_or(8));
                while let Some(elem) =
                    seq.next_element_seed(DeserializeWrapper::<C, T>::default())?
                {
                    list.push(elem);
                }
                Ok(list)
            }
        }

        deserializer.deserialize_seq(ListVisitor::<C, T>::default())
    }
}

impl_selector_seed_serde! { @codec_seed_visitor
    // { T: Representation + 'static }
    { T: Select<Ctx> + 'static } { }
    // { for<'b> CodecSeed<_C, _D, SelectorSeed<'b, Ctx, T>, T>: DeserializeSeed<'de> }
    // { for<'b> CodedSelectorSeed<'b, _C, _D, Ctx, T>: DeserializeSeed<'de> }
    List<T>
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A list of `{}`", T::NAME)
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if Self::D {
            unimplemented!()
        } else {
            match self.0.selector {
                Selector::Matcher(_) => self.match_list(seq),
                Selector::ExploreIndex(s) => self.explore_list_range(s.index as usize..s.index as usize, seq),
                Selector::ExploreRange(s) => self.explore_list_range(s.start as usize..s.end as usize, seq),
                Selector::ExploreAll(_) => self.explore_list_range(0.., seq),
                _ => Err(A::Error::custom(Error::unsupported_selector::<List<T>>(
                    self.0.selector,
                ))),
            }
        }
    }
}}

impl_selector_seed_serde! { @codec_seed_visitor_ext
    // { T: Representation + 'static }
    { T: Select<Ctx> + 'static } { }
    // { for<'b> CodecSeed<_C, _D, SelectorSeed<'b, Ctx, T>, T>: DeserializeSeed<'de> }
    // { for<'b> CodedSelectorSeed<'b, _C, _D, Ctx, T>: DeserializeSeed<'de> }
    List<T> {}
}

impl_selector_seed_serde! { @selector_seed_codec_deseed
    // { T: Representation + 'static }
    { T: Select<Ctx> + 'static } { }
    // { for<'b> SelectorSeed<'b, Ctx, T>: CodecDeserializeSeed<'de, Value = ()> }
    // { for<'b> CodedSelectorSeed<'b, _C, _D, Ctx, T>: DeserializeSeed<'de> }
    List<T>
{
    // #[inline]
    // fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<(), D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     deserializer.deserialize_seq(CodecSeed::<C, false, _, _>::from(self))
    // }
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}}

impl_selector_seed_serde! { @selector_seed_select
    // { T: Representation + 'static } // + Select<Ctx>
    { T: Select<Ctx> + 'static } { }
    // { for<'b, 'de> SelectorSeed<'b, Ctx, T>: CodecDeserializeSeed<'de> }
    // { for<'b, 'de> CodedSelectorSeed<'b, _C, _D, Ctx, T>: DeserializeSeed<'de> }
    List<T>
}

/*
impl<'a, 'de, Ctx, T> Visitor<'de> for ContextSeed<'a, C, List<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, Ctx, T>: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", <List<T>>::NAME)
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        match self.selector {
            Selector::Matcher(matcher) => self.match_list(matcher, seq),
            Selector::ExploreIndex(s) => self.explore_list(s.index as usize..s.index as usize, seq),
            Selector::ExploreRange(s) => self.explore_list(s.start as usize..s.end as usize, seq),
            Selector::ExploreAll(s) => self.explore_list(0.., seq),
            _ => Err(A::Error::custom(Error::unsupported_selector::<List<T>>(
                self.selector,
            ))),
        }
    }
}

impl<'a, 'de, Ctx, T> DeserializeSeed<'de> for ContextSeed<'a, C, List<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, Ctx, T>: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

// TODO replace with impl_selector_seed_serde
impl<'a, Ctx, T> Select<C> for List<T>
where
    C: Context,
    T: Representation + Send + Sync + 'static,
{
    fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
        unimplemented!()
    }
}
 */

impl<'a, const C: u64, const D: bool, Ctx, T>
    CodecSeed<C, D, SelectorSeed<'a, Ctx, List<T>>, List<T>>
where
    Ctx: Context,
    // T: Representation + 'static,
    T: Select<Ctx> + 'static,
    // for<'b> CodecSeed<C, D, SelectorSeed<>>
{
    /// match
    fn match_list<'de, A>(mut self, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        // CodecSeed<C, D, Self>: DeserializeSeed<'de>,
        // for<'b> CodecSeed<C, D, SelectorSeed<'b, Ctx, T>, T>: DeserializeSeed<'de>,
    {
        let matcher = self
            .0
            .selector
            .as_matcher()
            .expect("should know that this is a matcher");

        // select list node, or set up list
        let mode = self.0.mode();
        let mut dag: RefCell<List<T>> = Default::default();

        match mode {
            SelectionMode::SelectNode => {
                self.0
                    .select_matched_node(SelectedNode::List, matcher.label.as_deref())
                    .map_err(A::Error::custom)?;
            }
            SelectionMode::SelectDag => {
                // set up the dag
                *dag.get_mut() = List::<T>::with_capacity(seq.size_hint().unwrap_or(8));
            }
            _ => unimplemented!(),
        }

        let (selector, state, mut cb, ctx) = self.0.into_parts();

        // select against each child
        for index in 0usize.. {
            let seed = SelectorSeed::field_select_seed::<T>(
                selector,
                state,
                &mut cb,
                ctx,
                index.into(),
                match mode {
                    SelectionMode::SelectNode => None,
                    SelectionMode::SelectDag => {
                        Some(Box::new(|child, _| Ok(dag.borrow_mut().push(child))))
                    }
                    _ => unreachable!(),
                },
            )
            .map_err(A::Error::custom)?;

            // TODO call T::select_from(seed, SeqAccessDeserializer(&mut seq))
            // let is_empty = seq.next_element_seed(CodecSeed::from(seed))?.is_none();
            let is_empty = T::__select_from_seq::<C, _>(seed, &mut seq)?.is_none();
            // let de = SeqAccessDeserializer::new(std::iter::from_fn(move || {
            //     seq.next_element_seed
            // }))
            state.ascend::<T>().map_err(A::Error::custom)?;

            if is_empty {
                break;
            }
        }

        // finally, select the matched dag
        if mode == SelectionMode::SelectDag {
            let mut original_seed = SelectorSeed::from_parts(selector, state, cb, ctx);
            original_seed
                .select_matched_dag(dag.into_inner(), matcher.label.as_deref())
                .map_err(A::Error::custom)?;
        }

        Ok(())
    }

    /// explore index, range, or all
    fn explore_list_range<'de, A, R>(mut self, range: R, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        R: RangeBounds<usize> + Iterator<Item = usize>,
        // for<'b> CodecSeed<C, D, SelectorSeed<'b, Ctx, T>, T>: DeserializeSeed<'de>,
    {
        // select the list node
        if self.0.is_node() {
            self.0
                .select_node(SelectedNode::List)
                .map_err(A::Error::custom)?;
        }

        let is_unbounded = range.end_bound() == Bound::Unbounded;
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            _ => unreachable!(),
        };

        // ignore everything before the start (unless 0)
        // if empty, return an err
        let (selector, state, mut cb, ctx) = self.0.into_parts();
        if start > 0 {
            for index in 0usize..start {
                if seq.next_element::<IgnoredAny>()?.is_none() {
                    return Err(A::Error::custom(Error::explore_list_failure(
                        selector, index,
                    )));
                }
            }
        }

        // explore any/all indices in the range
        for index in range {
            let seed = SelectorSeed::field_select_seed::<T>(
                &selector,
                state,
                &mut cb,
                ctx,
                index.into(),
                None,
            )
            .map_err(A::Error::custom)?;

            let is_empty = T::__select_from_seq::<C, _>(seed, &mut seq)?.is_none();
            // .and_then(|seed| Ok(seq.next_element_seed(CodecSeed::from(seed))?.is_none()))?;
            state.ascend::<T>().map_err(A::Error::custom)?;

            // if unbounded and empty, then we're done exploring
            // if bounded and empty, then we failed to explore everything
            if is_unbounded && is_empty {
                return Ok(());
            } else if is_empty {
                return Err(A::Error::custom(Error::explore_list_failure(
                    &selector, index,
                )));
            }
        }

        // finish ignoring the remainder of the list
        for _ in 0usize.. {
            if seq.next_element::<IgnoredAny>()?.is_none() {
                break;
            }
        }

        Ok(())
    }
}

/*
impl<'a, Ctx, T> ContextSeed<'a, C, List<T>>
where
    C: Context,
    T: Representation + Send + Sync + 'static,
{
    /// exploreindex
    fn visit_list_index<'de, A>(
        mut self,
        explore_index: &ExploreIndex,
        mut seq: A,
    ) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, Ctx, T>: DeserializeSeed<'de, Value = ()>,
    {
        // select the list node
        if self.params.is_node() {
            self.select_node(Node::List).map_err(A::Error::custom)?;
        }

        // ignore elements until index
        let target_index = explore_index.index as usize;
        for _ in 0usize..target_index {
            if let None = seq.next_element::<IgnoredAny>()? {
                return Err(A::Error::custom(Error::ExploreIndexFailure(
                    "too few nodes to explore index",
                    explore_index.index as usize,
                )));
            }
        }

        // create the child's seed, then select the index
        let (_, state, mut params, ctx) = self.into_parts();
        Self::select_at_index_seed::<T>(&explore_index.next, state, &mut params, ctx, target_index)
            .map_err(A::Error::custom)
            .and_then(|seed| seq.next_element_seed(seed))?;
        state.ascend::<T>().map_err(A::Error::custom)?;

        // finish ignoring the remaining elements
        for _ in 0usize.. {
            if seq.next_element::<IgnoredAny>()?.is_none() {
                break;
            }
        }

        Ok(())
    }

    /// explorerange
    fn visit_list_range<'de, A>(
        mut self,
        explore_range: &ExploreRange,
        mut seq: A,
    ) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, Ctx, T>: DeserializeSeed<'de, Value = ()>,
    {
        // select the list node
        if self.params.is_node() {
            self.select_node(Node::List).map_err(A::Error::custom)?;
        }

        // ignore elements until start
        let start = explore_range.start as usize;
        let end = explore_range.end as usize;
        for _ in 0usize..start {
            if let None = seq.next_element::<IgnoredAny>()? {
                return Err(A::Error::custom(Error::ExploreRangeFailure(
                    "too few nodes to explore range",
                    explore_range.start as usize,
                    explore_range.end as usize,
                )));
            }
        }

        // select the range
        let (_, state, mut params, ctx) = self.into_parts();
        for index in start..end {
            Self::select_at_index_seed::<T>(&explore_range.next, state, &mut params, ctx, index)
                .map_err(A::Error::custom)
                .and_then(|seed| seq.next_element_seed(seed))?;
            state.ascend::<T>().map_err(A::Error::custom)?;
        }

        // finish ignoring the remaining elements
        for _ in 0usize.. {
            if seq.next_element::<IgnoredAny>()?.is_none() {
                break;
            }
        }

        Ok(())
    }

    /// exploreall
    fn visit_list_all<'de, A>(
        mut self,
        explore_all: &ExploreAll,
        mut seq: A,
    ) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, Ctx, T>: DeserializeSeed<'de, Value = ()>,
    {
        // select the list node
        if self.params.is_node() {
            self.select_node(Node::List).map_err(A::Error::custom)?;
        }

        // select the range
        let (_, state, mut params, ctx) = self.into_parts();
        for index in 0usize.. {
            let is_empty =
                Self::select_at_index_seed::<T>(&explore_all.next, state, &mut params, ctx, index)
                    .map_err(A::Error::custom)
                    .and_then(|seed| Ok(seq.next_element_seed(seed)?.is_none()))?;
            state.ascend::<T>().map_err(A::Error::custom)?;

            if is_empty {
                break;
            }
        }

        Ok(())
    }
}
 */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_match() {}
}
