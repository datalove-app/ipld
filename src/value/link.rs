use crate::dev::*;
use macros::derive_more::Into;
use std::{convert::TryFrom, marker::PhantomData, rc::Rc};

///
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Link<T: Representation = Value, Si: MultihashSize = DefaultMultihashSize> {
    Cid(CidGeneric<Si>),
    Inner {
        cid: CidGeneric<Si>,
        t: T,
        dirty: bool,
    },
}

impl<T: Representation, Si: MultihashSize> Link<T, Si> {
    ///
    #[cfg(feature = "multicodec")]
    #[inline]
    pub fn multicodec(&self) -> Result<Multicodec, Error> {
        let cid = self.cid();
        Multicodec::try_from(cid.codec())
    }

    ///
    #[inline]
    pub fn multihash(&self) -> Result<Multihash, Error> {
        let cid = self.cid();
        Ok(Multihash::try_from(cid.hash().code())?)
    }

    ///
    #[inline]
    pub const fn cid(&self) -> &CidGeneric<Si> {
        match self {
            Self::Cid(inner) => inner,
            Self::Inner { cid, .. } => cid,
        }
    }

    ///
    #[inline]
    pub fn to_meta(&self) -> BlockMeta<'_, Si> {
        let cid = self.cid();
        cid.into()
    }

    ///
    #[inline]
    pub fn to_meta_prefix(&self) -> BlockMeta<'_, Si> {
        let cid = self.cid();
        BlockMeta::from_prefix(cid.codec(), cid.hash().code(), None)
    }
}

impl<T: Representation, Si: MultihashSize> Representation for Link<T, Si> {
    const NAME: &'static str = concat!("Link<", stringify!(T::NAME), ">");
    const SCHEMA: &'static str = concat!("type", stringify!(Self::NAME), " ", stringify!(T::NAME));
    const KIND: Kind = Kind::Link;
    const IS_LINK: bool = true;
    const HAS_LINKS: bool = true;

    fn name(&self) -> &'static str {
        match self {
            Self::Cid(_) => Self::NAME,
            Self::Inner { t, .. } => t.name(),
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Self::Cid(_) => Self::KIND,
            Self::Inner { t, .. } => t.kind(),
        }
    }

    ///
    fn has_links(&self) -> bool {
        match self {
            Self::Cid(_) => T::HAS_LINKS,
            Self::Inner { t, .. } => t.has_links(),
        }
    }
}

// impl<'de, 'a, C: Context, T: Representation, U: Representation, Si: MultihashSize> Visitor<'de>
//     for ContextSeed<'a, C, Link<T, Si>, U>
// where
//     T: Send + Sync + 'static,
//     U: 'static,
//     // ContextSeed<'a, &'a mut C, Link<T, Si>, T>: DeserializeSeed<'de, Value = Option<T>>,
//     // ContextSeed<'a, C, T, T>: DeserializeSeed<'de, Value = Option<T>>,
//     // ContextSeed<'a, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
// {
//     type Value = Option<U>;

//     #[inline]
//     fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(formatter, "{}", Link::<T, Si>::NAME)
//     }
// }

// impl<'de, 'a, C: Context, T: Representation, U: Representation, Si: MultihashSize>
//     IpldVisitorExt<'de> for ContextSeed<'a, C, Link<T, Si>, U>
// where
//     T: Send + Sync + 'static,
//     U: 'static,
//     // ContextSeed<'a, &'a mut C, Link<T, Si>, T>: DeserializeSeed<'de, Value = Option<T>>,
//     // ContextSeed<'a, C, T, T>: DeserializeSeed<'de, Value = Option<T>>,
//     // ContextSeed<'a, C, T, U>: DeserializeSeed<'de, Value = Option<U>>,
// {
// }

// impl<'de, 'a, C: Context, T: Representation, U: Representation, Si: MultihashSize>
//     DeserializeSeed<'de> for ContextSeed<'a, C, Link<T, Si>, U>
// where
// // ContextSeed<'a, C, Link<T, Si>, U>: Visitor<'de, Value = Option<U>>,
// // ContextSeed<'a, C, Link<T, Si>, U>: IpldVisitorExt<'de, Value = Option<U>>,
// {
//     type Value = Option<U>;

