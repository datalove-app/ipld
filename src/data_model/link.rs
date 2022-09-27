use crate::dev::*;
use macros::{derive_more::From, impl_selector_seed_serde};
use std::fmt;

///
#[derive(
    Clone,
    Debug,
    Eq,
    From,
    // Hash, Ord,
    PartialEq,
    // PartialOrd
)]
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

    ///
    /// TODO: dirty links?
    #[inline]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Representation::serialize::<C, _>(self.cid(), serializer)
    }

    ///
    #[inline]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::Cid(Representation::deserialize::<C, _>(
            deserializer,
        )?))
    }
}

impl_selector_seed_serde! { @codec_seed_visitor
    { T: Select<Ctx> + 'static }
    // { for<'b> CodedSelectorSeed<'b, _C, _D, Ctx, T>: DeserializeSeed<'de, Value = ()>, }
    { }
    Link<T>
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A link to a `{}`", T::NAME)
    }
}}

impl_selector_seed_serde! { @codec_seed_visitor_ext
    { T: Select<Ctx> + 'static }
    // { for<'b> CodedSelectorSeed<'b, _C, _D, Ctx, T>: DeserializeSeed<'de, Value = ()>, }
    { }
    Link<T>
{
    #[inline]
    fn visit_cid<E>(self, cid: Cid) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_link(cid)
    }
}}

impl_selector_seed_serde! { @selector_seed_codec_deseed
    { T: Select<Ctx> + 'static }
    // { for<'b> SelectorSeed<'b, Ctx, T>: CodecDeserializeSeed<'de, Value = ()> }
    { }
    Link<T>
{
    // #[inline]
    // fn deserialize<const C: u64, D>(self, deserializer: D) -> Result<(), D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     cfg_if::cfg_if! {
    //         if #[cfg(feature = "dag-json")] {
    //             if C == DagJson::CODE {
    //                 return DagJson::deserialize_cid::<'de, D, _>(deserializer, CodecSeed::<C, false, _>(self));
    //             }
    //         }
    //     }
    //     cfg_if::cfg_if!{
    //         if #[cfg(feature = "dag-cbor")] {
    //             if C == DagCbor::CODE {
    //                 return DagCbor::deserialize_cid::<'de, D, _>(deserializer, CodecSeed::<C, false, _>(self));
    //             }
    //         }
    //     }

    //     Deserialize::deserialize(deserializer)
    // }
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        cfg_if::cfg_if! {
            if #[cfg(feature = "dag-json")] {
                if _C == DagJson::CODE {
                    return DagJson::deserialize_cid(deserializer, self);
                }
            }
        }
        cfg_if::cfg_if!{
            if #[cfg(feature = "dag-cbor")] {
                if _C == DagCbor::CODE {
                    return DagCbor::deserialize_cid(deserializer, self);
                }
            }
        }

        Deserialize::deserialize(deserializer)
    }
}}

impl_selector_seed_serde! { @selector_seed_select
    { T: Select<Ctx> + 'static }
    // { for<'b, 'de> SelectorSeed<'b, Ctx, T>: CodecDeserializeSeed<'de, Value = ()> }
    { }
    Link<T>
}

impl<'a, const C: u64, const D: bool, Ctx, T> CodedSelectorSeed<'a, C, D, Ctx, Link<T>>
where
    Ctx: Context,
    T: Select<Ctx> + 'static,
{
    fn visit_link<'de, E>(mut self, cid: Cid) -> Result<(), E>
    where
        E: de::Error,
        // for<'b> CodedSelectorSeed<'b, C, D, Ctx, T>: DeserializeSeed<'de, Value = ()>,
    {
        if let Some(matcher) = self.0.selector.as_matcher() {
            match self.0.mode() {
                SelectionMode::SelectNode => {
                    self.0
                        .select_matched_node(cid.into(), matcher.label.as_deref())
                        .map_err(E::custom)?;
                }
                SelectionMode::SelectDag => {
                    self.0
                        .select_matched_dag(Link::Cid(cid), matcher.label.as_deref())
                        .map_err(E::custom)?;
                }
                _ => unimplemented!(),
            };

            return Ok(());
        }

        /// TODO: continue selection if the current selector is not a matcher
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

// TODO: dirty links?
impl<T> Serialize for Link<T>
where
    T: Representation,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // <S as Encoder>::serialize_link(serializer, self.cid())
        // (&mut &mut &mut Encoder(serializer)).serialize_link(self.cid())
        // self.cid().serialize(serializer)
        Serialize::serialize(self.cid(), serializer)
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
        // Ok(Self::Cid(Cid::deserialize(deserializer)?))
        Ok(Self::Cid(Deserialize::deserialize(deserializer)?))
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
