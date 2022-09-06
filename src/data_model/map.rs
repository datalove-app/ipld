use crate::dev::*;
use macros::impl_ipld_serde;
use std::{cell::RefCell, collections::BTreeMap, fmt};

///
pub type Map<K = IpldString, V = Any> = BTreeMap<K, V>;

// TODO: write the 4 Select impls, then the latter 3 for BTreeMap<K, Link<V>>

impl<K, V> Representation for Map<K, V>
where
    K: Representation + Ord,
    V: Representation,
{
    const NAME: &'static str = "Map";
    const SCHEMA: &'static str = concat!(
        "type Map {",
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

impl_ipld_serde! { @context_seed_visitor
    { K: Representation + Ord + 'static, V: Representation + 'static }
    { for<'b> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
      for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>, }
    Map<K, V>
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Map::<K, V>::NAME)
    }

        #[inline]
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        match self.selector {
            Selector::Matcher(_) => self.match_map(map),
            // Selector::ExploreFields(_) => self.explore_map_fields(map),
            // Selector::ExploreAll(_) => self.explore_map_fields(map),
            _ => Err(A::Error::custom(Error::unsupported_selector::<Map<K, V>>(
                self.selector,
            ))),
        }
    }
}}

impl_ipld_serde! { @context_seed_visitor_ext
    { K: Representation + Ord + 'static, V: Representation + 'static }
    { for<'b> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
      for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>, }
    Map<K, V> {}
}

impl_ipld_serde! { @context_seed_deseed
    { K: Representation + Ord + 'static, V: Representation + 'static }
    { for<'b> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
      for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>, }
    Map<K, V>
{
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}}

impl_ipld_serde! { @context_seed_select
    { K: Representation + Ord + 'static, V: Representation + 'static }
    { for<'b, 'de> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
      for<'b, 'de> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>, }
    Map<K, V>
}

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
    V: Representation + 'static,
    // W: Representation + Send + Sync + 'static,
{
    ///
    pub(crate) fn match_map<'de, A>(mut self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        for<'b> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>,
    {
        let matcher = self
            .selector
            .as_matcher()
            .expect("should know that this is a matcher");

        let mode = self.mode();
        let mut dag: RefCell<Map<K, V>> = Default::default();

        match mode {
            SelectionMode::SelectNode => {
                self.select_matched_node(SelectedNode::Map, matcher.label.as_deref())
                    .map_err(A::Error::custom)?;
            }
            _ => unimplemented!(),
        }

        let (selector, state, mut params, ctx) = self.into_parts();

        unimplemented!()
    }

    ///
    pub(crate) fn visit_map_fields<'de, A>(self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        for<'b> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>,
    {
        unimplemented!()
    }

    ///
    pub(crate) fn visit_map_all<'de, A>(self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        for<'b> ContextSeed<'b, C, K>: DeserializeSeed<'de, Value = ()>,
        for<'b> ContextSeed<'b, C, V>: DeserializeSeed<'de, Value = ()>,
    {
        unimplemented!()
    }
}

// impl<'de, 'a, C, K, V> DeserializeSeed<'de> for ContextSeed<'a, C, Map<K, V>>
// where
//     C: Context,
//     K: Representation + Ord + 'static,
//     V: Representation + 'static,
//     // ContextSeed<'a, C, V, W>: DeserializeSeed<'de, Value = Option<V>>,
//     // ContextSeed<'a, C, Map<K, V>>: Visitor<'de, Value = ()>,
// {
//     type Value = ();

//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_map(self)
//     }
// }

// impl<'a, C, K, V> Select<C> for Map<K, V>
// where
//     C: Context,
//     K: Representation + Ord + 'static,
//     V: Representation + Send + Sync + 'static,
// {
//     fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
// }