//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         <D as Decoder<'de>>::deserialize_link(deserializer, self)
//     }
// }

// mod impl_self {
//     use crate::dev::*;
//     use serde::de;

//     impl_ipld! { @visitor
//         {T: Representation, Si: MultihashSize} {}
//         Link<T, Si> => Link<T, Si>
//     {
//         #[inline]
//         fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             write!(formatter, "A link to a {}", T::NAME)
//         }

//         // #[inline]
//         // fn $visit_fn<E>(self, $visit_arg : $visit_ty) -> Result<Self::Value, E>
//         // where
//         //     E: serde::de::Error,
//         // {
//         //     let Self { selector, state, .. } = self;
//         //     match selector {
//         //         Selector::Matcher(Matcher { label, .. }) => {
//         //             state.add_matched($visit_arg.into(), label.clone())
//         //                 .map_err(E::custom)?;
//         //             Ok(Some($visit_arg))
//         //         },
//         //         selector => Err(Error::unsupported_selector::<String>(selector)).map_err(E::custom)
//         //     }
//         // }
//     }}

//     impl_ipld! { @visitor_ext
//         {T: Representation, Si: MultihashSize} {}
//         Link<T, Si> => Link<T, Si>
//     {
//         #[inline]
//         fn visit_link_str<E>(self, cid_str: &'de str) -> Result<Self::Value, E>
//         where
//             E: de::Error,
//         {
//             let cid = CidGeneric::<Si>::try_from(cid_str).map_err(E::custom)?;
//             Ok(Some(Link::<T, Si>::from(cid)))
//         }

//         #[inline]
//         fn visit_link_bytes<E>(self, cid_bytes: &'de [u8]) -> Result<Self::Value, E>
//         where
//             E: de::Error,
//         {
//             let cid = CidGeneric::<Si>::try_from(cid_bytes).map_err(E::custom)?;
//             Ok(Some(Link::<T, Si>::from(cid)))
//         }
//     }}

//     impl_ipld! { @deseed
//         {T: Representation, Si: MultihashSize} {}
//         Link<T, Si> => Link<T, Si>
//     {
//         #[inline]
//         fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//         where
//             D: Deserializer<'de>,
//         {
//             deserializer.deserialize_link(self)
//         }
//     }}

//     impl_ipld! { @select_self
//         {T: Representation, Si: MultihashSize} {}
//         Link<T, Si>
//     {
//         #[inline]
//         fn select<U: Select<C>>(
//             selector: &Selector,
//             state: SelectorState,
//             ctx: &mut C,
//         ) -> Result<Option<U>, Error>
//         {
//             // primitive_select::<'_, '_, C, $native_ty>(selector, state, ctx)
//             unimplemented!()
//         }
//     //
//     //     fn patch(
//     //         &mut self,
//     //         selector: &Selector,
//     //         state: SelectorState,
//     //         dag: Self,
//     //         ctx: &mut C,
//     //     ) -> Result<(), Error>
//     //     {
//     //         // primitive_patch::<C, $native_ty>(self, selector, state, dag, ctx)
//     //         unimplemented!()
//     //     }
//     }}
// }

#[cfg(feature = "skipped")]
mod impl_generic {
    use crate::dev::*;

    impl<C, T, U, Si> Select<C, U> for Link<T, Si>
    where
        C: Context,
        T: Representation,
        // T: Representation + Select<C, U>,
        U: Representation + Select<C, U>,
        Si: MultihashSize,
    {
        default fn select(
            selector: &Selector,
            state: SelectorState,
            ctx: &mut C,
        ) -> Result<Option<U>, Error> {
            // primitive_select::<'_, '_, C, Self>(selector, state, ctx)
            unimplemented!()
        }

        default fn patch(
            &mut self,
            selector: &Selector,
            state: SelectorState,
            dag: U,
            ctx: &mut C,
        ) -> Result<(), Error> {
            // primitive_patch::<C, List<T>>(self, selector, state, dag, ctx)
            unimplemented!()
        }
    }
}

