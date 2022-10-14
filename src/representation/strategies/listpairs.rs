use crate::dev::*;
use macros::{
    derive_more::{AsMut, AsRef, Deref, Index, IndexMut, IntoIterator},
    repr_serde,
};
use maybestd::{fmt, marker::PhantomData, str::FromStr};

const STRATEGY: Strategy = Strategy::ListPairs;

/*
// Blanket impl for maps.
repr_serde! { @visitors for T => (K, V)
    { @dk (type_kinds::Map) @sk (type_kinds::Map) @rk (type_kinds::List) }
    { T, K, V } { T: Default + Extend<(K, V)> +  'static,
                  K: Select<Ctx> + StringRepresentation + 'static,
                  <K as FromStr>::Err: fmt::Display,
                  V: Select<Ctx> + 'static } @serde {
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A list of type {} of {}", S::NAME, T::NAME)
    }

    // #[inline]
    // fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    // where
    //     A: SeqAccess<'de>,
    // {
    //     if let Some(s) = self.0.selector.as_explore_union() {
    //         if s.matches_first() {
    //             let list = S::deserialize::<C, _>(SeqAccessDeserializer::new(seq))?;
    //             // todo: for each selector, __select_in
    //             return list.__select_in(self.0).map_err(A::Error::custom);
    //         } else {
    //             // todo: support multiple non-overlapping ranges
    //         }
    //     }

    //     let iter = SerdeListIterator::<'de, _>::from(seq);
    //     match self.0.selector {
    //         Selector::Matcher(_) => {
    //             self.0.match_list::<C, T, _, _, _, _, _>(
    //                 iter,
    //                 |_| RefCell::default(),
    //                 |dag| Box::new(|child, _| Ok(dag.borrow_mut().extend(iter::once(child)))),
    //                 RefCell::into_inner,
    //             ).map_err(A::Error::custom)
    //         },
    //         Selector::ExploreIndex(s) => self.0
    //             .explore_list_range::<C, T, _, _>(iter, s.to_range())
    //             .map_err(A::Error::custom),
    //         Selector::ExploreRange(s) => self.0
    //             .explore_list_range::<C, T, _, _>(iter, s.to_range())
    //             .map_err(A::Error::custom),
    //         Selector::ExploreAll(s) => self.0
    //             .explore_list_range::<C, T, _, _>(iter, s.to_range())
    //             .map_err(A::Error::custom),
    //         _ => Err(A::Error::custom(Error::unsupported_selector::<S>(
    //             self.0.selector,
    //         ))),
    //     }
    // }
}}
 */

// ///
// #[derive(
//     AsRef,
//     AsMut,
//     Clone,
//     Debug,
//     Deref,
//     // DerefMut,
//     Eq,
//     Hash,
//     Index,
//     IndexMut,
//     IntoIterator,
//     Ord,
//     PartialEq,
//     PartialOrd,
// )]
// #[as_ref(forward)]
// #[deref(forward)]
// pub struct ListPairsMap<K, V>(Map<K, V>);

// impl<K, V> Default for ListPairsMap<K, V> {
//     fn default() -> Self {
//         Self(Map::new())
//     }
// }

// type ListPair<K, V> = StructTuple2<(K, V), K, V>;
// type ListPairRef<'a, K, V> = StructTupleRef2<'a, (K, V), K, V>;

// impl<K, V> Representation for ListPairsMap<K, V>
// where
//     // TODO: remove clone requirement by switching up callbacks
//     K: Representation + AsRef<str> + Clone + Ord,
//     V: Representation,
// {
//     type ReprKind = type_kinds::List;

//     const NAME: &'static str = "Map";
//     const SCHEMA: &'static str = concat!(
//         "type Map {",
//         stringify!(K::NAME),
//         ":",
//         stringify!(V::NAME),
//         "} representation listpairs",
//     );
//     const DATA_MODEL_KIND: Kind = Kind::Map;
//     // const REPR_KIND: Kind = Kind::List;

//     fn has_links(&self) -> bool {
//         self.0.has_links()
//     }

//     #[inline]
//     #[doc(hidden)]
//     fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         use ser::SerializeSeq;

//         let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
//         for listpair in self.0.iter().map(ListPairRef::<K, V>::from) {
//             seq.serialize_element(&SerializeWrapper::<'_, C, _>(&listpair))?;
//         }
//         seq.end()
//     }

