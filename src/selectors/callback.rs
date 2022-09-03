use crate::dev::*;
use macros::derive_more::{AsMut, AsRef, From};
use std::{
    boxed::Box,
    fmt,
    path::{Path, PathBuf},
};

// ///
// #[must_use = "Streams do nothing unless polled"]
// // #[derive(Debug)]
// // TODO: pin vs box?
// pub struct SelectionStream<'a> {
//     inner: Pin<Box<dyn Stream<Item = Result<SelectedNode, Error>> + 'a>>,
// }
//
// // // impl Unpin for SelectionStream {}
//
// impl<'a> SelectionStream<'a> {
//     // TODO: requires that the stream be wrapped in a pinbox - why?
//     unsafe_pinned!(inner: dyn Stream<Item = Result<SelectedNode, Error>>);
//
//     ///
//     #[inline]
//     pub fn ok(selection: SelectedNode) -> Self {
//         Self::from(async { Ok(selection) }.into_stream())
//     }
//
//     ///
//     #[inline]
//     pub fn err(err: Error) -> Self {
//         Self::from(async { Err(err) }.into_stream())
//     }
//
//     ///
//     #[inline]
//     pub fn from<S>(inner: S) -> Self
//     where
//         S: Stream<Item = Result<SelectedNode, Error>> + 'a,
//     {
//         Self {
//             inner: Box::pin(inner),
//         }
//     }
// }
//
// impl<'a> Stream for SelectionStream<'a> {
//     type Item = Result<SelectedNode, Error>;
//
//     #[inline]
//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Cx<'_>) -> Poll<Option<Self::Item>> {
//         self.inner.as_mut().poll_next(cx)
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }
//
// impl<'a> From<SelectedNode> for SelectionStream<'a> {
//     #[inline]
//     fn from(selection: SelectedNode) -> Self {
//         Self::ok(selection)
//     }
// }
//
// impl<'a> From<Error> for SelectionStream<'a> {
//     #[inline]
//     fn from(err: Error) -> Self {
//         Self::err(err)
//     }
// }
//
// impl<'a> From<SelectedNode> for Result<SelectionStream<'a>, Error> {
//     #[inline]
//     fn from(selection: SelectedNode) -> Self {
//         Ok(SelectionStream::ok(selection))
//     }
// }

// /// A thin wrapper around a `Selector` and its selection state.
// #[derive(AsRef, AsMut, Clone, Debug)]
// pub struct SelectorState {
//     pub selector: Selector,
//     #[as_ref]
//     #[as_mut]
//     params: SelectorState,
// }

// pub trait Params {}
//
// pub struct SelectParams<T, U = T> {
//     /// if none, return first match
//     /// otherwise, send dag or node
//     sender: Option<SelectionSender>,
//     _t: PhantomData<(T, U)>,
// }
//
// pub struct PatchParams<'a, C, T, U = T> {
//     /// current dag we're selecting against
//     /// if none, then load and store while patching
//     current: &'a mut T,
//     /// op to perform on matching dags, allowing update-inplace
//     op: Box<dyn Fn(&mut U, &mut C) -> Result<(), Error> + 'a>,
//     // op: fn(&mut U, &mut C) -> Result<(), Error>,
//     flush: bool,
// }
//
// impl<T, U> Params for SelectParams<T, U> {}
// impl<'a, C, T, U> Params for PatchParams<'a, C, T, U> {}

pub(crate) use callbacks::*;
mod callbacks {
    use super::*;

    ///
    pub trait SelectNodeOp<C>: FnMut(NodeSelection, &mut C) -> Result<(), Error> {
        ///
        fn clone_box<'a>(&self) -> Box<dyn SelectNodeOp<C> + 'a>
        where
            Self: 'a;
    }

    impl<C, F> SelectNodeOp<C> for F
    where
        F: FnMut(NodeSelection, &mut C) -> Result<(), Error> + Clone,
    {
        fn clone_box<'a>(&self) -> Box<dyn SelectNodeOp<C> + 'a>
        where
            Self: 'a,
        {
            Box::new(self.clone())
        }
    }

