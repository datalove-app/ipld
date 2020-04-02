use crate::dev::{Context, FromContext, Representation, Selector};
// use ipld::prelude::*;
use std::convert::TryFrom;

/// ...
pub struct Executor<'a, Ctx> {
    context: &'a Ctx,
    // TODO? top-level selector?
    selector: Selector,
}

impl<'a, Ctx> Executor<'a, Ctx>
where
    Ctx: Context,
{
    // async fn focus<>(&self, root: )

    // ///
    // /// TODO?
    // async fn read_ipld(&self, root: &BorrowedIpld<'a>) -> Result<BorrowedIpld<'a>, ()>
    // where
    //     Ctx: Sync,
    // {
    //     unimplemented!()
    // }

    // ///
    // /// TODO?
    // async fn write_ipld<T>(&self, root: &BorrowedIpld<'a>) -> Result<Selector, ()>
    // where
    //     Ctx: Sync,
    // {
    //     unimplemented!()
    // }

    // async fn read<I, O>(&self, value: &I) -> Result<O, ()>
    // where
    //     I: Representation<Ctx> + Send + Sync,
    //     O: Representation

    // ///
    // /// takes in IO so that when resolving a link (of any kind), it can resolve itself, then continue munching the selector into a return type
    // /// TODO? return I? return O? return BorrowedIpld only?
    // pub async fn resolve<I, O>(&self, value: &I) -> Result<O, ()>
    // where
    //     Ctx: Sync,
    //     I: Representation<Ctx> + Send + Sync,
    //     O: Representation<Ctx> + Send + Sync + TryFrom<BorrowedIpld<'a>>,
    //     <O as <TryFrom<BorrowedIpld<'a>>>>::Error: Debug; // FIXME:
    // {
    //     let ipld = self.resolve_into_ipld(value).await?;
    //     Ok(O::try_from(ipld).unwrap())
    // }

    // pub async fn resolve_into_ipld<I>(&self, value: &I) -> Result<BorrowedIpld<'a>, ()>
    // where
    //     Ctx: Sync,
    //     I: Representation<Ctx> + Send + Sync,
    //     <O as <TryFrom<BorrowedIpld<'a>>>>::Error: Debug; // FIXME:
    // {
    //     value.resolve_into_ipld(&self.selector, self).await?;
    // }

    // TODO:
    // ///
    // pub async fn resolve_with_context<NewCtxT, T>(&self) -> Result<T, ()>
    // where
    //     NewCtxT: Context + FromContext<Ctx> + Sync,
    //     T: Representation<NewCtxT> + Send + Sync,
    // {
    //     // self.with_replaced_context().resolve().await
    //     unimplemented!()
    // }

    // ///
    // pub fn with_replaced_context<'b, NewCtxT>(&'b self) -> Executor<'b, NewCtxT>
    // where
    //     NewCtxT: Context + FromContext<Ctx>,
    // {
    //     Executor {
    //         context: <NewCtxT as FromContext<Ctx>>::from(self.context),
    //         selector: Selector,
    //     }
    // }

    ///
    pub fn context(&self) -> &'a Ctx {
        self.context
    }
}
