use crate::dev::*;
use macros::{derive_more::From, repr_serde};
use maybestd::fmt;

///
#[derive(
    Copy,
    Clone,
    Debug,
    Eq, // todo
    From,
    // Hash, Ord,
    PartialEq, // todo
               // PartialOrd
)]
pub enum Link<T: Representation = Any> {
    ///
    Cid(Cid),

    ///
    #[from(ignore)]
    Inner {
        ///
        cid: Cid,
        ///
        t: T,
        ///
        dirty: bool,
    },
}

impl<T: Representation> Link<T> {
    ///
    #[inline]
    pub const fn cid(&self) -> &Cid {
        match self {
            Self::Cid(cid) => cid,
            Self::Inner { cid, .. } => cid,
        }
    }

    ///
    #[inline]
    pub const fn is_dirty(&self) -> bool {
        match self {
            Self::Cid(_) => false,
            Self::Inner { dirty, .. } => *dirty,
        }
    }

    ///
    #[inline]
    pub const fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Cid(_) => None,
            Self::Inner { t, .. } => Some(t),
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
    type ReprKind = type_kinds::Link;

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
    #[inline]
    fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.is_dirty() {
            Err(S::Error::custom(
                "cannot serialize dirty links; flush changes first",
            ))
        } else {
            Representation::serialize::<C, _>(self.cid(), serializer)
        }
    }

    ///
    #[inline]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::Cid(Cid::deserialize::<C, _>(deserializer)?))
    }
}

repr_serde! { @visitor S T { type_kinds::Link } { S, T }
    { S: 'static, T: Select<Ctx> + 'static }
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A link of type {} to a {}", S::NAME, T::NAME)
    }
}}

// ? impl From<T>?
repr_serde! { @visitor_ext S T { type_kinds::Link } { S, T }
    { S: 'static, T: Select<Ctx> + 'static }
{
    #[inline]
    fn visit_cid<E>(self, cid: Cid) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // self.0.select_link::<C>(cid).map_err(E::custom)
        unimplemented!()
    }
}}

repr_serde! { @select Link<T> => T { T } { T: Select<Ctx> + 'static } }

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, Link<T>>
where
    Ctx: Context,
    T: Select<Ctx> + 'static,
{
    fn select_link<'de, const C: u64>(mut self, cid: Cid) -> Result<(), Error> {
        // TODO: handle "blocks encoded with rawa codec are valid Bytes kinds"

        if self.selector.is_matcher() {
            if self.is_dag_select() {
                self.select_dag(Link::Cid(cid))?;
            } else {
                self.select_node(cid.into())?;
            }

            return Ok(());
        }

        /// TODO: continue selection if the current selector is not a matcher
        unimplemented!()
    }
}

impl<T: Representation> Into<Cid> for Link<T> {
    fn into(self) -> Cid {
        match self {
            Self::Cid(cid) => cid,
            Self::Inner { cid, .. } => cid,
        }
    }
}

// // TODO: dirty links?
// impl<T> Serialize for Link<T>
// where
//     T: Representation,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         // <S as Encoder>::serialize_link(serializer, self.cid())
//         // (&mut &mut &mut Encoder(serializer)).serialize_link(self.cid())
//         // self.cid().serialize(serializer)
//         Serialize::serialize(self.cid(), serializer)
//     }
// }

// impl<'de, T> Deserialize<'de> for Link<T>
// where
//     T: Representation,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // Ok(Self::Cid(Cid::deserialize(deserializer)?))
//         Ok(Self::Cid(Deserialize::deserialize(deserializer)?))
//     }
// }

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
