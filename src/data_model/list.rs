use crate::dev::*;
use macros::impl_selector_seed_serde;
use maybestd::{
    cell::RefCell,
    fmt, iter,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};
use serde::de::value::SeqAccessDeserializer;

pub use iterators::*;

///
pub type List<T = Any> = Vec<T>;

impl<T: Representation> Representation for List<T> {
    const NAME: &'static str = "List";
    const SCHEMA: &'static str = concat!("type List [", stringify!(T::NAME), "]");
    const DATA_MODEL_KIND: Kind = Kind::List;
    const SCHEMA_KIND: Kind = Kind::List;
    const REPR_KIND: Kind = Kind::List;
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
        impl<'de, const C: u64, T> Visitor<'de> for ListVisitor<C, T>
        where
            T: Representation,
        {
            type Value = List<T>;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A list of `{}`", T::NAME)
            }
            #[inline]
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut list = List::with_capacity(seq.size_hint().unwrap_or(8));
                let mut iter = SerdeListIterator::<'de, A>::from(seq);
                while let Some(dag) = iter.next::<C>().map_err(A::Error::custom)? {
                    list.push(dag);
                }
                Ok(list)
            }
        }

        deserializer.deserialize_seq(ListVisitor::<C, T>(PhantomData))
    }
}

impl_selector_seed_serde! { @codec_seed_visitor
    { T: Select<Ctx> + 'static } {} List<T>
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
        if let Some(s) = self.0.selector.as_explore_union() {
            if s.matches_first() {
                let list = <List<T>>::deserialize::<_C, _>(SeqAccessDeserializer::new(seq))?;
                return list.__select_in(self.0).map_err(A::Error::custom);
            } else {
                // todo: support multiple non-overlapping ranges
            }
        }

        let iter = SerdeListIterator::<'de, _>::from(seq);
        match self.0.selector {
            Selector::Matcher(_) => {
                self.0.match_list::<_C, T, _, _, _, _, _>(
                    iter,
                    |iter| {
                        let len = <_ as ListIterator<T>>::size_hint(iter).unwrap_or(8);
                        RefCell::new(List::<T>::with_capacity(len))
                    },
                    |dag| Box::new(|child, _| Ok(dag.borrow_mut().push(child))),
                    RefCell::into_inner,
                ).map_err(A::Error::custom)
            },
            Selector::ExploreIndex(s) => self.0
                .explore_list_range::<_C, T, _, _>(iter, s.to_range())
                .map_err(A::Error::custom),
            Selector::ExploreRange(s) => self.0
                .explore_list_range::<_C, T, _, _>(iter, s.to_range())
                .map_err(A::Error::custom),
            Selector::ExploreAll(s) => self.0
                .explore_list_range::<_C, T, _, _>(iter, s.to_range())
                .map_err(A::Error::custom),
            _ => Err(A::Error::custom(Error::unsupported_selector::<List<T>>(
                self.0.selector,
            ))),
        }
    }
}}

