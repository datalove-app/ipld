use crate::dev::*;
use std::collections::BTreeMap;

///
pub type Map<K = String, V = Any> = BTreeMap<K, V>;

// TODO: write the 4 Select impls, then the latter 3 for BTreeMap<K, Link<V>>

impl<K, V> Representation for Map<K, V>
where
    K: Representation + Ord,
    V: Representation,
{
    const NAME: &'static str = "Map";
    const SCHEMA: &'static str = concat!(
        "type ",
        stringify!(Self::NAME),
        " {",
        stringify!(K::NAME),
        ":",
        stringify!(V::NAME),
        "}",
    );
    const DATA_MODEL_KIND: Kind = Kind::Map;

    fn has_links(&self) -> bool {
        self.iter().any(|(k, v)| k.has_links() || v.has_links())
    }
}

// impl_ipld_serde! { @context_visitor
//     { K: Representation + Ord + 'static, V: Representation }
//     { for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>, }
//     Map<K, V>
// {
//     #[inline]
//     fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", Map::<K, V>::NAME)
//     }
// }}

// impl_ipld_serde! { @context_deseed
//     { K: Representation + Ord + 'static, V: Representation }
//     { for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>, }
//     Map<K, V>
// {
//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_map(self)
//     }
// }}

/*
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
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: $crate::dev::MapAccess<'de>,
            {
                match self.selector {
                    s if type_eq::<$type, T>() && s.is_matcher() => {
                        let dag = self.into::<$type, T>().visit_map(map)?;
                        Ok(take_concrete_type::<_, T>(dag))
                    }
                    s if s.is_explore_fields() => self.into::<$type, T>().visit_map_fields(map),
                    s if s.is_explore_all() => self.into::<$type, T>().visit_map_all(map),
                    _ => Err(A::Error::custom($crate::Error::unsupported_selector::<$type, T>(self.selector))),
                }
            }
        }
    };
}
 */

impl<'a, C, K, V> ContextSeed<'a, C, Map<K, V>>
where
    C: Context,
    K: Representation + Ord + 'static,
    V: Representation + Send + Sync + 'static,
    // W: Representation + Send + Sync + 'static,
{
    ///
    pub(crate) fn visit_map<'de, A>(self, mut map: A) -> Result<Option<Map<K, V>>, A::Error>
    where
        A: MapAccess<'de>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = Option<V>>,
    {
        let matcher = self
            .selector
            .as_matcher()
            .expect("should know that this is a matcher");

        // match state.mode() {
        //     SelectorState::NODE_MODE => {
        //         state
        //             .send_matched(Node::Map < K, Vmatcher.label.clone())
        //             .map_err(A::Error::custom)?;
        //
        //         for i in 0.. {
        //             let seed = ContextSeed::<C, V>::from(selector, state, ctx)
        //                 .descend_index(i)
        //                 .map_err(A::Error::custom)?;
        //             if let None = seq.next_element_seed(seed)? {
        //                 break;
        //             }
        //         }
        //
        //         Ok(None)
        //     }
        //     SelectorState::DAG_MATCH_MODE | SelectorState::DAG_MODE => {
        //         let mut dag = Vec::with_capacity(seq.size_hint().unwrap_or(256));
        //
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
        //
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
    pub(crate) fn visit_map_fields<'de, A>(self, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>,
    {
        unimplemented!()
    }

    ///
    pub(crate) fn visit_map_all<'de, A>(self, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>,
    {
        unimplemented!()
    }
}

impl<'de, 'a, C, K, V> DeserializeSeed<'de> for ContextSeed<'a, C, Map<K, V>>
where
    C: Context,
    K: Representation + Ord + 'static,
    V: Representation + Send + Sync + 'static,
    // ContextSeed<'a, C, V, W>: DeserializeSeed<'de, Value = Option<V>>,
    ContextSeed<'a, C, Map<K, V>>: Visitor<'de, Value = ()>,
{
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'a, C, K, V> Select<C> for Map<K, V>
where
    C: Context,
    K: Representation + Ord + 'static,
    V: Representation + Send + Sync + 'static,
{
    fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
        unimplemented!()
    }
}
