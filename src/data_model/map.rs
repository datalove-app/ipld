use crate::dev::*;
use macros::impl_selector_seed_serde;
use std::{cell::RefCell, collections::BTreeMap, fmt};

///
/// TODO: indexmap?
pub type Map<K = IpldString, V = Any> = BTreeMap<K, V>;

impl<K, V> Representation for Map<K, V>
where
    // TODO: remove clone requirement by switching up callbacks
    K: Representation + Clone + Ord + AsRef<str>,
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

impl_selector_seed_serde! { @codec_seed_visitor
    { K: Representation + Clone + Ord + AsRef<str> + 'static,
      V: Representation + 'static }
    { for<'b> CodedSeed<'b, C, Ctx, K>: DeserializeSeed<'de, Value = ()>,
      for<'b> CodedSeed<'b, C, Ctx, V>: DeserializeSeed<'de, Value = ()>, }
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
        match self.0.selector {
            Selector::Matcher(_) => self.match_map(map),
            Selector::ExploreFields(_) => self.explore_map_fields(map),
            Selector::ExploreAll(_) => self.explore_map_all(map),
            _ => Err(A::Error::custom(Error::unsupported_selector::<Map<K, V>>(
                self.0.selector,
            ))),
        }
    }
}}

impl_selector_seed_serde! { @codec_seed_visitor_ext
    { K: Representation + Clone + Ord + AsRef<str> + 'static,
      V: Representation + 'static }
    { for<'b> CodedSeed<'b, C, Ctx, K>: DeserializeSeed<'de, Value = ()>,
      for<'b> CodedSeed<'b, C, Ctx, V>: DeserializeSeed<'de, Value = ()>, }
    Map<K, V> {}
}

impl_selector_seed_serde! { @selector_seed_codec_deseed
    { K: Representation + Clone + Ord + AsRef<str> + 'static,
      V: Representation + 'static }
    { for<'b> SelectorSeed<'b, Ctx, K>: CodecDeserializeSeed<'de>,
      for<'b> SelectorSeed<'b, Ctx, V>: CodecDeserializeSeed<'de>, }
    Map<K, V>
{
    #[inline]
    fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(CodecSeed::<C, _>(self))
    }
}}

impl_selector_seed_serde! { @selector_seed_select
    { K: Representation + Clone + Ord + AsRef<str> + 'static,
      V: Representation + 'static }
    { for<'b, 'de> SelectorSeed<'b, Ctx, K>: CodecDeserializeSeed<'de>,
      for<'b, 'de> SelectorSeed<'b, Ctx, V>: CodecDeserializeSeed<'de>, }
    Map<K, V>
}

impl<'a, const C: u64, Ctx, K, V> CodedSeed<'a, C, Ctx, Map<K, V>>
where
    Ctx: Context,
    K: Representation + Clone + Ord + AsRef<str> + 'static,
    V: Representation + 'static,
{
    ///
    pub(crate) fn match_map<'de, A>(mut self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        for<'b> CodedSeed<'b, C, Ctx, K>: DeserializeSeed<'de, Value = ()>,
        for<'b> CodedSeed<'b, C, Ctx, V>: DeserializeSeed<'de, Value = ()>,
    {
        let matcher = self
            .0
            .selector
            .as_matcher()
            .expect("should know that this is a matcher");

        let mode = self.0.mode();
        let dag: RefCell<Map<K, V>> = Default::default();

        match mode {
            SelectionMode::SelectNode => {
                self.0
                    .select_matched_node(SelectedNode::Map, matcher.label.as_deref())
                    .map_err(A::Error::custom)?;
            }
            SelectionMode::SelectDag => (),
            _ => unimplemented!(),
        }

        let (selector, state, mut params, ctx) = self.0.into_parts();

        // select against each child
        while let Some(key) = map.next_key::<K>()? {
            SelectorSeed::field_select_seed::<V>(
                selector,
                state,
                &mut params,
                ctx,
                key.as_ref().into(),
                match mode {
                    SelectionMode::SelectNode => None,
                    SelectionMode::SelectDag => Some(Box::new(|child, _| {
                        dag.borrow_mut().insert(key.clone(), child);
                        Ok(())
                    })),
                    _ => unreachable!(),
                },
            )
            .map_err(A::Error::custom)
            .and_then(|seed| map.next_value_seed(CodecSeed::<C, _>(seed)))?;

            state.ascend::<V>().map_err(A::Error::custom)?;
        }

        // finally, select the matched dag
        if mode == SelectionMode::SelectDag {
            let mut original_seed = SelectorSeed::from(selector, state, params, ctx);
            original_seed
                .select_matched_dag(dag.into_inner(), matcher.label.as_deref())
                .map_err(A::Error::custom)?;
        }

        Ok(())
    }

    ///
    pub(crate) fn explore_map_fields<'de, A>(self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        for<'b> CodedSeed<'b, C, Ctx, K>: DeserializeSeed<'de, Value = ()>,
        for<'b> CodedSeed<'b, C, Ctx, V>: DeserializeSeed<'de, Value = ()>,
    {
        unimplemented!()
    }

    ///
    pub(crate) fn explore_map_all<'de, A>(self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        for<'b> CodedSeed<'b, C, Ctx, K>: DeserializeSeed<'de, Value = ()>,
        for<'b> CodedSeed<'b, C, Ctx, V>: DeserializeSeed<'de, Value = ()>,
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
//
//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_map(self)
//     }
// }
//
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