    impl<'a, C> Clone for Box<dyn SelectNodeOp<C> + 'a>
    where
        C: 'a,
    {
        fn clone(&self) -> Self {
            (**self).clone_box()
        }
    }

    ///
    pub trait SelectDagOp<C>: FnMut(DagSelection, &mut C) -> Result<(), Error> {
        ///
        fn clone_box<'a>(&self) -> Box<dyn SelectDagOp<C> + 'a>
        where
            Self: 'a;
    }

    impl<C, F> SelectDagOp<C> for F
    where
        F: FnMut(DagSelection, &mut C) -> Result<(), Error> + Clone,
    {
        fn clone_box<'a>(&self) -> Box<dyn SelectDagOp<C> + 'a>
        where
            Self: 'a,
        {
            Box::new(self.clone())
        }
    }

    impl<'a, C> Clone for Box<dyn SelectDagOp<C> + 'a>
    where
        C: 'a,
    {
        fn clone(&self) -> Self {
            (**self).clone_box()
        }
    }

    ///
    pub trait MatchDagOp<T, C>: FnMut(T, &mut C) -> Result<(), Error> {
        ///
        fn clone_box<'a>(&self) -> Box<dyn MatchDagOp<T, C> + 'a>
        where
            Self: 'a;
    }

    impl<T, C, F> MatchDagOp<T, C> for F
    where
        F: FnMut(T, &mut C) -> Result<(), Error> + Clone,
    {
        fn clone_box<'a>(&self) -> Box<dyn MatchDagOp<T, C> + 'a>
        where
            Self: 'a,
        {
            Box::new(self.clone())
        }
    }

    impl<'a, T, C> Clone for Box<dyn MatchDagOp<T, C> + 'a>
    where
        T: 'a,
        C: 'a,
    {
        fn clone(&self) -> Self {
            (**self).clone_box()
        }
    }
}

/*
///
/// https://stackoverflow.com/questions/65203307/how-do-i-create-a-trait-object-that-implements-fn-and-can-be-cloned-to-distinct
pub trait PatchOp<C, U>: Fn(&mut U, &mut C) -> Result<(), Error> {
    ///
    fn clone_box<'a>(&self) -> Box<dyn PatchOp<C, U> + 'a>
    where
        Self: 'a;
}

impl<C, U, F> PatchOp<C, U> for F
where
    F: Fn(&mut U, &mut C) -> Result<(), Error> + Clone,
{
    fn clone_box<'a>(&self) -> Box<dyn PatchOp<C, U> + 'a>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl<'a, C, U> Clone for Box<dyn PatchOp<C, U> + 'a>
where
    C: 'a,
    U: 'a,
{
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}
 */

// type SelectFn<U, C> = fn(U, &mut C) -> Result<(), Error>;
// type PatchFn<U, C> = fn(U, &mut C) -> Result<(), Error>;

/// The selection mode of the selector, which determines what gets visited,
/// matched, sent and returned.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SelectionMode {
    /// Selection will invoke the provided callback on all traversed [`Node`]s.
    SelectNode,

    /// Selection will invoke the provided callback on all matched [`Dag`]s.
    SelectDag,
    // ///
    // MatchDag,
    // /// Selection updates matching dags with the output of a callback.
    // /// Optionally flushes changes after each callback.
    // Patch,
}

///
pub(crate) enum SelectionCallback<'a, C, T> {
    SelectNode {
        cb: Box<dyn SelectNodeOp<C> + 'a>,
        only_matched: bool,
    },
    SelectDag {
        // TODO: does this need to be cloneable? it is either called on U, or wrapped
        cb: Box<dyn SelectDagOp<C> + 'a>,
    },
    MatchDag {
        cb: Box<dyn MatchDagOp<T, C> + 'a>,
    }, // Patch {
       //     /// current dag we're selecting against
       //     /// if none, then load and store while patching
       //     current: &'a mut T,
       //     flush: bool,
       //     // op to perform on matching dags, allowing update-inplace
       //     // op: Box<dyn PatchOp<C, U> + 'a>,
       //     // op: PatchFn<C, U>,
       // }
}

