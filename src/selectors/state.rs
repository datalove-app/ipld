use crate::dev::*;
use macros::derive_more::{AsMut, AsRef, From};
use std::{
    boxed::Box,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver, SendError, Sender},
        Arc,
    },
};

///
#[derive(Clone, Debug, From, Deserialize, Serialize)]
// #[from(forward)]
pub enum Node {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "int8")]
    Int8(i8),
    #[serde(rename = "int16")]
    Int16(i16),
    #[serde(rename = "int")]
    Int(Int),
    #[serde(rename = "int64")]
    Int64(i64),
    #[serde(rename = "int128")]
    Int128(i128),
    #[serde(rename = "uint8")]
    Uint8(u8),
    #[serde(rename = "uint16")]
    Uint16(u16),
    #[serde(rename = "uint32")]
    Uint32(u32),
    #[serde(rename = "uint64")]
    Uint64(u64),
    #[serde(rename = "uint128")]
    Uint128(u128),
    #[serde(rename = "float32")]
    Float32(f32),
    #[serde(rename = "float")]
    Float(Float),
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "bytes")]
    Bytes(crate::dev::Bytes),
    #[serde(rename = "list")]
    #[from(ignore)]
    List,
    #[serde(rename = "map")]
    #[from(ignore)]
    Map,
    #[serde(rename = "link")]
    Link(Cid),
}

impl<'a> From<&'a str> for Node {
    fn from(s: &'a str) -> Self {
        Self::String(s.into())
    }
}

impl<T: Representation> From<List<T>> for Node {
    fn from(_: List<T>) -> Self {
        Self::List
    }
}

impl<K: Representation, V: Representation> From<Map<K, V>> for Node {
    fn from(_: Map<K, V>) -> Self {
        Self::Map
    }
}

// TODO:
impl<T: Representation, Si: MultihashSize> From<Link<T, Si>> for Node {
    fn from(link: Link<T, Si>) -> Self {
        let cid: CidGeneric<Si> = link.into();
        let mh = cid.hash();
        let mh = DefaultMultihash::wrap(mh.code(), mh.digest())
            .expect("should not fail to convert a `MultihashGeneric` into a `DefaultMultihash`");
        Self::Link(
            Cid::new(cid.version(), cid.codec(), mh)
                .expect("should not fail to convert a `CidGeneric` into `Cid`"),
        )
    }
}

impl<T: Representation> From<Option<T>> for Node
where
    Node: From<T>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => Self::Null,
            Some(t) => <Self as From<T>>::from(t),
        }
    }
}

impl From<Value> for Node {
    fn from(val: Value) -> Self {
        match val {
            Value::Null => Self::Null,
            Value::Bool(inner) => Self::Bool(inner),
            Value::Int(inner) => Self::Int(inner),
            Value::Float(inner) => Self::Float(inner),
            Value::String(inner) => Self::String(inner),
            Value::Bytes(inner) => Self::Bytes(inner),
            Value::List(_) => Self::List,
            Value::Map(_) => Self::Map,
            Value::Link(link) => Self::Link(link.into()),
        }
    }
}

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

/// The selection mode of the selector, which determines what gets visited,
/// matched, sent and returned.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    /// Selection will send [`SelectionNode`]s across a [`channel`].
    Node,

    /// Selection will send [`SelectionDag`]s across a [`channel`].
    Dag,

    /// Selection will end with and return the first matching dag.
    MatchedDag,
    // /// Selection updates matching dags with the output of a callback.
    // Patch,

    // /// Selection patches and flushes changes to matching dags.
    // PatchAndFlush,
}

///
pub enum SelectionMode2<'a, C, T, U = T> {
    Select {
        /// if none, return first match
        /// otherwise, send dag or node
        sender: Option<SelectionSender>,
        _t: PhantomData<(T, U)>,
    },
    Patch {
        /// current dag we're selecting against
        /// if none, then load and store while patching
        corrent: &'a mut T,
        /// op to perform on matching dags, allowing update-inplace
        op: Box<dyn FnMut(&mut U, &mut C) -> Result<(), Error>>,
        flush: bool,
    },
}