// /// Link type, used to switch between a `cid::CidGeneric` and it's underlying dag.
// ///
// /// Under the hood, `Link` uses a `std::cell::Cell` in order to load links while
// /// reading into the dag.
// /// TODO: impl Serialize for Link, checking if impls!(S: Encoder)
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct Link<T = Box<Value>, Si = DefaultMultihashSize>(InnerLink<T, Si>)
// where
//     T: Representation,
//     Si: MultihashSize;

// #[derive(Clone, Debug, Eq, PartialEq)]
// enum InnerLink<T, Si = DefaultMultihashSize>
// where
//     T: Representation,
//     Si: MultihashSize,
// {
//     /// Represents a raw `cid::CidGeneric` contained within a dag.
//     Cid(CidGeneric<Si>),

//     /// Represents a parsed subset of a dag and its original `cid::CidGeneric`.
//     Selection {
//         cid: CidGeneric<Si>,
//         // selector: Rc<Selector>,
//         dag: T,
//     },
// }

// TODO: impl_root_selector for each IS => T: Select<IS> (Ctx: Block for recursive)
// TODO: ? impl Select<IS> for each IS (Ctx: Block for recursive)

// // TODO: write the Select impls, then the latter 3 for Vec<Link<T>>
// impl_root_select!(
//     Matcher, ExploreAll, ExploreFields, ExploreIndex,
//     ExploreRange, ExploreRecursive, ExploreConditional, ExploreRecursiveEdge {
//     default impl<Ctx, T> Select<Selector, Ctx> for Link<T>
//     where
//         Ctx: Context,
//         T: Representation + 'static
// });

// impl<Ctx, S, T> Select<Ctx, S> for Link<T>
// where
//     Ctx: Context,
//     S: ISelector,
//     T: Select<Ctx, S>,
// {
//     type Output = <T as Select<Ctx, S>>::Output;

//     fn select<'a>(self, selector: &S, executor: &Executor<'a, Ctx>) -> Result<Self::Output, ()> {
//         match self {
//             Self::Cid(_) => Err(()),
//             Self::Full { dag, .. } | Self::Selection { dag, .. } => {
//                 T::select(dag, s, executor)
//             }
//         }
//     }
// }

// #[async_trait]
// impl<R, W, T> Representation<R, W> for Link<T>
// where
//     R: Read + Unpin + Send,
//     W: Write + Unpin + Send,
//     T: Representation<R, W> + Sync,
// {
//     #[inline]
//     default async fn read<C>(ctx: &mut C) -> Result<Self, Error>
//     where
//         R: 'async_trait,
//         W: 'async_trait,
//         C: Context<R, W> + Send,
//     {
//         let cid = Cid::read(ctx).await?;
//         if ctx.try_apply(ResolveBlock::new(&cid)).await {
//             let dag = T::read(ctx).await?;
//             Ok(Link::Dag(cid, dag))
//         } else {
//             Ok(Link::Cid(cid))
//         }
//     }

//     #[inline]
//     default async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
//     where
//         R: 'async_trait,
//         W: 'async_trait,
//         C: Context<R, W> + Send,
//     {
//         match self {
//             Link::Cid(cid) => {
//                 Cid::write(cid, ctx).await?;
//                 Ok(())
//             }
//             Link::Dag(old_cid, dag) => {
//                 if ctx.try_apply(ResolveBlock::new(&old_cid)).await {
//                     T::write(dag, ctx).await?;
//                     let cid = ctx.try_apply(FlushBlock::new(&old_cid)).await?;
//                     Cid::write(&cid, ctx).await?;
//                 } else {
//                     Cid::write(old_cid, ctx).await?;
//                 }
//                 Ok(())
//             }
//         }
//     }
// }

// impl<T: Representation> RepresentationExt<T> for Link<T> {
//     // fn codec(&self) ->

