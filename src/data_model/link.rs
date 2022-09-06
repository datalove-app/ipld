use crate::dev::*;
use macros::{derive_more::From, impl_ipld_serde};
use std::fmt;

///
#[derive(Clone, Debug, Eq, From, Ord, PartialEq, PartialOrd)]
pub enum Link<T: Representation = Any> {
    ///
    Cid(Cid),

    ///
    #[from(ignore)]
    Inner { cid: Cid, t: T, dirty: bool },
}

impl<T: Representation> Link<T> {
    ///
    #[inline]
    pub const fn cid(&self) -> &Cid {
        match self {
            Self::Cid(inner) => inner,
            Self::Inner { cid, .. } => cid,
        }
    }

    /*
    ///
    #[inline]
    pub fn to_meta(&self) -> BlockMeta<'_, S> {
        self.cid().into()
    }

    ///
    #[inline]
    pub fn to_meta_prefix(&self) -> BlockMeta<'_, S> {
        let cid = self.cid();
        BlockMeta::from_prefix(cid.codec(), cid.hash().code(), None)
    }
     */
}

impl<T: Representation> Representation for Link<T> {
    const NAME: &'static str = "Link";
    const SCHEMA: &'static str = concat!("type Link &", stringify!(T::NAME));
    const DATA_MODEL_KIND: Kind = Kind::Link;
    const IS_LINK: bool = true;
    const HAS_LINKS: bool = true;

    fn name(&self) -> &'static str {
        match self {
            Self::Cid(_) => Self::NAME,
            Self::Inner { t, .. } => t.name(),
        }
    }

    fn has_links(&self) -> bool {
        match self {
            Self::Cid(_) => T::HAS_LINKS,
            Self::Inner { t, .. } => t.has_links(),
        }
    }
}

impl_ipld_serde! { @context_seed_visitor
    { T: Representation + 'static }
    { for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>, }
    Link<T>
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A link to a `{}`", T::NAME)
    }
}}

impl_ipld_serde! { @context_seed_visitor_ext
    { T: Representation + 'static }
    { for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>, }
    Link<T>
{
    #[inline]
    fn visit_link_str<E>(self, cid_str: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let cid = Cid::try_from(cid_str).map_err(E::custom)?;
        self.visit_link(cid)
    }

    #[inline]
    fn visit_link_bytes<E>(self, cid_bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let cid = Cid::try_from(cid_bytes).map_err(E::custom)?;
        self.visit_link(cid)
    }
}}

impl_ipld_serde! { @context_seed_deseed
    { T: Representation + 'static }
    { for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
    Link<T>
{
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_link(self)
    }
}}

impl_ipld_serde! { @context_seed_select
    { T: Representation + 'static }
    { for<'b, 'de> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
    Link<T>
}

impl<'a, C, T> ContextSeed<'a, C, Link<T>>
where
    C: Context,
    T: Representation + 'static,
{
    ///
    /// TODO: continue selection if the current selector is not a matcher
    fn visit_link<E>(mut self, cid: Cid) -> Result<(), E>
    where
        E: de::Error,
    {
        if let Some(matcher) = self.selector.as_matcher() {
            return Ok(match self.mode() {
                SelectionMode::SelectNode => {
                    self.select_matched_node(cid.into(), matcher.label.as_deref())
                        .map_err(E::custom)?;
                }
                SelectionMode::SelectDag => {
                    self.select_matched_dag(Link::Cid(cid), matcher.label.as_deref())
                        .map_err(E::custom)?;
                }
                _ => unimplemented!(),
            });
        }

        unimplemented!()
    }
}

// impl<'a, 'de, C, T> Visitor<'de> for ContextSeed<'a, C, Link<T>>
// where
//     C: Context,
//     T: Representation + 'static,
//     for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     type Value = ();
//
//     #[inline]
//     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(formatter, "{}", Link::<T>::NAME)
//     }
// }

// impl<'a, 'de, C, T> IpldVisitorExt<'de> for ContextSeed<'a, C, Link<T>>
// where
//     C: Context,
//     T: Representation + 'static,
//     for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     // TODO:
// }

// impl<'a, 'de, C, T> DeserializeSeed<'de> for ContextSeed<'a, C, Link<T>>
// where
//     C: Context,
//     T: Representation + 'static,
//     for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     type Value = ();
//
//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_link(self)
//     }
// }

// impl<'a, 'de, C, T> Select<C> for Link<T>
// where
//     C: Context,
//     T: Representation + Send + Sync + 'static,
//     // ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
// }

impl<T: Representation> Into<Cid> for Link<T> {
    fn into(self) -> Cid {
        match self {
            Self::Cid(cid) => cid,
            Self::Inner { cid, .. } => cid,
        }
    }
}

// TODO dirty links?
impl<T> Serialize for Link<T>
where
    T: Representation,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <S as Encoder>::serialize_link(serializer, self.cid())
    }
}

impl<'de, T> Deserialize<'de> for Link<T>
where
    T: Representation,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::Cid(Cid::deserialize(deserializer)?))
    }
}

////////////////////////////////////////////////////////////////////////////////
// additional implementations
////////////////////////////////////////////////////////////////////////////////

// impl<const SI: usize, const SO: usize, T> From<CidGeneric<SI>> for Link<SO, T>
// where
//     T: Representation,
// {
//     fn from(cid: CidGeneric<SI>) -> Self {
//         Self::Cid(Cid::Generic(cid))
//     }
// }

// impl<const S: usize, T> From<Link<T>> for CidGeneric<S>
// where
//     T: Representation,
// {
//     fn from(link: Link<T>) -> Self {
//         match link {
//             Link::Cid(Cid::Generic(inner)) => inner,
//             Link::Inner { cid, .. } => cid,
//         }
//     }
// }
