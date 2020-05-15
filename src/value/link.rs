use crate::dev::*;
use std::{cell::Cell, rc::Rc};

/// Link type, used to switch between a `Cid` and it's underlying dag.
/// TODO: impl Serialize for Link, checking if impls!(S: Encoder)
#[derive(Debug)]
pub struct Link<T>(Cell<InnerLink<T>>);

#[derive(Debug)]
enum InnerLink<T> {
    /// Represents a raw `Cid` contained within a dag.
    Cid(Cid),

    /// Represents a parsed subset of a dag and its original `Cid`.
    Selection {
        cid: Cid,
        selector: Rc<Selector>,
        dag: T,
    },
}

impl<T: Representation> Representation for Link<T> {
    const NAME: &'static str = "Link";
    // const SCHEMA: &'static str = format!("type {} = &{}", Self::NAME, T::NAME);
    // const KIND: Kind = Kind::Link;
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

impl<T: Representation> RepresentationExt<T> for Link<T> {
    // fn codec(&self) ->

    // /// resolves a link into it's full underlying dag T
    // ///
    // /// when a Link::Cid is focused, its cloned, fully/partially resolved, replaces itself with T, then delegates to T::focus
    // /// when a Link::Dag is focused, it delegates to T::focus
    // /// when a Link::Selection is focused, panic/error?
    // ///
    // ///
    // /// when a Link::Cid is resolved, it deserializes T against the Executor
    // /// when a Link::Dag is resolved, it returns T
    // /// when a Link::Selection is resolved, panic/error?
    // /// TODO: FromContext
    // async fn resolve<'a>(self, executor: &'a Executor<'a, Ctx>) -> Result<T, ()> {
    //     unimplemented!()
    // }

    //
    //
    //
    //
    //

    // /// resolves a link against a selector into a selection of dag T
    // /// TODO? FromContext
    // async fn resolve_selector<'a>(
    //     self,
    //     selector: &Selector,
    //     executor: &'a Executor<'a, CtxT>,
    // ) -> Result<T, ()>
    // where
    //     Selector: Visitor<'a, Value = T>,
    //     T: Representation<CtxT>,
    // {
    //     unimplemented!()
    // }
}

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

////////////////////////////////////////////////////////////////////////////////
// additional implementations
////////////////////////////////////////////////////////////////////////////////

impl<T> From<Cid> for Link<T> {
    fn from(cid: Cid) -> Self {
        Link(Cell::new(InnerLink::Cid(cid)))
    }
}
