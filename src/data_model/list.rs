use crate::dev::*;
use macros::repr_serde;
use maybestd::{
    cell::RefCell,
    fmt, iter,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    vec::Vec,
};

pub use iterators::*;

/// A list type, implemented as a [`Vec`].
///
/// [`Vec`]: crate::maybestd::vec::Vec
pub type List<T = Any> = Vec<T>;

impl<T: Representation> Representation for List<T> {
    const NAME: &'static str = "List";
    const SCHEMA: &'static str = "type List [Any]";
    const DATA_MODEL_KIND: Kind = Kind::List;
    const HAS_LINKS: bool = T::HAS_LINKS;

    fn has_links(&self) -> bool {
        self.iter().any(Representation::has_links)
    }

    fn to_selected_node(&self) -> SelectedNode {
        SelectedNode::List
    }

    #[inline]
    #[doc(hidden)]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // #[cfg(feature = "dag-rkyv")]

        use ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for elem in self {
            seq.serialize_element(&SerializeRepr::<'_, C, _>(elem))?;
        }
        seq.end()
    }

    #[inline]
    #[doc(hidden)]
    fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ListVisitor<const MC: u64, T>(PhantomData<T>);
        impl<'de, const MC: u64, T> Visitor<'de> for ListVisitor<MC, T>
        where
            T: Representation,
        {
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
                // let mut iter = SerdeListIterator::<'de, A>::from(seq);
                // while let Some(dag) = iter.next::<MC>().map_err(A::Error::custom)? {
                //     list.push(dag);
                // }

                while let Some(dag) = seq.next_element_seed(DeserializeRepr::<MC, T>::new())? {
                    list.push(dag);
                }
                Ok(list)
            }
        }

        deserializer.deserialize_seq(ListVisitor::<MC, T>(PhantomData))
    }
}