impl<'a, C, T> fmt::Debug for SelectionCallback<'a, C, T>
where
    C: Context,
    T: Representation,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelectNode { only_matched, .. } => f
                .debug_struct("SelectionParams::SelectNode")
                .field("source", &T::NAME)
                .field("only_matched", only_matched)
                .finish(),
            Self::SelectDag { .. } => f
                .debug_struct("SelectionParams::SelectDag")
                .field("source", &T::NAME)
                .finish(),
            Self::MatchDag { .. } => f
                .debug_struct("SelectionParams::MatchDag")
                .field("source", &T::NAME)
                .finish(),
            // Self::Patch { current, flush, .. } => f
            //     .debug_struct("SelectionParams::Patch")
            //     .field("current", &current)
            //     .field("flush", &flush)
            //     .finish(),
        }
    }
}

impl<'a, C, T> Default for SelectionCallback<'a, C, T>
where
    C: Context,
    T: Representation,
{
    /// Defaults to a no-op function for selecting nodes.
    fn default() -> Self {
        Self::SelectNode {
            cb: Box::new(|_, _| Ok(())),
            only_matched: true,
        }
    }
}

// impl<'a, C, T, Cb> Into<SelectionCallback<'a, C, T>> for Cb
// where
//     C: Context,
//     T: Representation,
//     Cb: SelectNodeOp<C> + 'a,
// {
//     fn into(self) -> SelectionCallback<'a, C, T> {
//         Self::SelectNode {
//             cb: Box::new(self),
//             only_matched: false,
//         }
//     }
// }

impl<'a, C, T> SelectionCallback<'a, C, T>
where
    C: Context,
    T: Representation,
{
    // #[inline]
    // pub(crate) fn select_node(&self, node: SelectedNode) -> Result<(), Error> {
    //     self.sender()?.send_node(node)
    // }
    //
    // #[inline]
    // pub(crate) fn select_dag(&self, dag: SelectedDag) -> Result<(), Error> {
    //     self.sender()?.send_dag(dag)
    // }
    //
    // #[inline]
    // fn sender(&self) -> Result<&SelectionSender, Error> {
    //     let sender = match self {
    //         Self::Select { sender, .. } => sender.as_ref(),
    //         _ => None,
    //     };
    //
    //     sender.ok_or_else(|| Error::InvalidSelectionMode("`SelectionParams` missing a channel"))
    // }

    // /// transmutes the select params current source and target
    // pub(crate) fn to_select<'b, V, W>(&mut self) -> SelectionParams<'b, C, V, W>
    // where
    //     'a: 'b,
    //     C: 'b,
    // {
    //     match self {
    //         Self::Select { cb, mode, .. } => SelectionParams::Select {
    //             cb: cb.clone(),
    //             mode: *mode,
    //             _t: PhantomData,
    //         },
    //         _ => unreachable!(),
    //     }
    // }

    ///
    pub const fn mode(&self) -> SelectionMode {
        match self {
            Self::SelectNode { .. } => SelectionMode::SelectNode,
            Self::SelectDag { .. } | Self::MatchDag { .. } => SelectionMode::SelectDag,
            // Self::Patch { .. } => SelectionMode::Patch,
        }
    }

    pub const fn is_node(&self) -> bool {
        match self {
            Self::SelectNode { .. } => true,
            _ => false,
        }
    }

    pub const fn is_dag(&self) -> bool {
        match self {
            Self::SelectDag { .. } | Self::MatchDag { .. } => true,
            _ => false,
        }
    }

    pub(super) fn select_node(
        &mut self,
        selected_node: NodeSelection,
        ctx: &mut C,
    ) -> Result<(), Error> {
        match self {
            Self::SelectNode { cb, only_matched }
                if !*only_matched || selected_node.matched && *only_matched =>
            {
                cb(selected_node, ctx)
            }
            Self::SelectNode { .. } => Ok(()),
            _ => unreachable!(),
        }
    }

    pub(super) fn select_dag(&mut self, dag: DagSelection, ctx: &mut C) -> Result<(), Error> {
        match self {
            Self::SelectDag { cb } => cb(dag, ctx),
            _ => unreachable!(),
        }
    }
}
