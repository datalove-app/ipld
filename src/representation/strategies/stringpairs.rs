use crate::dev::*;
use maybestd::{borrow::Cow, fmt, str::FromStr};

const STRATEGY: Strategy = Strategy::StringPairs;

/*
// Blanket impl for maps.
repr_serde! { @visitors for T => (K, V)
    { @dk (type_kinds::Map) @sk (type_kinds::Map) @rk (type_kinds::String) }
    { T, K, V } { T: Default + Extend<(K, V)> +  'static,
                  K: Select<Ctx> + StringRepresentation + 'static,
                  <K as FromStr>::Err: fmt::Display,
                  V: Select<Ctx> + 'static } @serde {
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A list of type {} of {}", S::NAME, T::NAME)
    }
}
 */

/*
// Blanket impl for structs.
macro_rules! stringpairs {
    () => {
        repr_serde! { @visitors for T => (K, V)
            { @dk (type_kinds::Map) @sk (type_kinds::Map) @rk (type_kinds::String) }
            { T, K, V } { T: Default + Extend<(K, V)> +  'static,
                          K: Select<Ctx> + StringRepresentation + 'static,
                          <K as FromStr>::Err: fmt::Display,
                          V: Select<Ctx> + 'static } @serde {
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A list of type {} of {}", S::NAME, T::NAME)
            }
        }
    }
}
 */

/*
///
#[derive(Debug)]
pub struct StringPairsMap<const I: char, const E: char, K, V>(pub Map<K, V>)
where
    K: StringRepresentation,
    V: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    <V as FromStr>::Err: fmt::Display;

// type StringPair<K, V> = Tuple2<{ Strategy::Tuple as u8 }, (K, V), K, V>;
// type StringPairRef<'a, K, V> = TupleRef2<'a, { Strategy::Tuple as u8 }, (K, V), K, V>;

impl<const I: char, const E: char, K, V> Default for StringPairsMap<I, E, K, V>
where
    K: StringRepresentation,
    V: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    <V as FromStr>::Err: fmt::Display,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

// TODO: impl Display for StringPairsIter
impl<const I: char, const E: char, K, V> fmt::Display for StringPairsMap<I, E, K, V>
where
    K: StringRepresentation,
    V: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    <V as FromStr>::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last_idx = self.0.len() - 1;
        for (idx, (key, val)) in self.0.iter().enumerate() {
            key.fmt(f)?;
            I.fmt(f)?;
            val.fmt(f)?;
            if idx < last_idx {
                E.fmt(f)?;
            }
        }

        Ok(())
    }
}

impl<const I: char, const E: char, K, V> FromStr for StringPairsMap<I, E, K, V>
where
    K: StringRepresentation,
    V: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    <V as FromStr>::Err: fmt::Display,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl<const I: char, const E: char, K, V> Representation for StringPairsMap<I, E, K, V>
where
    // TODO: remove clone requirement by switching up callbacks
    K: StringRepresentation + AsRef<str> + Clone + Ord,
    V: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    <V as FromStr>::Err: fmt::Display,
{
    type DataModelKind = type_kinds::Map;
    type SchemaKind = type_kinds::Map;
    type ReprKind = type_kinds::String;

    const NAME: &'static str = "Map";
    const SCHEMA: &'static str = concat!(
        "type Map {",
        stringify!(K::NAME),
        ":",
        stringify!(V::NAME),
        "} representation stringpairs {",
        "}",
    );
    const DATA_MODEL_KIND: Kind = Kind::Map;
    // const REPR_KIND: Kind = Kind::List;

    fn has_links(&self) -> bool {
        self.0.iter().any(|(k, v)| k.has_links() || v.has_links())
    }

    #[inline]
    #[doc(hidden)]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use ser::SerializeSeq;

        // let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        // for listpair in self.0.iter().map(StringPairRef::<K, V>::from) {
        //     seq.serialize_element(&SerializeWrapper::<'_, C, _>(&listpair))?;
        // }
        // seq.end()
        unimplemented!()
    }

    #[inline]
    #[doc(hidden)]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /*
        struct StringPairsMapVisitor<const C: u64, const I: char, const E: char, K, V>(
            PhantomData<(K, V)>,
        );
        impl<'de, const C: u64, const I: char, const E: char, K, V> Visitor<'de>
            for StringPairsMapVisitor<C, I, E, K, V>
        where
            K: StringRepresentation + AsRef<str> + Ord,
            V: StringRepresentation,
            <K as FromStr>::Err: fmt::Display,
            <V as FromStr>::Err: fmt::Display,
        {
            type Value = StringPairsMap<I, E, K, V>;
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A map of `{}` to `{}` listpairs", K::NAME, V::NAME)
            }
            #[inline]
            fn visit_str<Er: de::Error>(self, s: &str) -> Result<Self::Value, Er> {
                let mut map = StringPairsMap::default();

                // TODO: move this to StringPairsIter
                while let Some(pair) = s.split(E).map(|s| s.split_once(I)).next() {
                    let (key_str, val_str) = pair.ok_or_else(|| Er::custom("missing pair"))?;

                    let key = K::from_str(key_str).map_err(Er::custom)?;
                    let val = V::from_str(val_str).map_err(Er::custom)?;

                    map.0.insert(key, val);
                }

                Ok(map)
            }
        }

        deserializer.deserialize_str(StringPairsMapVisitor::<C, I, E, K, V>(PhantomData))
         */

        let s = Cow::<'_, str>::deserialize(deserializer)?;
        Self::from_str(s.as_ref()).map_err(D::Error::custom)
    }
}
 */