repr_serde! { @select for List<T> { T } { T: Select<Ctx> + 'static }}
repr_serde! { @visitors for List<T> { T } { T: Select<Ctx> + 'static } @serde {
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A list of type {} of {}", <List<T>>::NAME, T::NAME)
    }
    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        use de::value::SeqAccessDeserializer as De;

        match (self.mode(), self.selector()) {
            (SelectionMode::Decode, _) => {
                let list = <List<T>>::deserialize::<MC, _>(De::new(seq))?;
                Ok(AstResult::Value(list))
                // self.into_inner()
                //     .match_list::<MC, T, _>(SerdeListIterator::from(seq))
                //     .map_err(A::Error::custom)

            }
            // (SelectionMode::Patch, _) => {}
            (_, Selector::ExploreUnion(s)) if s.matches_first() => {
                let list = <List<T>>::deserialize::<MC, _>(De::new(seq))?;
                // todo: for each selector, List<T>::select
                // list.__select_in(self.into_inner()).map_err(A::Error::custom)?;
                Ok(AstResult::Ok)
            }
            _ => {
                self.into_inner()
                    .select_list::<MC, false, T, _>(SeqAccessIterator::from(seq))
                    .map_err(A::Error::custom)
            }
        }
    }
}}

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    // T: Representation<ReprKind = type_kinds::List>,
    T: Select<Ctx> + 'static,
{
    ///
    pub fn select_list<'de, const MC: u64, const IN: bool, U, I>(
        mut self,
        mut iter: I,
        // dag: Either<&T2, I>,
    ) -> Result<AstResult<T>, Error>
    where
        'a: 'de,
        Ctx: 'de,
        U: Select<Ctx> + 'a,
        I: ListIterator<'de, U>,
    {
        match self.selector {
            Selector::Matcher(_) => self.match_list::<MC, U, _>(iter),
            Selector::ExploreIndex(s) => self.explore_list_range::<MC, U, _, _>(iter, s.to_range()),
            Selector::ExploreRange(s) => self.explore_list_range::<MC, U, _, _>(iter, s.to_range()),
            Selector::ExploreAll(s) => self.explore_list_range::<MC, U, _, _>(iter, s.to_range()),
            s => Err(Error::unsupported_selector::<T>(s)),
        }

        // self.cover_node(SelectedNode::List)?;
        // loop {
        //     if self.handle_index::<MC, U>(&mut iter, None)? {
        //         break;
        //     }
        // }

        // Ok(AstResult::Ok)
    }

    /// Executes a [`Matcher`] selector against a list (data model) type.
    /// todo could probably use a collection trait instead of multiple Fns
    pub fn match_list<'de, const MC: u64, U, I>(
        mut self,
        mut iter: I,
        // init_new_dag: F1,
        // mut match_cb: F2,
        // into_dag: F3,
    ) -> Result<AstResult<T>, Error>
    where
        'a: 'de,
        Ctx: 'de,
        U: Select<Ctx> + 'a,
        I: ListIterator<'de, U>,
        // T2: Default,
        // F1: FnOnce(&I) -> T2,
        // for<'b> F2: FnMut(&'b T2) -> Box<dyn MatchDagOp<U, Ctx> + 'b>,
        // F3: FnOnce(T2) -> T,
    {
        // if self.is_dag_select() {
        //     self.handle_dag(T::deserialize(deserializer))
        // }

        // select the matched list node, or setup the list
        // self.handle_node(SelectedNode::List)?;
        // let new_dag = self
        //     .is_dag_select()
        //     .then(|| init_new_dag(&iter))
        //     .unwrap_or_default();

        // select against each child
        // while let Some(_) = self.handle_index::<MC, U>(&mut iter)? {
        //     break;
        // }
        let (selector, mut state, cb, mut ctx) = self.into_parts();
        for index in 0usize.. {
            let field = Field::Index(index);
            let seed =
                Self::to_field_select_seed::<U>(&selector, &mut state, cb.clone(), ctx, &field)?;
            let res = match seed {
                None => (&mut iter).next_ignored::<MC>()?,
                Some(seed) => (&mut iter).next_element_seed::<MC, Ctx>(seed)?,
            };

            match res {
                None => break,
                Some(AstResult::Value(v)) => {}
                _ => {}
                // Some(AstResult::Continue) => Ok(Some(AstResult::Continue)),
                // Some(AstResult::Break) => Ok(Some(AstResult::Break)),
                _ => unimplemented!(),
            };
        }

        // finally, select the dag itself (if applicable)
        // if self.is_dag_select() {
        //     self.handle_dag(into_dag(new_dag))?;
        // }

        Ok(AstResult::Ok)
    }

    /// explore index, range, or all
    fn explore_list_range<'de, const MC: u64, U, I, R>(
        mut self,
        mut iter: I,
        range: R,
    ) -> Result<AstResult<T>, Error>
    where
        'a: 'de,
        // 'b: 'c,
        Ctx: 'de,
        U: Select<Ctx>,
        I: ListIterator<'de, U>,
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

        // // select the list node
        // self.handle_node(SelectedNode::List)?;

        // ignore everything before the start (unless 0)
        // if empty, return an err
        if start_idx > 0 {
            for index in 0usize..start_idx {
                if iter.next_ignored::<MC>()?.is_none() {
                    return Err(Error::explore_list_failure::<U>(self.selector, index));
                }
            }
        }

        // explore any/all indices in the range
        let (selector, mut state, cb, mut ctx) = self.into_parts();
        for index in range {
            let field = Field::Index(index);
            let res = if selector.next(Some(&field)).is_none() {
                (&mut iter).next_ignored::<MC>()?
            } else {
                let iter = &mut iter;
                let res = iter.next_element_seed::<MC, Ctx>(SelectorSeed::from_parts(
                    &selector,
                    state.descend::<U>(&field)?,
                    cb.clone(),
                    ctx,
                ))?;
                state.ascend::<U>();
                res
            };

            // if unbounded and empty, then we're done exploring
            // if bounded and empty, then we failed to explore everything
            if is_unbounded && res.is_none() {
                return Ok(AstResult::Ok);
            } else if res.is_none() && range_copy.contains(&index) {
                return Err(Error::explore_list_failure::<U>(&selector, index));
            }
        }

        // finish ignoring the remainder of the list
        if !is_unbounded {
            for _ in ignore_end_idx.unwrap().. {
                if iter.next_ignored::<MC>()?.is_none() {
                    break;
                }
            }
        }

        Ok(AstResult::Ok)
    }
}

// #[cfg(feature = "skipped")]
mod iterators {
    use super::*;
    use serde::de::IntoDeserializer;