//     // /// resolves a link into it's full underlying dag T
//     // ///
//     // /// when a Link::Cid is focused, its cloned, fully/partially resolved, replaces itself with T, then delegates to T::focus
//     // /// when a Link::Dag is focused, it delegates to T::focus
//     // /// when a Link::Selection is focused, panic/error?
//     // ///
//     // ///
//     // /// when a Link::Cid is resolved, it deserializes T against the Executor
//     // /// when a Link::Dag is resolved, it returns T
//     // /// when a Link::Selection is resolved, panic/error?
//     // /// TODO: FromContext
//     // async fn resolve<'a>(self, executor: &'a Executor<'a, Ctx>) -> Result<T, ()> {
//     //     unimplemented!()
//     // }

//     //
//     //
//     //
//     //
//     //

//     // /// resolves a link against a selector into a selection of dag T
//     // /// TODO? FromContext
//     // async fn resolve_selector<'a>(
//     //     self,
//     //     selector: &Selector,
//     //     executor: &'a Executor<'a, CtxT>,
//     // ) -> Result<T, ()>
//     // where
//     //     Selector: Visitor<'a, Value = T>,
//     //     T: Representation<CtxT>,
//     // {
//     //     unimplemented!()
//     // }
// }

////////////////////////////////////////////////////////////////////////////////
// additional implementations
////////////////////////////////////////////////////////////////////////////////

impl<T, Si> From<CidGeneric<Si>> for Link<T, Si>
where
    T: Representation,
    Si: MultihashSize,
{
    fn from(cid: CidGeneric<Si>) -> Self {
        Self::Cid(cid)
    }
}

impl<T, Si> From<Link<T, Si>> for CidGeneric<Si>
where
    T: Representation,
    Si: MultihashSize,
{
    fn from(link: Link<T, Si>) -> Self {
        match link {
            Link::Cid(inner) => inner,
            Link::Inner { cid, .. } => cid,
        }
    }
}

impl<T, Si> Serialize for Link<T, Si>
where
    T: Representation,
    Si: MultihashSize,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        <Se as Encoder>::serialize_link(serializer, self.cid())
    }
}

impl<'de, T, Si> Deserialize<'de> for Link<T, Si>
where
    T: Representation,
    Si: MultihashSize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LinkVisitor<T, Si: MultihashSize = DefaultMultihashSize>(
            CidVisitor<Si>,
            PhantomData<T>,
        );

        impl<'de, T, Si> Visitor<'de> for LinkVisitor<T, Si>
        where
            T: Representation,
            Si: MultihashSize,
        {
            type Value = Link<T, Si>;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an IPLD link")
            }
        }

        impl<'de, T, Si> IpldVisitorExt<'de> for LinkVisitor<T, Si>
        where
            T: Representation,
            Si: MultihashSize,
        {
            #[inline]
            fn visit_link_str<E>(self, cid_str: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(self.0.visit_link_str(cid_str)?.into())
            }

            #[inline]
            fn visit_link_bytes<E>(self, cid_bytes: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(self.0.visit_link_bytes(cid_bytes)?.into())
            }
        }

        <D as Decoder<'de>>::deserialize_link(
            deserializer,
            LinkVisitor(Default::default(), PhantomData),
        )
    }
}

// impl<'de, C, H> Visitor<'de> for CidGeneric<C, H>
// where
//     C: Into<u64> + TryFrom<u64> + Copy,
//     <C as TryFrom<u64>>::Error: Debug,
//     H: Into<u64> + TryFrom<u64> + Copy,
//     <H as TryFrom<u64>>::Error: Debug,
// {
// }

// impl<'de, C, H> IpldVisitorExt<'de> for CidGeneric<C, H>
// where
//     C: Into<u64> + TryFrom<u64> + Copy,
//     <C as TryFrom<u64>>::Error: Debug,
//     H: Into<u64> + TryFrom<u64> + Copy,
//     <H as TryFrom<u64>>::Error: Debug,
// {
// }