//     #[inline]
//     #[doc(hidden)]
//     fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct ListPairsMapVisitor<const C: u64, K, V>(PhantomData<(K, V)>);
//         impl<'de, const C: u64, K, V> Visitor<'de> for ListPairsMapVisitor<C, K, V>
//         where
//             K: Representation + Ord,
//             V: Representation,
//         {
//             type Value = ListPairsMap<K, V>;
//             #[inline]
//             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                 write!(f, "A map of `{}` to `{}` listpairs", K::NAME, V::NAME)
//             }
//             #[inline]
//             fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//             where
//                 A: SeqAccess<'de>,
//             {
//                 let mut map = ListPairsMap::default();
//                 while let Some(listpair) =
//                     seq.next_element_seed(DeserializeWrapper::<C, ListPair<K, V>>::new())?
//                 {
//                     map.0.insert(listpair.0, listpair.1);
//                 }
//                 Ok(map)

//                 // let mut new_map = Map::new();
//                 // let mut iter = SerdeListPairsIterator::<'de, A>::from(map);
//                 // while let Some(key) =
//                 //     <SerdeListPairsIterator<'de, A> as MapIterator<K, V>>::next_key::<C>(&mut iter, None)
//                 //         .map_err(A::Error::custom)?
//                 // {
//                 //     let val = key
//                 //         .as_field()
//                 //         .ok_or_else(|| Error::explore_key_failure::<K>(None))
//                 //         .and_then(|field| {
//                 //             <SerdeListPairsIterator<'de, A> as MapIterator<K, V>>::next_value::<C>(
//                 //                 &mut iter, &field,
//                 //             )
//                 //         })
//                 //         .map_err(A::Error::custom)?;
//                 //     new_map.insert(key, val);
//                 // }
//                 // Ok(new_map)
//             }
//         }

//         deserializer.deserialize_seq(ListPairsMapVisitor::<C, K, V>(PhantomData))
//     }
// }

mod iterators {
    use super::*;
    use de::value::SeqAccessDeserializer;

    // /// An iterator over a key-value pair represented as a serde list.
    // #[derive(Debug)]
    // pub struct SerdeListPairIterator<'de, A>
    // where
    //     A: SeqAccess<'de>,
    // {
    //     inner: A,
    //     _t: PhantomData<&'de ()>,
    // }

    // impl<'de, T, A> ListIterator<T> for SerdeListPairIterator<'de, A> where A: SeqAccess<'de> {}

