use crate::dev::*;
use std::cell::Cell;

/// Link type, used to switch between a `Cid` and it's underlying dag.
#[derive(Debug)]
pub struct Link<T>(Cell<InnerLink<T>>);

#[derive(Debug)]
enum InnerLink<T> {
    /// Represents a raw `Cid` contained within a dag.
    Cid(Cid),

    /// Represents a raw `Cid` and an instance of the dag it originally represented.
    Dag { cid: Cid, dag: T, dirty: bool },

    /// Represents selected subset of a dag.
    Selection(Selector, T),
}

impl<T> Representation for Link<T> {
    const NAME: &'static str = format!("{}Link", T::NAME);
    const KIND: SchemaKind = SchemaKind::Link;
}

impl<T> RepresentationExt<T> Link<T> {
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