// fn noop<C, U>(dag: &mut U, ctx: &mut C) -> Result<Option<U>, Error> {
//     Ok(None)
// }

#[derive(AsRef, AsMut, Debug)]
// pub struct SelectorState<'a> {
pub struct SelectorState {
    // selector: Selector,
    // mode: SelectionMode,
    #[as_ref]
    #[as_mut]
    path: PathBuf,
    // path: &'a mut PathBuf,
    path_depth: usize,
    link_depth: usize,
    max_path_depth: usize,
    max_link_depth: usize,
    sender: Option<SelectionSender>,
}

// impl<'a> SelectorState<'a> {
impl SelectorState {
    pub const NODE_MODE: SelectionMode = SelectionMode::Node;
    pub const DAG_MODE: SelectionMode = SelectionMode::Dag;
    pub const DAG_MATCH_MODE: SelectionMode = SelectionMode::MatchedDag;

    // pub fn new(root: PathBuf, sender: Option<SelectionSender>) -> Self {
    //     Self {
    //         path: root,
    //         path_depth: 0,
    //         link_depth: 0,
    //         max_path_depth: usize::MAX,
    //         max_link_depth: usize::MAX,
    //         sender,
    //     }
    // }

    ///
    pub fn node_selector(only_matching: bool) -> (Receiver<SelectedNode>, Self) {
        let (sender, receiver) = channel();
        (
            receiver,
            Self {
                sender: Some(SelectionSender::Node {
                    sender,
                    only_matching,
                }),
                ..Default::default()
            },
        )
    }

    ///
    pub fn dag_selector(// root: &'a mut PathBuf
    ) -> (Receiver<SelectedDag>, Self) {
        let (sender, receiver) = channel();
        (
            receiver,
            Self {
                sender: Some(SelectionSender::Dag(sender)),
                ..Default::default()
            },
        )
    }

    ///
    #[inline]
    pub fn with_max_path_depth(mut self, max_path_depth: usize) -> Self {
        if self.max_path_depth == usize::MAX {
            self.max_path_depth = max_path_depth;
        }
        self
    }

    ///
    #[inline]
    pub fn with_max_link_depth(mut self, max_link_depth: usize) -> Self {
        if self.max_link_depth == usize::MAX {
            self.max_link_depth = max_link_depth;
        }
        self
    }

