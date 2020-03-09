use crate::{async_trait, CborError, ReadCbor, WriteCbor};
use crate::{
    context::{FlushBlock, ResolveBlock},
    Cid, Context, Error, Read, Representation, Write,
};

/// Link type, used to switch between a `Cid` and it's underlying dag.
#[derive(Debug)]
pub enum Link<T: Representation> {
    /// Represents a raw `Cid` contained within a dag.
    Cid(Cid),

    /// Represents a raw `Cid` and an instance of the type it originally represented.
    Dag { original_cid: Cid, dag: T },
}

impl<T> Link<T> {
    // fn codec(&self) ->
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
        Link::Cid(cid)
    }
}
