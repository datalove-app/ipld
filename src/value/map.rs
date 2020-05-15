use crate::dev::*;
use std::collections::BTreeMap;

pub type Map<K, V> = BTreeMap<K, V>;

// TODO: write the 4 Select impls, then the latter 3 for BTreeMap<K, Link<V>>

// impl<K: Representation, V: Representation> Representation for BTreeMap<K, V> {
//     const NAME: &'static str = "Map";
//     // const SCHEMA: &'static str = format!("type {} = &{}", Self::NAME, T::NAME);
//     // const KIND: Kind = Kind::Map;
// }

// // TODO: add the rest of the selectors
// impl_root_select!(Matcher, ExploreAll, ExploreFields, ExploreRange, {
//     default impl<Ctx, K, V> Select<Selector, Ctx> for BTreeMap<K, V>
//     where
//         Ctx: Context,
//         K: Representation + 'static,
//         V: Representation + 'static,
// });
