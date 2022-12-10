use crate::dev::*;
use macros::repr_serde;
use maybestd::{
    cell::RefCell, collections::BTreeMap, fmt, iter, marker::PhantomData, str::FromStr,
};
use serde::de::value::MapAccessDeserializer;

pub use iterators::*;

/// A map type, implemented as a [`BTreeMap`].
///
/// [`BTreeMap`]: crate::maybestd::collections::BTreeMap
/// TODO: indexmap?
pub type Map<K = String, V = Any> = BTreeMap<K, V>;

impl<K, V> Representation for Map<K, V>
where
    // TODO: remove clone requirement by switching up callbacks
    K: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    // K: AsRef<Field<'_>>
    V: Representation,
{
    const NAME: &'static str = "Map";
    const SCHEMA: &'static str = "type Map {String:Any}";
    const DATA_MODEL_KIND: Kind = Kind::Map;

    fn has_links(&self) -> bool {
        self.iter().any(|(k, v)| k.has_links() || v.has_links())
    }

    fn to_selected_node(&self) -> SelectedNode {
        SelectedNode::Map
    }

    #[inline]
    #[doc(hidden)]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, val) in self {
            map.serialize_entry(
                &SerializeRepr::<'_, C, _>(key),
                &SerializeRepr::<'_, C, _>(val),
            )?;
        }
        map.end()
    }

    #[inline]
    #[doc(hidden)]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MapVisitor<const C: u64, K, V>(PhantomData<(K, V)>);
        impl<'de, const C: u64, K, V> Visitor<'de> for MapVisitor<C, K, V>
        where
            K: StringRepresentation,
            <K as FromStr>::Err: fmt::Display,
            V: Representation,
        {
            type Value = Map<K, V>;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A map of `{}` to `{}`", K::NAME, V::NAME)
            }
            #[inline]
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut new_map = Map::new();

                // while let Some(key) = iter.next_key::<C, K>()? {
                //     new_map.insert(key, iter.next_value::<C, K, V>()?)
                // }

                let mut iter = SerdeMapIterator::<'de, A>::from(map);
                while let Some(key) = <_ as MapIterator<K, V>>::next_key::<C>(&mut iter, None)
                    .map_err(A::Error::custom)?
                {
                    let val = key
                        .as_field()
                        .ok_or_else(|| Error::explore_key_failure::<K>(None))
                        .and_then(|field| {
                            <_ as MapIterator<K, V>>::next_value::<C>(&mut iter, &field)
                        })
                        .map_err(A::Error::custom)?;
                    new_map.insert(key, val);
                }

                Ok(new_map)
            }
        }

        deserializer.deserialize_map(MapVisitor::<C, K, V>(PhantomData))
    }
}

repr_serde! { @select for Map<K, V>
    { K, V } { K: Select<Ctx> + StringRepresentation + 'static,
               <K as FromStr>::Err: fmt::Display,
               V: Select<Ctx> + 'static }
}
repr_serde! { @visitors for Map<K, V>
    { K, V } { K: Select<Ctx> + StringRepresentation + 'static,
               <K as FromStr>::Err: fmt::Display,
               V: Select<Ctx> + 'static } @serde {
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A map of type {} of {} to {}", <Map<K, V>>::NAME, K::NAME, V::NAME)
    }
    #[inline]
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if let Some(s) = self.as_ref().selector.as_explore_union() {
            if s.matches_first() {
                // TODO: transform the seed to a phantom seed, then recurse
                let map = <Map<K, V>>::deserialize::<MC, _>(MapAccessDeserializer::new(map))?;
                return map.__select_in(self.into_inner()).map_err(A::Error::custom);
            }
        }

        self.into_inner()
            .select_map::<MC, false, K, V, _>(SerdeMapIterator::from(map))
            .map_err(A::Error::custom)
        /*
        match self.as_ref().selector {
            Selector::Matcher(_) => {
                self.into_inner().match_map::<C, K, V, _, _, _, _, _>(
                    iter,
                    |_| RefCell::default(),
                    |key, dag| Box::new(|child, _| {
                        dag.borrow_mut().extend(iter::once((key.clone(), child)));
                        Ok(())
                    }),
                    RefCell::into_inner,
                ).map_err(A::Error::custom)
            },
            Selector::ExploreFields(_) => self.into_inner()
                .explore_map_fields::<C, K, V, _>(iter)
                .map_err(A::Error::custom),
            Selector::ExploreAll(_) => self.into_inner()
                .explore_map_fields::<C, K, V, _>(iter)
                .map_err(A::Error::custom),
            _ => Err(A::Error::custom(Error::unsupported_selector::<Map<K, V>>(
                self.as_ref().selector,
            ))),
        }
         */
    }
}}