    ///
    #[inline]
    pub fn mode(&self) -> SelectionMode {
        match self.sender() {
            Ok(SelectionSender::Dag(..)) => SelectionMode::Dag,
            Ok(SelectionSender::Node { .. }) => SelectionMode::Node,
            Err(_) => SelectionMode::MatchedDag,
        }
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn max_path_depth(&self) -> usize {
        self.max_path_depth
    }

    #[inline]
    pub fn max_link_depth(&self) -> usize {
        self.max_link_depth
    }

    #[inline]
    pub(crate) fn sender(&self) -> Result<&SelectionSender, Error> {
        self.sender
            .as_ref()
            .ok_or_else(|| Error::Context(anyhow::Error::msg("`SelectorSeed` missing a channel")))
    }

    // TODO: add method for diving/rising
    // diving:
    //  clones seed with
    //      - next subpath added path
    //      - incremented path depth [= incremented link depth]
    //      -

    #[inline]
    pub(crate) fn descend_index(&mut self, index: usize, is_link: bool) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    pub(crate) fn descend_field<P: AsRef<Path>>(
        &mut self,
        field: P,
        is_link: bool,
    ) -> Result<(), Error> {
        // self.descend()
        Ok(())
    }

    #[inline]
    pub(crate) fn descend<P: AsRef<Path>>(
        &mut self,
        // next_selector: Selector,
        next_path: P,
        is_link: bool,
    ) -> Result<(), Error> {
        if self.path_depth >= self.max_path_depth {
            return Err(Error::SelectorDepth(
                "descending would exceed max path depth",
                self.max_path_depth,
            ));
        } else if self.link_depth >= self.max_link_depth {
            return Err(Error::SelectorDepth(
                "descending would exceed max link depth",
                self.max_link_depth,
            ));
        }

        self.path.push(next_path);
        self.path_depth += 1;
        if is_link {
            self.link_depth += 1;
        }

        Ok(())
    }

    // #[inline]
    // pub(crate) fn ascend(
    //     &mut self,
    //     // previous_selector: Selector,
    //     is_link: bool,
    // ) -> Result<(), Error> {
    //     self.path.pop();
    //     self.path_depth = self
    //         .path_depth
    //         .checked_sub(1)
    //         .ok_or_else(|| Error::SelectorDepth("exceeds root path depth", self.path_depth))?;
    //
    //     if is_link {
    //         self.link_depth = self
    //             .link_depth
    //             .checked_sub(1)
    //             .ok_or_else(|| Error::SelectorDepth("exceeds root link depth", self.link_depth))?;
    //     }
    //
    //     // self.selector = previous_selector;
    //     Ok(())
    // }

    #[inline]
    pub(crate) fn send_selection(&self, node: Node) -> Result<(), Error> {
        Ok(self.sender()?.send_node(SelectedNode {
            path: self.path.clone(),
            node,
            matched: false,
            label: None,
        })?)
    }

    #[inline]
    pub(crate) fn send_matched(&self, node: Node, label: Option<String>) -> Result<(), Error> {
        Ok(self.sender()?.send_node(SelectedNode {
            path: self.path.clone(),
            node,
            matched: true,
            label,
        })?)
    }

    #[inline]
    pub(crate) fn send_dag<T: Representation + Send + Sync + 'static>(
        &self,
        dag: T,
        label: Option<String>,
    ) -> Result<(), Error> {
        Ok(self.sender()?.send_dag(SelectedDag {
            path: self.path.clone(),
            dag: Box::new(dag),
            label,
        })?)
    }
}

impl Default for SelectorState {
    #[inline]
    fn default() -> Self {
        Self {
            path: Default::default(),
            path_depth: 0,
            link_depth: 0,
            max_path_depth: usize::MAX,
            max_link_depth: usize::MAX,
            sender: None,
        }
    }
}

///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectedNode {
    path: PathBuf,
    node: Node,
    matched: bool,
    label: Option<String>,
}

pub type SelectedDag = InnerSelectedDag<Box<dyn ErasedRepresentation>>;

#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InnerSelectedDag<T> {
    path: PathBuf,
    dag: T,
    label: Option<String>,
}

///
#[derive(Clone, Debug, From)]
pub enum SelectionSender {
    Dag(Sender<SelectedDag>),
    Node {
        sender: Sender<SelectedNode>,
        only_matching: bool,
    },
}

impl SelectionSender {
    ///
    #[inline]
    fn send_node(&self, node: SelectedNode) -> Result<(), Error> {
        match self {
            Self::Node {
                sender,
                only_matching,
            } if (*only_matching && node.matched) || !only_matching => Ok(sender.send(node)?),
            Self::Node { .. } => Ok(()),
            _ => Err(Error::Custom(anyhow::Error::msg(
                "channel is only for `Node`s",
            ))),
        }
    }

    ///
    #[inline]
    fn send_dag(&self, dag: SelectedDag) -> Result<(), Error> {
        match self {
            Self::Dag(inner) => Ok(inner.send(dag)?),
            _ => Err(Error::Custom(anyhow::Error::msg(
                "channel is only for `Representation`s",
            ))),
        }
    }
}

#[doc(hidden)]
pub type DagSelectionSenderError = SendError<SelectedDag>;
#[doc(hidden)]
pub type NodeSelectionSenderError = SendError<SelectedNode>;

impl SelectionSender {}