impl_selector_seed_serde! { @codec_seed_visitor_ext
    { T: Select<Ctx> + 'static } {} List<T> {}
}

// impl_selector_seed_serde! { @selector_seed_codec_deseed
//     { T: Select<Ctx> + 'static } {} List<T>
// {
//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_seq(self)
//     }
// }}

impl_selector_seed_serde! { @selector_seed_select
    { T: Select<Ctx> + 'static } {} List<T>
}

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Select<Ctx> + 'static,
{
    /// match
    /// todo could probably use a collection trait instead of multiple Fns
    pub fn match_list<const C: u64, U, I, T2, F1, F2, F3>(
        mut self,
        mut iter: I,
        init_new_dag: F1,
        mut match_cb: F2,
        into_dag: F3,
    ) -> Result<(), Error>
    where
        U: Select<Ctx>,
        I: ListIterator<U>,
        T2: Default,
        F1: FnOnce(&I) -> T2,
        for<'b> F2: FnMut(&'b T2) -> Box<dyn MatchDagOp<U, Ctx> + 'b>,
        F3: FnOnce(T2) -> T,
    {
        // select the matched list node, or setup the list
        self.select_node(SelectedNode::List)?;
        let new_dag = self
            .is_dag_select()
            .then(|| init_new_dag(&iter))
            .unwrap_or_default();

        // select against each child
        for idx in 0usize.. {
            // TODO: should be iter.next_element_seed(self.to_field_select_seed())
            if self.select_index::<C, U>(
                idx,
                self.is_dag_select().then(|| match_cb(&new_dag)),
                &mut iter,
            )? {
                break;
            }
        }

        // finally, select the dag itself (if applicable)
        if self.is_dag_select() {
            self.select_dag(into_dag(new_dag))?;
        }

        Ok(())
    }

    /// explore index, range, or all
    pub fn explore_list_range<const C: u64, U, I, R>(
        mut self,
        mut iter: I,
        range: R,
    ) -> Result<(), Error>
    where
        U: Select<Ctx>,
        I: ListIterator<U>,
        R: RangeBounds<usize> + Iterator<Item = usize>,
    {
        let range_copy = (range.start_bound().cloned(), range.end_bound().cloned());

        let (start_idx, ignore_end_idx) = match range_copy {
            (Bound::Included(start), Bound::Unbounded) => (start, None),
            (Bound::Included(start), Bound::Included(i)) => (start, Some(i + 1)),
            (Bound::Included(start), Bound::Excluded(i)) => (start, Some(i)),
            _ => unreachable!(),
        };
        let is_unbounded = ignore_end_idx.is_none();

        // select the list node
        self.select_node(SelectedNode::List)?;

        // ignore everything before the start (unless 0)
        // if empty, return an err
        if start_idx > 0 {
            for index in 0usize..start_idx {
                if iter.next_ignored()? {
                    return Err(Error::explore_list_failure::<U>(self.selector, index));
                }
            }
        }

        // explore any/all indices in the range
        for index in range {
            // TODO: should be iter.next_element_seed(self.to_field_select_seed())
            let is_empty = self.select_index::<C, U>(index, None, &mut iter)?;

            // if unbounded and empty, then we're done exploring
            // if bounded and empty, then we failed to explore everything
            if is_unbounded && is_empty {
                return Ok(());
            } else if is_empty && range_copy.contains(&index) {
                return Err(Error::explore_list_failure::<U>(&self.selector, index));
            }
        }

        // finish ignoring the remainder of the list
        if !is_unbounded {
            for _ in ignore_end_idx.unwrap().. {
                if iter.next_ignored()? {
                    break;
                }
            }
        }

        Ok(())
    }
}

mod iterators {
    use super::*;

    /// A [`Select`]able list iterator over a serde sequence representation.
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct SerdeListIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        inner: A,
        index: usize,
        _t: PhantomData<&'de ()>,
    }

    impl<'de, A> From<A> for SerdeListIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        fn from(inner: A) -> Self {
            Self {
                inner,
                index: 0,
                _t: PhantomData,
            }
        }
    }

    impl<'de, T, A> ListIterator<T> for SerdeListIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.inner.size_hint()
        }

        fn field(&self) -> Field<'_> {
            Field::Index(self.index)
        }

        fn next_ignored(&mut self) -> Result<bool, Error> {
            let is_empty = self
                .inner
                .next_element::<IgnoredAny>()
                .map_err(|_| Error::explore_index_failure::<IgnoredAny>(self.index))?
                .is_none();
            if !is_empty {
                self.index += 1;
            }
            Ok(is_empty)
        }

        fn next<const C: u64>(&mut self) -> Result<Option<T>, Error>
        where
            T: Representation,
        {
            let dag = self
                .inner
                .next_element_seed(DeserializeWrapper::<C, T>::default())
                .map_err(|_| Error::explore_index_failure::<T>(self.index))?;

            if dag.is_none() {
                Ok(None)
            } else {
                self.index += 1;
                Ok(dag)
            }
        }

        fn next_seed<'a, const C: u64, Ctx: Context>(
            &mut self,
            seed: SelectorSeed<'a, Ctx, T>,
        ) -> Result<bool, Error>
        where
            T: Select<Ctx>,
        {
            let is_empty = T::__select_seq::<C, _>(seed, &mut self.inner)
                .map_err(|_| Error::explore_index_failure::<T>(self.index))?
                .is_none();
            if !is_empty {
                self.index += 1;
            }
            Ok(is_empty)
        }
    }

    /// A [`Select`]able list iterator over an underlying iterator.
    pub struct MemoryListIterator<'a, T, I> {
        iter: I,
        index: usize,
        len: usize,
        _t: PhantomData<&'a T>,
    }

    impl<'a, T, I> ListIterator<T> for MemoryListIterator<'a, T, I>
    where
        T: Representation,
        I: Iterator<Item = &'a T>,
    {
        fn field(&self) -> Field<'_> {
            Field::Index(self.index)
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.len)
        }

        //
        fn next_ignored(&mut self) -> Result<bool, Error> {
            self.iter
                .next()
                .ok_or_else(|| Error::explore_index_failure::<T>(self.index))?;
            self.index += 1;
            Ok(self.len == self.index)
        }

        fn next_seed<'_a, const C: u64, Ctx: Context>(
            &mut self,
            seed: SelectorSeed<'_a, Ctx, T>,
        ) -> Result<bool, Error>
        where
            T: Select<Ctx>,
        {
            self.iter
                .next()
                .ok_or_else(|| Error::explore_index_failure::<T>(self.index))?
                .__select_in(seed)?;
            self.index += 1;
            Ok(self.len == self.index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_match() {}
}
