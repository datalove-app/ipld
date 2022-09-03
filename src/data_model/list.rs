use crate::dev::*;
use std::{
    cell::RefCell,
    ops::{Bound, RangeBounds},
};

///
pub type List<T = Any> = Vec<T>;

impl<T: Representation> Representation for List<T> {
    const NAME: &'static str = concat!("List<", stringify!(T::NAME), ">");
    const SCHEMA: &'static str = concat!(
        "type ",
        stringify!(Self::NAME),
        " [",
        stringify!(T::NAME),
        "]",
    );
    const DATA_MODEL_KIND: Kind = Kind::List;
    const HAS_LINKS: bool = T::HAS_LINKS;

    fn has_links(&self) -> bool {
        self.iter().any(|e| e.has_links())
    }
}

impl_ipld_serde! { @context_visitor
    { T: Representation + 'static }
    {
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
        // ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
    }
    List<T>
{
    #[inline]
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <List<T>>::NAME)
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
}}

impl_ipld_serde! { @context_deseed
    { T: Representation + 'static }
    {
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
        // ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
    }
    List<T>
{
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}}

impl_ipld_serde! { @context_select
    { T: Representation + Send + Sync + 'static }
    {
        for<'b, 'de> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>
    }
    List<T>
}

/*
impl<'a, 'de, C, T> Visitor<'de> for ContextSeed<'a, C, List<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
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

impl<'a, 'de, C, T> DeserializeSeed<'de> for ContextSeed<'a, C, List<T>>
where
    C: Context,
    T: Representation + 'static,
    for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
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

// TODO replace with impl_ipld_serde
impl<'a, C, T> Select<C> for List<T>
where
    C: Context,
    T: Representation + Send + Sync + 'static,
{
    fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
        unimplemented!()
    }
}
 */

// match impl
impl<'a, C, T> ContextSeed<'a, C, List<T>>
where
    C: Context,
    T: Representation + 'static,
{
    /// match
    fn match_list<'de, A>(mut self, matcher: &Matcher, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
        // ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
    {
        let mode = self.mode();
        let mut dag: RefCell<List<T>> = Default::default();

        // select list node, or set up list
        match mode {
            // select the list node
            SelectionMode::SelectNode => {
                self.select_matched_node(SelectedNode::List, matcher.label.as_deref())
                    .map_err(A::Error::custom)?;
            }
            SelectionMode::SelectDag => {
                // set up the dag
                *dag.get_mut() = List::<T>::with_capacity(seq.size_hint().unwrap_or(8));
            }
            _ => unimplemented!(),
        }

        let (selector, state, mut params, ctx) = self.into_parts();

        // select against each child
        for index in 0usize.. {
            let is_empty = Self::field_select_seed::<T>(
                selector,
                state,
                &mut params,
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
            .map_err(A::Error::custom)
            .and_then(|seed| Ok(seq.next_element_seed(seed)?.is_none()))?;
            state.ascend::<T>().map_err(A::Error::custom)?;

            if is_empty {
                break;
            }
        }

        // finally, select the matched dag
        if mode == SelectionMode::SelectDag {
            let mut original_seed = Self::from(selector, state, params, ctx);
            original_seed
                .select_matched_dag(dag.into_inner(), matcher.label.as_deref())
                .map_err(A::Error::custom)?;
        }

        Ok(())
    }

    /// explore index, range, or all
    fn explore_list<'de, A, R>(mut self, range: R, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        R: RangeBounds<usize> + Iterator<Item = usize>,
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
        // ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
    {
        // select the list node
        if self.is_node() {
            self.select_node(SelectedNode::List)
                .map_err(A::Error::custom)?;
        }

        let is_unbounded = range.end_bound() == Bound::Unbounded;
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            _ => unreachable!(),
        };

        // ignore everything before the start (unless 0)
        // if empty, return an err
        let (selector, state, mut params, ctx) = self.into_parts();
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
            let is_empty = Self::field_select_seed::<T>(
                &selector,
                state,
                &mut params,
                ctx,
                index.into(),
                None,
            )
            .map_err(A::Error::custom)
            .and_then(|seed| Ok(seq.next_element_seed(seed)?.is_none()))?;
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
impl<'a, C, T> ContextSeed<'a, C, List<T>>
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
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
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
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
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
        for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
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