    ///
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct SerdeListPairsIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        inner: A,
        // key: Option<Cow<'de, str>>,
        // pair_de: Option<SerdeListPairIterator<'de, A>>,
        _t: PhantomData<&'de ()>,
    }

    impl<'de, A> From<A> for SerdeListPairsIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        fn from(inner: A) -> Self {
            Self {
                inner,
                // key: None,
                // pair_de: None,
                _t: PhantomData,
            }
        }
    }

    impl<'de, K, V, A> MapIterator<K, V> for SerdeListPairsIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.inner.size_hint()
        }

        fn field(&self) -> Field<'_> {
            unimplemented!()
        }

        ///
        /// pass a visitor that deserializes the key, and if the field_name matches, deserializes the value
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

        fn next_value_seed<'a, const C: u64, Ctx: Context>(
            &mut self,
            seed: SelectorSeed<'a, Ctx, V>,
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! listpairs {
    ($name:ident $ref_name:ident => $($ty:ident)*) => {
        /// Type that implements selection against tuple-represented structs.
        #[doc(hidden)]
        // #[derive(Clone, Debug, From)]
        pub struct $ref_name<'a, T, $($ty,)*>($(pub &'a $ty,)* PhantomData<T>);

        impl<'a, T, $($ty,)*> $ref_name<'a, T, $($ty,)*>
        where
            // T: Representation,
            $($ty: Representation,)*
        {
            pub(crate) const LEN: usize = $crate::tuple!(@len $($ty)*);
        }

        // impl<'a, T, $($ty,)*> From<&'a ($($ty,)*)>
        //     for $ref_name<'a, T, $($ty,)*>
        // {
        //     #[inline]
        //     fn from(($($ty,)*): &'a ($($ty,)*)) -> Self {
        //         Self($($ty,)* PhantomData)
        //     }
        // }

        // impl<'a, T, $($ty,)*> From<($(&'a $ty,)*)>
        //     for $ref_name<'a, T, $($ty,)*>
        // {
        //     #[inline]
        //     fn from(($($ty,)*): ($(&'a $ty,)*)) -> Self {
        //         Self($($ty,)* PhantomData)
        //     }
        // }

        // impl<'a, T, $($ty,)*> From<&'a $name<T, $($ty,)*>>
        //     for $ref_name<'a, T, $($ty,)*>
        // {
        //     #[inline]
        //     fn from(tuple: &'a $name<T, $($ty,)*>) -> Self {
        //         let $name($(ref $ty,)* _) = tuple;
        //         Self($($ty,)* PhantomData)
        //     }
        // }

        ///
        #[doc(hidden)]
        // #[derive(Clone, Debug, From)]
        pub struct $name<T, $($ty,)*>($(pub $ty,)* PhantomData<T>);

        impl<T, $($ty,)*> $name<T, $($ty,)*>
        where
            // T: Representation,
            $($ty: Representation,)*
        {
            pub(crate) const LEN: usize = $crate::tuple!(@len $($ty)*);
        }

        // impl<T, $($ty,)*> From<($($ty,)*)> for $name<T, $($ty,)*> {
        //     fn from(($($ty,)*): ($($ty,)*)) -> Self {
        //         Self($($ty,)* PhantomData)
        //     }
        // }

        const _: () = {
            impl<'a, T, $($ty,)*> Representation for $ref_name<'a, T, $($ty,)*>
            where
                T: Representation,
                $($ty: Representation,)*
            {
                $crate::tuple!(@repr_consts $ref_name List => $($ty)*);

                fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
                where
                    Se: Serializer,
                {
                    use ser::SerializeSeq;

                    // let Self($($ty,)* _) = self;
                    // let mut seq = serializer.serialize_seq(Some(Self::LEN))?;
                    // $(
                    //     seq.serialize_element(&SerializeWrapper::<'_, MC, $ty>($ty))?;
                    // )*
                    // seq.end()
                    unimplemented!()
                }

                #[inline]
                #[doc(hidden)]
                fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
                where
                    De: Deserializer<'de>,
                {
                    unreachable!(
                        "cannot deserialize {}; use {}::as_ref instead",
                        stringify!($ref_name),
                        stringify!($name),
                    )
                }
            }

            impl<T, $($ty,)*> Representation for $name<T, $($ty,)*>
            where
                T: Representation,
                $($ty: Representation,)*
            {
                const NAME: &'static str = stringify!($name);
                const SCHEMA: &'static str = "";
                const DATA_MODEL_KIND: Kind = Kind::List;
                const SCHEMA_KIND: Kind = Kind::Struct;
                const REPR_KIND: Kind = Kind::List;
                const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

                $crate::tuple!(@repr_consts $name List => $($ty)*);

                fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
                where
                    Se: Serializer,
                {
                    // $ref_name::from(self).serialize::<MC, Se>(serializer)
                    unimplemented!()
                }

                #[inline]
                #[doc(hidden)]
                fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
                where
                    De: Deserializer<'de>,
                {
                    // Ok(Self::from(<($($ty,)*)>::deserialize::<MC, _>(deserializer)?))
                    unimplemented!()
                }
            }
        };
    };
}

// listpairs!(ListPairsStruct2 ListPairsStructRef2 => A B);
// listpairs!(ListPairsStruct3 ListPairsStructRef3 => A B C);
// listpairs!(ListPairsStruct4 ListPairsStructRef4 => A B C D);
// listpairs!(ListPairsStruct5 ListPairsStructRef5 => A B C D E);
// listpairs!(ListPairsStruct6 ListPairsStructRef6 => A B C D E F);
// listpairs!(ListPairsStruct7 ListPairsStructRef7 => A B C D E F G);
// listpairs!(ListPairsStruct8 ListPairsStructRef8 => A B C D E F G H);
// listpairs!(ListPairsStruct9 ListPairsStructRef9 => A B C D E F G H I);
// listpairs!(ListPairsStruct10 ListPairsStructRef10 => A B C D E F G H I J);
// listpairs!(ListPairsStruct11 ListPairsStructRef11 => A B C D E F G H I J K);
// listpairs!(ListPairsStruct12 ListPairsStructRef12 => A B C D E F G H I J K L);
// listpairs!(ListPairsStruct13 ListPairsStructRef13 => A B C D E F G H I J K L M);
// listpairs!(ListPairsStruct14 ListPairsStructRef14 => A B C D E F G H I J K L M N);
// listpairs!(ListPairsStruct15 ListPairsStructRef15 => A B C D E F G H I J K L M N O);
// listpairs!(ListPairsStruct16 ListPairsStructRef16 => A B C D E F G H I J K L M N O P);
// listpairs!(ListPairsStruct17 ListPairsStructRef17 => A B C D E F G H I J K L M N O P Q);
// listpairs!(ListPairsStruct18 ListPairsStructRef18 => A B C D E F G H I J K L M N O P Q R);
// listpairs!(ListPairsStruct19 ListPairsStructRef19 => A B C D E F G H I J K L M N O P Q R S);
