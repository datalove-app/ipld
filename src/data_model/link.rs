use crate::dev::*;
use macros::{derive_more::From, repr_serde};
use maybestd::fmt;

///
pub trait LinkRepresentation: Representation {
    // const_assert <Self as Representation>::{DM_KIND, SCHEMA_KIND} == Link
}
// TODO
impl LinkRepresentation for Cid {}

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
pub enum Link<T: Representation = Any, I: LinkRepresentation = Cid> {
    ///
    Id(I),

    ///
    #[from(ignore)]
    Resolved {
        ///
        id: I,
        ///
        t: T,
        ///
        dirty: bool,
    },
}

impl<T: Representation, I: LinkRepresentation> Link<T, I> {
    ///
    #[inline]
    pub const fn id(&self) -> &I {
        match self {
            Self::Id(id) => id,
            Self::Resolved { id, .. } => id,
        }
    }

    ///
    #[inline]
    pub const fn is_dirty(&self) -> bool {
        match self {
            Self::Id(_) => false,
            Self::Resolved { dirty, .. } => *dirty,
        }
    }

    ///
    #[inline]
    pub const fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Id(_) => None,
            Self::Resolved { t, .. } => Some(t),
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

// TODO: restrict to some I
impl<T: Representation> Representation for Link<T> {
    const NAME: &'static str = "Link";
    const SCHEMA: &'static str = "type Link &Any";
    const DATA_MODEL_KIND: Kind = Kind::Link;

    fn name(&self) -> &'static str {
        match self {
            Self::Id(_) => Self::NAME,
            Self::Resolved { t, .. } => t.name(),
        }
    }

    fn has_links(&self) -> bool {
        match self {
            Self::Id(_) => T::HAS_LINKS,
            Self::Resolved { t, .. } => t.has_links(),
        }
    }

    fn to_selected_node(&self) -> SelectedNode {
        SelectedNode::Link(*self.id())
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
            Representation::serialize::<C, _>(self.id(), serializer)
        }
    }

    ///
    #[inline]
    fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::Id(Cid::deserialize::<C, _>(deserializer)?))
    }
}

repr_serde! { @select for Link<T> { T } { T: Select<Ctx> + 'static }}
repr_serde! { @visitors for Link<T> { T } { T: Select<Ctx> + 'static }
    @serde {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "A link of type {} to a {}", <Link<T>>::NAME, T::NAME)
        }
        #[inline]
        fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_link_bytes(bytes)
        }
        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_link_str(s)
        }
    }
    @link {
        #[inline]
        fn visit_cid<E>(self, cid: Cid) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // self.0.select_link::<C>(cid).map_err(E::custom)
            unimplemented!()
        }
    }
}

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, Link<T>>
where
    Ctx: Context,
    T: Select<Ctx> + 'static,
{
    fn select_link<'de, const C: u64>(mut self, cid: Cid) -> Result<(), Error> {
        // TODO: handle "blocks encoded with rawa codec are valid Bytes kinds"

        if self.selector.is_matcher() {
            if self.is_dag_select() {
                self.handle_dag(Link::Id(cid))?;
            } else {
                self.handle_node(cid.into())?;
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
            Self::Id(cid) => cid,
            Self::Resolved { id: cid, .. } => cid,
        }
    }
}

#[cfg(feature = "dep:rkyv")]
mod rkyv {}

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
//             Link::Resolved { cid, .. } => cid,
//         }
//     }
// }
