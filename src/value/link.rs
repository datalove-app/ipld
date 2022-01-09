use crate::dev::*;
use std::{convert::TryFrom, marker::PhantomData, rc::Rc};

///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Link<T: Representation = Value, Si: MultihashSize = DefaultMultihashSize> {
    cid: CidGeneric<Si>,
    t: PhantomData<T>,
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

impl<T, Si> Representation for Link<T, Si>
where
    T: Representation,
    Si: MultihashSize,
{
    const NAME: &'static str = "Link";
    // const SCHEMA: &'static str = "type" + Self::NAME + " = " + T::NAME;
    const KIND: Kind = Kind::Link;
}

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
        Link {
            cid,
            t: PhantomData,
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
        // match &self.0 {
        //     InnerLink::Cid(cid) => <Se as Encoder>::serialize_link(serializer, cid),
        //     _ => Err(<Se::Error as serde::ser::Error>::custom(
        //         "cannot serialize IPLD selection",
        //     )),
        // }

        <Se as Encoder>::serialize_link(serializer, &self.cid)
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
            PhantomData<T>,
            PhantomData<Si>,
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
            fn visit_link<E>(self, cid_bytes: Box<[u8]>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let cid = CidGeneric::<Si>::try_from(cid_bytes.as_ref()).map_err(E::custom)?;
                Ok(Link {
                    cid,
                    t: PhantomData,
                })
            }
        }

        <D as Decoder<'de>>::deserialize_link(deserializer, LinkVisitor(PhantomData, PhantomData))
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