    /*
    // /// A [`Select`]able list iterator over a serde sequence representation.
    // #[doc(hidden)]
    // #[derive(Debug)]
    // struct SerdeListIterator2<'de, const C: u64, A, T = IgnoredAny, S = DeserializeWrapper<C, T>> {
    //     seq: A,
    //     seed: S,
    //     _de: PhantomData<&'de T>,
    // }

    // impl<'de, const C: u64, A> SerdeListIterator2<'de, C, A> {
    //     pub const fn ignored( A) -> Self {
    //         SerdeListIterator2 {
    //             seq,
    //             seed: DeserializeWrapper::<C, IgnoredAny>::new(),
    //             _de: PhantomData,
    //         }
    //     }
    // }

    // impl<'de, const C: u64, A, T, S> SerdeListIterator2<'de, C, A, T, S> {
    //     pub const fn from(seq: A) -> SerdeListIterator2<'de, C, A, T> {
    //         SerdeListIterator2 {
    //             seq,
    //             seed: DeserializeWrapper::<C, T>::new(),
    //             _de: PhantomData,
    //         }
    //     }

    //     pub fn take(self) -> (S, SerdeListIterator2<'de, C, A, IgnoredAny>) {
    //         SerdeListIterator2 {
    //             seq: self.seq,
    //             seed: DeserializeWrapper::<C, IgnoredAny>::new(),
    //             _de: PhantomData,
    //         }
    //     }
    // }

    // impl<'de, const C: u64, A, T, S> Iterator for SerdeListIterator2<'de, C, A, T, S>
    // where
    //     A: SeqAccess<'de>,
    //     S: DeserializeSeed<'de, Value = T>,
    // {
    //     type Item = Result<T, A::Error>;
    //     fn next(&mut self) -> Option<Self::Item> {
    //         match self.seq.next_element_seed(seed) {
    //             Ok(Some(val)) => Some(Ok(val)),
    //             Ok(None) => None,
    //             Err(err) => Some(Err(err)),
    //         }
    //     }
    // }
     */

    /// A [`Select`]able list iterator over a serde sequence representation.
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct SeqAccessIterator<'de, A> {
        seq: A,
        index: usize,
        _t: PhantomData<&'de ()>,
    }
    impl<'de, A> From<A> for SeqAccessIterator<'de, A> {
        fn from(seq: A) -> Self {
            Self {
                seq,
                index: 0,
                _t: PhantomData,
            }
        }
    }
    impl<'de: 'a, 'a, T, A> ListIterator<'a, T> for SeqAccessIterator<'de, A>
    where
        T: Representation,
        A: SeqAccess<'de>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.seq.size_hint()
        }

        fn index(&self) -> usize {
            self.index
        }

        fn next_ignored<const MC: u64>(&mut self) -> Result<Option<AstResult<T>>, Error>
where
            // 'a: 'b,
            // 'b: 'de,
            // Ctx: Context,
            // T: Select<Ctx>,
        {
            let res = self
                .seq
                .next_element::<IgnoredAny>()
                .map_err(|_| Error::explore_index_failure::<IgnoredAny>(self.index))?;
            match res {
                None => Ok(None),
                Some(_) => {
                    self.index += 1;
                    Ok(Some(AstResult::Ok))
                }
            }
        }

        /*
        fn next<const C: u64>(&mut self) -> Result<Option<T>, Error> {
            let dag = self
                .seq
                .next_element_seed(DeserializeRepr::<C, T>::new())
                .map_err(|_| Error::explore_index_failure::<T>(self.index))?;

            if dag.is_none() {
                Ok(None)
            } else {
                self.index += 1;
                Ok(dag)
            }
        }
        fn next_seed<'de, const C: u64, Ctx: Context>(
            &mut self,
            seed: SelectorSeed<'de, Ctx, T>,
        ) -> Result<bool, Error>
        where
            T: Select<Ctx>,
        {
            let is_empty = self
                .seq
                .next_element_seed(DeserializeSelect::<C, Ctx, _, T>::from(seed))
                .map_err(|_| Error::explore_index_failure::<T>(self.index))?
                .is_none();
            if !is_empty {
                self.index += 1;
            }
            Ok(is_empty)
        }
         */

        fn next_element_seed<'b, const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'b, Ctx, T>,
        ) -> Result<Option<AstResult<T>>, Error>
        where
            // 'a: 'b,
            // 'b: 'de,
            'de: 'b,
            Ctx: Context + 'de,
            T: Select<Ctx> + 'de,
        {
            let res = self
                .seq
                .next_element_seed(<T as Select<_>>::Walker::<'b, MC>::from(seed))
                .map_err(|_| Error::explore_index_failure::<T>(self.index))?;
            if res.is_some() {
                self.index += 1;
            }
            Ok(res)
        }
    }

    /*
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
     */
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_match() {}
}
