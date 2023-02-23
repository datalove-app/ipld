use crate::dev::*;
use macros::{
    derive_more::{AsMut, AsRef, Deref, Index, IndexMut, IntoIterator},
    repr_serde,
};
use maybestd::{fmt, marker::PhantomData, str::FromStr};

const STRATEGY: Strategy = Strategy::Listpairs;

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

///
#[derive(
    AsRef,
    AsMut,
    Clone,
    Debug,
    Deref,
    // DerefMut,
    Eq,
    Hash,
    Index,
    IndexMut,
    IntoIterator,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[as_ref(forward)]
#[deref(forward)]
pub struct ListpairsMap<K, V>(Map<K, V>)
where
    K: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    V: Representation;

impl<K, V> Default for ListpairsMap<K, V>
where
    K: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    V: Representation,
{
    fn default() -> Self {
        Self(Map::new())
    }
}

type ListpairRef<'a, K, V> = TupleRef2<'a, ListpairsMap<K, V>, K, V>;

impl<K, V> Representation for ListpairsMap<K, V>
where
    K: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    V: Representation,
{
    const NAME: &'static str = "Map";
    const SCHEMA: &'static str = concat!(
        "type Map {",
        stringify!(K::NAME),
        ":",
        stringify!(V::NAME),
        "} representation listpairs",
    );
    const DATA_MODEL_KIND: Kind = Kind::Map;
    const SCHEMA_KIND: Kind = Kind::Map;
    const REPR_KIND: Kind = Kind::List;
    const REPR_STRATEGY: Strategy = Strategy::Listpairs;

    #[inline]
    #[doc(hidden)]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for listpair in self.0.iter().map(ListpairRef::<'_, K, V>::from) {
            seq.serialize_element(&SerializeRepr::<'_, C, _>(&listpair))?;
        }
        seq.end()
    }

    #[inline]
    #[doc(hidden)]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ListpairsMapVisitor<const C: u64, K, V>(PhantomData<(K, V)>);
        impl<'de, const C: u64, K, V> Visitor<'de> for ListpairsMapVisitor<C, K, V>
        where
            K: StringRepresentation,
            <K as FromStr>::Err: fmt::Display,
            V: Representation,
        {
            type Value = ListpairsMap<K, V>;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A map of `{}` to `{}` listpairs", K::NAME, V::NAME)
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = ListpairsMap::default();
                while let Some((key, val)) =
                    seq.next_element_seed(DeserializeRepr::<C, (K, V)>::new())?
                {
                    map.0.insert(key, val);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_seq(ListpairsMapVisitor::<C, K, V>(PhantomData))
    }
}

repr_serde! { @select for ListpairsMap<K, V>
    { K, V } { K: Select<Ctx> + StringRepresentation + 'static,
               <K as FromStr>::Err: fmt::Display,
               V: Select<Ctx> + 'static }
}
repr_serde! { @visitors for ListpairsMap<K, V>
    { K, V } { K: Select<Ctx> + StringRepresentation + 'static,
               <K as FromStr>::Err: fmt::Display,
               V: Select<Ctx> + 'static } @serde {
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A map of type {} of {} to {}", <ListpairsMap<K, V>>::NAME, K::NAME, V::NAME)
    }
    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
    A: SeqAccess<'de>,
    {
        /*
        if let Some(s) = self.as_ref().selector.as_explore_union() {
            if s.matches_first() {
                // TODO: transform the seed to a phantom seed, then recurse
                let map = <ListpairsMap<K, V>>::deserialize::<MC, _>(SeqAccessDeserializer::new(map))?;
                return map.__select_in(self.into_inner()).map_err(A::Error::custom);
            }
        }

        self.into_inner()
            .select_map::<MC, false, K, V, _>(SerdeMapIterator::from(map))
            .map_err(A::Error::custom)
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

        unimplemented!()
    }
}}

mod iterators {
    use super::*;
    use de::value::SeqAccessDeserializer;

    ///
    #[doc(hidden)]
    // #[derive(Debug)]
    pub struct SerdeListpairsIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        inner: A,
        index: usize,
        // key: Option<Cow<'de, str>>,
        // value_de: Option<Box<dyn erased_serde::Deserializer<'de>>>,
        _t: PhantomData<&'de ()>,
    }

    impl<'de, A> SerdeListpairsIterator<'de, A>
    where
        A: SeqAccess<'de>,
    {
        pub fn new(inner: A) -> Self {
            Self {
                inner,
                index: 0,
                // key: None,
                // value_de: None,
                _t: PhantomData,
            }
        }
    }

    ///
    /// call to next_key:
    ///     - next_element -> (key(_str), Box<dyn SeqAccessDe>)
    /// call to next_value:
    ///     - creates Visitor.visit_seq that plucks one V from the seq
    ///     - value_de.deserialize_seq ->
    impl<'de, K, V, A> MapIterator<K, V> for SerdeListpairsIterator<'de, A>
    where
        K: StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Representation,
        A: SeqAccess<'de>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.inner.size_hint()
        }

        ///
        /// if name is provided
        fn next_key<'a, const MC: u64>(
            &'a mut self,
            expected: Option<&Field<'static>>,
        ) -> Result<Option<AstResult<'a, K>>, Error> {
            unimplemented!()
        }

        // fn next_value_ignored(&mut self, field: &Field<'_>) -> Result<(), Error> {
        //     unimplemented!()
        // }

        fn next_value_seed<'a: 'b, 'b, const MC: u64, Ctx, F>(
            &'a mut self,
            seeder: F,
        ) -> Result<AstResult<'a, V>, Error>
        where
            Ctx: Context + 'b,
            V: Select<Ctx>,
            F: FnOnce(&Field<'_>) -> Result<Option<SelectorSeed<'b, Ctx, V>>, Error>,
        {
            unimplemented!()
        }
    }

    struct EntryVisitor<const C: u64, K, V> {
        expected_field_name: Option<&'static str>,
        _t: PhantomData<(K, V)>,
    }
    impl<const C: u64, K, V> EntryVisitor<C, K, V> {
        fn new(expected_field_name: Option<&'static str>) -> Self {
            Self {
                expected_field_name,
                _t: PhantomData,
            }
        }
    }
    impl<'de, const C: u64, K, V> Visitor<'de> for EntryVisitor<C, K, V>
    where
        K: StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Representation,
    {
        // type Value = (K, Box<dyn erased_serde::Deserializer<'de>>);
        type Value = ();
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "A listpairs map key-value pair of {} to {}",
                K::NAME,
                V::NAME
            )
        }
        #[inline]
        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            unimplemented!()
        }
    }
    impl<'de, const C: u64, K, V> DeserializeSeed<'de> for EntryVisitor<C, K, V>
    where
        K: StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Representation,
    {
        // type Value = (K, Box<dyn erased_serde::Deserializer<'de>>);
        type Value = ();
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(self)
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
