use crate::dev::*;
use std::collections::BTreeMap;

///
pub type Map<K = String, V = Value> = BTreeMap<K, V>;

// TODO: write the 4 Select impls, then the latter 3 for BTreeMap<K, Link<V>>

impl<K: Representation + Ord, V: Representation> Representation for Map<K, V> {
    const NAME: &'static str = "Map";
    // const SCHEMA: &'static str = format!("type {} = &{}", Self::NAME, T::NAME);
    const KIND: Kind = Kind::Map;

    fn has_links(&self) -> bool {
        self.iter().any(|(k, v)| k.has_links() || v.has_links())
    }
}

///
/// ? should be explicitly implemented for each concrete Map<K, V>
#[macro_export]
macro_rules! impl_ipld_map {
    ($inner_ty:ty) => {
        $crate::dev::impl_ipld_map! {
            @visit_map {} {}
            $crate::dev::Map<K, Vinner_ty> [ $inner_ty ]
        }
    };

    (@visit_map
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty [ $inner_ty:ty ]
    ) => {
        impl<'a, 'de, C, T, $($generics)*> $crate::dev::Visitor<'de> for $crate::dev::ContextSeed<'a, C, $type, T>
        where
            T: $crate::dev::Representation + Send + Sync + 'static,
            C: $crate::dev::Context,
            for<'b> $crate::dev::ContextSeed<'b, C, $inner_ty, T>: $crate::dev::DeserializeSeed<'de, Value = Option<T>>,
            $($bounds)*
        {
            type Value = Option<T>;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", <$type>::NAME)
            }

            #[inline]
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: $crate::dev::SeqAccess<'de>,
            {
                match self.selector {
                    s if type_eq::<$type, T>() && s.is_matcher() => {
                        let dag = self.into::<$type, T>().visit_map(seq)?;
                        Ok(take_concrete_type::<_, T>(dag))
                    }
                    s if s.is_explore_fields() => self.into::<$type, T>().visit_map_fields(seq),
                    s if s.is_explore_all() => self.into::<$type, T>().visit_map_full(seq),
                    _ => Err(A::Error::custom($crate::Error::unsupported_selector::<$type, T>(self.selector))),
                }
            }
        }
    };
}

impl<'a, C, K, V, W> ContextSeed<'a, C, Map<K, V>, W>
where
    C: Context,
    K: Representation + Ord,
    V: Representation + Send + Sync + 'static,
    W: Representation + Send + Sync + 'static,
{
    ///
    pub fn visit_map<'de, A>(self, mut seq: A) -> Result<Option<Map<K, V>>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = Option<V>>,
    {
        let Self {
            selector,
            state,
            ctx,
            ..
        } = self;

        let matcher = selector
            .assert_matcher::<Map<K, V>, W>()
            .map_err(A::Error::custom)?;

        // match state.mode() {
        //     SelectorState::NODE_MODE => {
        //         state
        //             .send_matched(Node::Map < K, Vmatcher.label.clone())
        //             .map_err(A::Error::custom)?;

        //         for i in 0.. {
        //             let seed = ContextSeed::<C, V>::from(selector, state, ctx)
        //                 .descend_index(i)
        //                 .map_err(A::Error::custom)?;
        //             if let None = seq.next_element_seed(seed)? {
        //                 break;
        //             }
        //         }

        //         Ok(None)
        //     }
        //     SelectorState::DAG_MATCH_MODE | SelectorState::DAG_MODE => {
        //         let mut dag = Vec::with_capacity(seq.size_hint().unwrap_or(256));

        //         for i in 0.. {
        //             let seed = ContextSeed::<C, V>::from(selector, state, ctx)
        //                 .descend_index(i)
        //                 .map_err(A::Error::custom)?;
        //             match seq.next_element_seed(seed)? {
        //                 Some(Some(inner)) => dag.push(inner),
        //                 None => break,
        //                 Some(None) => unreachable!(),
        //             };
        //         }

        //         match state.mode() {
        //             SelectorState::DAG_MATCH_MODE => Ok(Some(dag)),
        //             SelectorState::DAG_MODE => {
        //                 state
        //                     .send_dag(dag, matcher.label.clone())
        //                     .map_err(A::Error::custom)?;
        //                 Ok(None)
        //             }
        //             _ => unreachable!(),
        //         }
        //     }
        // }

        unimplemented!()
    }

    ///
    pub fn visit_map_fields<'de, A>(self, mut seq: A) -> Result<Option<W>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, V, W>: DeserializeSeed<'de, Value = Option<W>>,
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
    pub fn visit_map_full<'de, A>(self, mut seq: A) -> Result<Option<W>, A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, V, W>: DeserializeSeed<'de, Value = Option<W>>,
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

impl<'de, 'a, C: Context, K: Representation, V: Representation, W: Representation>
    DeserializeSeed<'de> for ContextSeed<'a, C, Map<K, V>, W>
where
    K: Ord,
    // ContextSeed<'a, C, V, W>: DeserializeSeed<'de, Value = Option<V>>,
    ContextSeed<'a, C, Map<K, V>, W>: Visitor<'de, Value = Option<W>>,
{
    type Value = Option<W>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}
