use crate::dev::*;
use macros::derive_more::From;

///
#[derive(Clone, Debug, Eq, From, PartialEq)]
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

impl<T: Representation> Into<Cid> for Link<T> {
    fn into(self) -> Cid {
        match self {
            Self::Cid(cid) => cid,
            Self::Inner { cid, .. } => cid,
        }
    }
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

// impl_ipld_serde! { @context_visitor
//     { T: Representation + 'static }
//     { for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>, }
//     Link<T>
// {
//     #[inline]
//     fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", Link::<T>::NAME)
//     }
// }}

// impl<'a, 'de, C, T> Visitor<'de> for ContextSeed<'a, C, Link<T>>
// where
//     C: Context,
//     T: Representation + 'static,
//     for<'b> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     type Value = ();

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

//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_link(self)
//     }
// }

// impl_ipld_serde! { @context_select
//     { T: Representation + Send + Sync + 'static }
//     { for<'b, 'de> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
//     List<T>
// }

// TODO dirty links?
impl<T> Serialize for Link<T>
where
    T: Representation,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        <Se as Encoder>::serialize_link(serializer, self.cid())
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

// impl_ipld_serde! { @select_with_seed
//     { T: Representation + Send + Sync + 'static }
//     { for<'b, 'de> ContextSeed<'b, C, T>: DeserializeSeed<'de, Value = ()> }
//     Link<T>
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