mod iterators {
    use super::*;

    ///
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct SerdeMapIterator<'de, A>(A, PhantomData<&'de ()>);
    impl<'de, A> From<A> for SerdeMapIterator<'de, A> {
        fn from(inner: A) -> Self {
            Self(inner, PhantomData)
        }
    }
    impl<'de, K, V, A> MapIterator<K, V> for SerdeMapIterator<'de, A>
    where
        K: StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Representation,
        A: MapAccess<'de>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.0.size_hint()
        }

        fn next_key<const C: u64>(
            &mut self,
            expected_field_name: Option<&'static str>,
        ) -> Result<Option<K>, Error> {
            let key = self
                .0
                .next_key_seed(DeserializeRepr::<C, K>::new())
                .or_else(|_| Err(Error::explore_key_failure::<K>(expected_field_name)))?;

            // TODO: assert that key == expected_field_name
            Ok(key)
        }

        fn next_value_ignored(&mut self, field: &Field<'_>) -> Result<(), Error> {
            self.0
                .next_value::<IgnoredAny>()
                .or_else(|_| Err(Error::explore_value_failure::<IgnoredAny>(field)))?;
            Ok(())
        }

        fn next_value<const C: u64>(&mut self, field: &Field<'_>) -> Result<V, Error> {
            self.0
                .next_value_seed(DeserializeRepr::<C, V>::new())
                .or_else(|_| Err(Error::explore_value_failure::<V>(field)))
        }

        fn next_value_seed<'a, const C: u64, Ctx: Context>(
            &mut self,
            seed: SelectorSeed<'a, Ctx, V>,
            // field: &Field<'_>,
        ) -> Result<(), Error>
        where
            V: Select<Ctx>,
        {
            // let key = <Self as MapIterator<K, V>>::key(self);
            // let field = Representation::as_field(key);
            // let field = self.field();
            // let err = || Error::explore_value_failure::<V>(field);

            // V::__select_map::<C, _>(seed, &mut self.inner, false)
            //     .ok()
            //     .flatten()
            //     .ok_or_else(|| Error::explore_value_failure::<V>(field))
            unimplemented!()
        }
    }

    /*
    struct MemoryMapIterator<'a, K, V, I> {
        iter: I,
        // index: usize,
        _t: PhantomData<&'a (K, V)>,
    }

    impl<'a, K, V, I> MemoryMapIterator<'a, K, V, I> {
        // const fn is_last(&self) -> bool {
        //     self.len == self.index + 1
        // }
    }

    impl<'a, K, V, I> MapIterator<K, V> for MemoryMapIterator<'a, K, V, I>
    where
        K: Representation,
        V: Representation,
        I: Iterator<Item = (&'a K, &'a V)> + iter::ExactSizeIterator,
    {
        fn size_hint(&self) -> Option<usize> {
            Some(self.iter.len())
        }

        fn field(&self) -> Field<'_> {
            unimplemented!()
        }

        fn next_key<const C: u64>(
            &mut self,
            expected_field_name: Option<&'static str>,
        ) -> Result<Option<K>, Error>
        where
            K: Representation,
        {
            // let key = self
            //     .inner
            //     .next_key_seed(DeserializeWrapper::<C, K>::default())
            //     .or_else(|_| Err(Error::explore_key_failure::<K>(expected_field_name)))?;

            // // TODO: assert that key == expected_field_name
            // Ok(key)
            unimplemented!()
        }

        fn next_value_ignored(&mut self, field: &Field<'_>) -> Result<(), Error> {
            // self.inner
            //     .next_value::<IgnoredAny>()
            //     .or_else(|_| Err(Error::explore_value_failure::<IgnoredAny>(field)))?;
            // Ok(())
            unimplemented!()
        }

        fn next_value<const C: u64>(&mut self, field: &Field<'_>) -> Result<V, Error>
        where
            V: Representation,
        {
            // self.inner
            //     .next_value_seed(DeserializeWrapper::<C, V>::default())
            //     .or_else(|_| Err(Error::explore_value_failure::<V>(field)))
            unimplemented!()
        }

        fn next_value_seed<'b, const C: u64, Ctx: Context>(
            &mut self,
            seed: SelectorSeed<'b, Ctx, V>,
            // field: &Field<'_>,
        ) -> Result<(), Error>
        where
            K: Representation,
            V: Select<Ctx>,
        {
            // let key = <Self as MapIterator<K, V>>::key(self);
            // let field = Representation::as_field(key);
            // let field = self.field();
            // let err = || Error::explore_value_failure::<V>(field);

            // V::__select_map::<C, _>(seed, &mut self.inner, false)
            //     .ok()
            //     .flatten()
            //     .ok_or_else(|| Error::explore_value_failure::<V>(field))
            unimplemented!()
        }
    }
     */
}

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Select<Ctx> + 'static,
{
    ///
    pub fn select_map<const C: u64, const IN: bool, K, V, I>(
        mut self,
        mut iter: I,
        // dag: Either<&T2, I>,
    ) -> Result<(), Error>
    where
        K: Select<Ctx> + StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Select<Ctx>,
        I: MapIterator<K, V>,
    {
        unimplemented!()
    }

    ///
    pub fn patch_map<const C: u64, const FLUSH: bool, K, V, I>(
        mut self,
        mut iter: I,
        // dag: Either<&T2, I>,
    ) -> Result<(), Error>
    where
        K: Select<Ctx> + StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Select<Ctx>,
        I: MapIterator<K, V>,
    {
        unimplemented!()
    }

    ///
    pub fn match_map<const C: u64, K, V, I, T2, F1, F2, F3>(
        mut self,
        mut iter: I,
        init_new_dag: F1,
        mut match_cb: F2,
        into_dag: F3,
    ) -> Result<(), Error>
    where
        K: Select<Ctx> + StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Select<Ctx>,
        I: MapIterator<K, V>,
        T2: Default + 'static,
        F1: FnOnce(&I) -> T2,
        for<'b> F2: FnMut(&'b K, &'b T2) -> Box<dyn MatchDagOp<V, Ctx> + 'b>,
        F3: FnOnce(T2) -> T,
    {
        // select the matched node, or set up the dag
        self.handle_node(SelectedNode::Map)?;
        let new_dag = self
            .is_dag_select()
            .then(|| init_new_dag(&iter))
            .unwrap_or_default();

        // select against each child
        while let Some(key) = iter.next_key::<C>(None)? {
            self.handle_field::<C, K, V>(
                &mut iter,
                self.is_dag_select().then(|| match_cb(&key, &new_dag)),
            )?;
        }

        // // let mut seed = self;
        // // let match_cb = &mut match_cb;
        // loop {
        //     // let seed = self.to_field_select_seed(field, match_cb)
        //     // let mut seed = self;
        //     if iter.next_entry_seed::<C, Ctx, _>(|key| {
        //         let field = key
        //             .as_field()
        //             .ok_or_else(|| Error::explore_key_failure::<K>(None))?;
        //         self.to_field_select_seed(
        //             &field,
        //             self.is_dag_select().then(|| match_cb(&key, &new_dag)),
        //         )
        //     })? {
        //         break;
        //     }
        // }
        // TODO: should be iter.next_entry_seed(|key| self.to_field_select_seed())
        // while !iter.next_entry_seed::<C, T, Ctx>(&mut self)? {}

        // finally, select the matched dag
        if self.is_dag_select() {
            self.handle_dag(into_dag(new_dag))?;
        }

        Ok(())
    }

    ///
    pub(crate) fn explore_map_fields<const C: u64, K, V, I>(self, mut iter: I) -> Result<(), Error>
    where
        K: Select<Ctx> + StringRepresentation + 'static,
        <K as FromStr>::Err: fmt::Display,
        V: Select<Ctx> + 'static,
        I: MapIterator<K, V>,
    {
        unimplemented!()
    }
}
