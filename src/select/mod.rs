//! IPLD Selectors
//!
//! TODO:
//!     - selectors are types that impl Representation (can be defined with `schema!`)
//!     - macro can compile selector string to a type
//!     - type implements Context
#![allow(non_camel_case_types)]

// mod path;
mod context;
#[macro_use]
mod seed;
mod selectors;
mod state;

pub use context::*;
pub use dag_selection::*;
pub use field::*;
pub use node_selection::*;
pub use params::*;
pub use seed::*;
pub use selectors::*;
pub use state::*;

use crate::dev::*;
use macros::derive_more::{Display, From};
use maybestd::{
    cell::RefCell,
    future::{self, IntoFuture},
    path::{Path, PathBuf},
    str::FromStr,
    task::Poll,
    vec::IntoIter,
};

///
/// TODO: + 'static?
pub trait Select<Ctx: Context = ()>: Representation + 'static {
    ///
    #[doc(hidden)]
    type Walker<'a, const MC: u64>: Walk<'a, MC, Self> + From<SelectorSeed<'a, Ctx, Self>>
    where
        Ctx: 'a;

    /// ...
    ///
    /// Under the hood, this serves as the entrypoint for deserialization of a
    /// block via a typed `ContextSeed`: a type that implements
    /// [`DeserializeSeed`] for each [`Select`]-able type, and uses the
    /// contained [`Selector`], the type's [`Representation`] and the provided
    /// [`Context`] to govern how to interpret the types found in blocks.
    ///
    ///
    /// TODO: update this interface, since SelectorSeed is doing the work and it should be refactored a bit (borrow state, )
    ///
    /// todo: 1. build seed, fetch deserializer => seed.deserialize(de)
    /// todo: 2. should maybe accept a deserializer + params + ctx
    fn select_in<'a, P>(params: P, ctx: Ctx) -> Result<(), Error>
    where
        P: Into<Params<'a, Ctx>>,
        Self: 'a,
    {
        let Params {
            selector,
            mut state,
            callback,
        } = params.into();
        let seed = SelectorSeed::from_parts(&selector, &mut state, callback, ctx);

        // todo:
        // if matcher, Repr::deserialize and call callback (match_val)
        // if explore_union, read block for each (? in parallel?)
        // if explore_interpret_as, ...
        // if let Some(matcher) = seed.selector.as_matcher() {
        //     // params.callback(self.as_ref())
        //     return Ok(());
        // }

        Self::__select_in(seed)
    }

    #[doc(hidden)]
    // fn __select<'a, W: Into<Self::Walk<'a, Ctx>>>(walker: W) -> Result<(), Error> {
    fn __select_in<'a>(mut seed: SelectorSeed<'a, Ctx, Self>) -> Result<(), Error> {
        match seed.selector {
            // if exploreunion, call select on each (? in parallel?)
            Selector::ExploreUnion(_) => todo!(),
            _ => {
                let cid = &seed.state.current_block;
                let block = seed.ctx.block_reader(cid)?;
                cid.multicodec()?.read_with_seed(seed, block)
            }
        }
    }

    // #[doc(hidden)]
    // fn __select_de<'a, 'de, const MC: u64, D>(
    //     seed: SelectorSeed<'a, Ctx, Self>,
    //     deserializer: D,
    // ) -> Result<(), D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     Err(D::Error::custom("__select_de not yet implemented"))
    //     // TODO: default impl should use GAT for CodedSeed
    //     // Self::Seed::<'a, 'de, C>::from(seed).deserialize(deserializer)
    // }

    /// Selects against the dag, loading more blocks from `Ctx` if required.
    ///
    /// todo: 1. build seed,
    ///     if match, call selectdagrefop on ref (match_ref)
    ///     otherwise, create XAccessDeserializer => seed.visit_XX(accessor)
    ///     * select_in and select share same code path b/c ... callbacks on refs have indeterminate lifetimes?
    #[doc(hidden)]
    fn select<'a, P>(&'a self, params: P, ctx: Ctx) -> Result<(), Error>
    where
        P: Into<Params<'a, Ctx>>,
    {
        let Params {
            selector,
            mut state,
            callback,
        } = params.into();
        let seed = SelectorSeed::from_parts(&selector, &mut state, callback, ctx);

        self.__select(seed)
    }

    #[doc(hidden)]
    fn __select<'a>(&'a self, mut seed: SelectorSeed<'a, Ctx, Self>) -> Result<(), Error> {
        match seed.selector {
            // if matcher, call selectdagop on self
            Selector::Matcher(_) => {
                seed.select_ref(self)?;
                return Ok(());
            }
            // if exploreunion, call select on each (? in parallel?)
            Selector::ExploreUnion(_) => todo!(),
            _ => unreachable!(),
        }

        // let mut de = self.[into_access().]into_deserializer();
        // Self::Walk::<'a, IDENTITY>::from(seed).deserialize(&mut de)
    }

    /// Patches the dag according to the selector, loading more blocks from
    /// `Ctx` if required. Returns `true` if any patch operation was executed
    /// and subsequently mutated a part of the dag.
    ///
    /// todo: 1. build seed,
    ///     if match, call patchdagop on ref_mut (match_ref_mut)
    ///     if exploreunion, patch one at a time
    ///     otherwise, create self+seed [+ iterator] => seed.patch_list/map(..)
    #[doc(hidden)]
    fn patch<'a, P>(&'a mut self, params: P, ctx: Ctx) -> Result<(), Error>
    where
        P: Into<Params<'a, Ctx>>,
    {
        let Params {
            selector,
            mut state,
            callback,
        } = params.into();
        let seed = SelectorSeed::<'_, Ctx, Self>::from_parts(&selector, &mut state, callback, ctx);

        self.__patch(seed)
    }

    #[doc(hidden)]
    fn __patch<'a>(&'a mut self, mut seed: SelectorSeed<'a, Ctx, Self>) -> Result<(), Error> {
        // if matcher, call selectdagop on self
        // if exploreunion, call select on each (? in parallel?)
        match seed.selector {
            Selector::Matcher(_) => {
                seed.patch_dag(self)?;
                return Ok(());
            }
            Selector::ExploreUnion(_) => todo!(),
            _ => self.__patch(seed),
        }
    }

    /// Flushes the dag according to the selector, writing blocks to `Ctx` if
    /// flushing linked dags.
    ///
    /// TODO
    #[doc(hidden)]
    fn flush(&mut self, params: Params<'_, Ctx>, ctx: &mut Ctx) -> Result<(), Error> {
        unimplemented!()
    }

    // fn patch<S: Select<C>>(seed: ContextSeed<'_, C, Self, S>) -> Result<(), Error> {
    //     unimplemented!()
    // }
}

fn select_async<Ctx, T>(
    params: Params<'_, Ctx>,
    ctx: &mut Ctx,
) -> impl IntoFuture<Output = Result<(), Error>>
where
    Ctx: Context,
    T: Select<Ctx>,
{
    enum SM {
        Waiting,
        Selecting,
        Done,
    }
    // let first = future::poll_fn(|cx|
    // 1. poll block_reader; SM::Waiting -> SM::Selecting
    // Poll::Pending);

    // let next = future::poll_fn(|cx|
    // 2. begin selection
    //  - every subsequent block_reader call
    // Poll::Pending);
    // 3. await ... ?

    future::pending()
}

// impl<C, T> Select<C> for T
// where
//     C: Context,
//     T: Representation + 'static,
//     for<'a, 'de> ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     fn select(params: SelectionParams<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
//         select_from_seed::<C, T>(params, ctx)
//     }
// }

// ///
// #[doc(hidden)]
// pub fn select_from_seed<C, T>(
//     params: SelectionParams<'_, C, T>,
//     mut ctx: &mut C,
// ) -> Result<(), Error>
// where
//     C: Context,
//     T: Representation + 'static,
//     for<'a, 'de> ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
// {
//     let default_selector = Selector::DEFAULT;
//
//     let SelectionParams {
//         cid,
//         selector,
//         max_path_depth,
//         max_link_depth,
//         callback,
//     } = params;
//     let mut state = SelectionState {
//         max_path_depth,
//         max_link_depth,
//         ..Default::default()
//     };
//     let seed = ContextSeed {
//         selector: &selector.unwrap_or(&default_selector),
//         state: &mut state,
//         callback,
//         ctx: &mut ctx,
//     };
//
//     Ok(seed.read(&cid)?)
// }

mod params {
    use super::*;

    ///
    #[derive(Debug)]
    pub struct Params<'a, Ctx>
    where
        Ctx: Context,
        // T: Representation,
    {
        pub(crate) selector: &'a Selector,
        pub(crate) state: State,
        // pub(crate) max_path_depth: Option<usize>,
        // pub(crate) max_link_depth: Option<usize>,
        pub(crate) callback: Option<Callback<'a, Ctx>>,
    }

    // impl<'a, Ctx, T> Default for Params<'a, Ctx, T>
    // where
    //     Ctx: Context,
    //     T: Representation,
    // {
    //     fn default() -> Self {
    //         Self {
    //             selector: None,
    //             state: Default::default(),
    //             // max_path_depth: None,
    //             // max_link_depth: None,
    //             callback: None,
    //         }
    //     }
    // }

    impl<'a, Ctx> Params<'a, Ctx>
    where
        Ctx: Context,
        // T: Representation,
    {
        ///
        pub fn new(selector: &'a Selector) -> Self {
            Self {
                selector,
                state: Default::default(),
                callback: Default::default(),
            }
        }

        // ///
        // pub fn with_root(mut self, cid: Cid) -> Self {
        //     self.state.current_block = cid;
        //     self
        // }

        // ///
        // pub fn with_selector(mut self, selector: &'a Selector) -> Self {
        //     self.selector.replace(selector);
        //     self
        // }

        ///
        pub fn with_max_path_depth(mut self, max_path_depth: usize) -> Self {
            self.state.max_path_depth.replace(max_path_depth);
            self
        }

        ///
        pub fn with_max_link_depth(mut self, max_link_depth: usize) -> Self {
            self.state.max_link_depth.replace(max_link_depth);
            self
        }

        // ///
        // pub fn into_node_iter<T>(
        //     self,
        //     only_results: bool,
        //     ctx: &mut Ctx,
        // ) -> Result<IntoIter<NodeSelection>, Error>
        // where
        //     T: Select<Ctx>,
        // {
        //     let vec = RefCell::new(Vec::new());
        //     let params = Params {
        //         callback: Some(Callback::SelectNode {
        //             only_matched: only_results,
        //             cb: Box::new(|node, _| {
        //                 vec.borrow_mut().push(node);
        //                 Ok(())
        //             }),
        //         }),
        //         ..self
        //     };

        //     T::select_in(params, ctx)?;
        //     Ok(vec.into_inner().into_iter())
        // }

        ///
        /// TODO: make this more like an actual iterator, that can pause across links
        pub fn select_in<T>(self, cid: Cid, ctx: Ctx) -> Result<IntoIter<DagSelection>, Error>
        where
            T: Select<Ctx>,
        {
            let vec = RefCell::new(Vec::new());
            let params = Params {
                state: State {
                    current_block: cid,
                    ..self.state
                },
                callback: Some(Callback::SelectDag {
                    cb: Box::new(|node, _| {
                        vec.borrow_mut().push(node);
                        Ok(())
                    }),
                }),
                ..self
            };

            T::select_in(params, ctx)?;
            Ok(vec.into_inner().into_iter())
        }

        /*
        pub(crate) fn into_parts(
            self,
        ) -> (
            Cid,
            &'a Selector,
            Option<usize>,
            Option<usize>,
            SelectionCallback<'a, C, T>,
        ) {
            let SelectionParams {
                cid,
                selector,
                max_path_depth,
                max_link_depth,
                callback,
            } = self;
            (cid, selector, max_path_depth, max_link_depth, callback)
        }
         */
    }
}

mod node_selection {
    use super::*;

    ///
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NodeSelection {
        ///
        pub path: PathBuf,
        ///
        pub node: SelectedNode,
        ///
        pub matched: bool,
        ///
        pub label: Option<String>,
    }

    impl NodeSelection {
        ///
        pub fn covered<T>(path: &Path, node: T) -> Self
        where
            T: Into<SelectedNode>,
        {
            Self {
                path: path.to_owned(),
                node: node.into(),
                matched: false,
                label: None,
            }
        }

        ///
        pub fn result<T>(path: &Path, node: T, label: Option<&str>) -> Self
        where
            T: Into<SelectedNode>,
        {
            Self {
                path: path.to_owned(),
                node: node.into(),
                matched: true,
                label: label.map(str::to_string),
            }
        }
    }
}

mod dag_selection {
    use super::*;

    ///
    pub struct DagRefSelection<'a> {
        pub path: &'a Path,
        pub dag: &'a dyn ErasedRepresentation,
        pub label: Option<&'a str>,
    }

    impl<'a> DagRefSelection<'a> {
        ///
        pub fn new<T>(path: &'a Path, dag: &'a T, label: Option<&'a str>) -> Self
        where
            T: Representation + 'static,
        {
            Self {
                path,
                dag: dag,
                label,
            }
        }

        ///
        #[inline]
        pub fn is<T>(&self) -> bool
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any().is::<T>()
        }

        ///
        #[inline]
        pub fn downcast_ref<T>(&self) -> Option<&T>
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any().downcast_ref()
        }
    }

    ///
    pub struct DagRefMutSelection<'a> {
        pub path: &'a Path,
        pub dag: &'a mut dyn ErasedRepresentation,
        pub label: Option<&'a str>,
    }

    impl<'a> DagRefMutSelection<'a> {
        ///
        pub fn new<T>(path: &'a Path, dag: &'a mut T, label: Option<&'a str>) -> Self
        where
            T: Representation + 'static,
        {
            Self {
                path,
                dag: dag,
                label,
            }
        }

        ///
        #[inline]
        pub fn is<T>(&self) -> bool
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any().is::<T>()
        }

        ///
        #[inline]
        pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any_mut().downcast_mut()
        }
    }

    ///
    pub struct DagSelection {
        pub path: PathBuf,
        pub dag: Box<dyn ErasedRepresentation>,
        pub label: Option<std::string::String>,
    }

    impl DagSelection {
        ///
        pub fn new<T>(path: &Path, dag: T, label: Option<&str>) -> Self
        where
            T: Representation + 'static,
        {
            Self {
                path: path.to_owned(),
                dag: Box::new(dag),
                label: label.map(str::to_string),
            }
        }

        ///
        #[inline]
        pub fn is<T>(&self) -> bool
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any().is::<T>()
        }

        ///
        #[inline]
        pub fn downcast_ref<T>(&self) -> Option<&T>
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any().downcast_ref()
        }

        ///
        #[inline]
        pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
        where
            T: Representation + 'static,
        {
            (*self.dag).as_any_mut().downcast_mut()
        }

        ///
        #[inline]
        pub fn downcast<T>(self) -> Result<T, Error>
        where
            T: Representation + 'static,
        {
            self.dag
                .downcast()
                .map(|dag| *dag)
                .map_err(|_| Error::downcast_failure::<T>("incorrect type"))
        }
    }

    // pub trait IntoDagIterator: Iterator<Item = DagSelection> + Sized {
    //     fn into<T: Representation + 'static>(
    //         self,
    //     ) -> std::iter::Map<Self, Box<dyn Fn(DagSelection) -> Result<T, Error>>> {
    //         self.map(Box::new(|DagSelection { dag, .. }| dag.downcast()))
    //     }
    // }
    //
    // impl<I> IntoDagIterator for I where I: Iterator<Item = DagSelection> + Sized {}
    //
    // impl<T> Into<(PathBuf, Option<T>, Option<String>)> for DagSelection
    // where
    //     T: Representation + 'static,
    // {
    //     fn into(self) -> (PathBuf, Option<T>, Option<String>) {
    //         let dag = self.dag.downcast();
    //         (self.path, dag, self.label)
    //     }
    // }

    // schema! {
    //     #[ipld_attr(internal)]
    //     type List = Null
    // }

    // schema! {
    //     #[ipld_attr(internal)]
    //     type Map = Null
    // }

    // schema! {
    //     #[ipld_attr(internal)]
    //     pub type SelectedNode2 union {
    //         | Null "null"
    //         | Bool "bool"
    //         | Int8 "int8"
    //         | Int16 "int16"
    //         | Int32 "int32"
    //         | Int64 "int64"
    //         | Int128 "int"
    //         | Uint8 "uint8"
    //         | Uint16 "uint16"
    //         | Uint32 "uint32"
    //         | Uint64 "uint64"
    //         | Uint128 "uint128"
    //         | Float32 "float32"
    //         | Float64 "float64"
    //         | String "string"
    //         | Bytes "bytes"
    //         | List "list"
    //         | Map "map"
    //         | Link "link"
    //     } representation keyed
    // }

    ///
    #[derive(
        Clone,
        Debug,
        From,
        Deserialize,
        Serialize,
        // Representation
    )]
    // #[from(forward)]
    // #[ipld(internal)]
    pub enum SelectedNode {
        ///
        #[serde(rename = "null")]
        // #[ipld(rename = "null")]
        Null,

        ///
        #[serde(rename = "bool")]
        // #[ipld(rename = "bool")]
        Bool(bool),

        ///
        #[serde(rename = "int8")]
        // #[ipld(rename = "int8")]
        Int8(i8),

        ///
        #[serde(rename = "int16")]
        // #[ipld(rename = "int16")]
        Int16(i16),

        ///
        #[serde(rename = "int32")]
        // #[ipld(rename = "int32")]
        Int32(i32),

        ///
        #[serde(rename = "int64")]
        // #[ipld(rename = "int64")]
        Int64(i64),

        ///
        #[serde(rename = "int")]
        // #[ipld(rename = "int")]
        Int128(i128),

        ///
        #[serde(rename = "uint8")]
        // #[ipld(rename = "uint8")]
        Uint8(u8),

        ///
        #[serde(rename = "uint16")]
        // #[ipld(rename = "uint16")]
        Uint16(u16),

        ///
        #[serde(rename = "uint32")]
        // #[ipld(rename = "uint32")]
        Uint32(u32),

        ///
        #[serde(rename = "uint64")]
        // #[ipld(rename = "uint64")]
        Uint64(u64),

        ///
        #[serde(rename = "uint128")]
        // #[ipld(rename = "uint128")]
        Uint128(u128),

        ///
        #[serde(rename = "float32")]
        // #[ipld(rename = "float32")]
        Float32(f32),

        ///
        #[serde(rename = "float64")]
        // #[ipld(rename = "float64")]
        Float64(f64),

        ///
        #[serde(skip)] // TODO
        #[serde(rename = "string")]
        // #[ipld(rename = "string")]
        String(String),

        ///
        #[serde(skip)] // TODO
        #[serde(rename = "bytes")]
        // #[ipld(rename = "bytes")]
        Bytes(Bytes),

        ///
        #[serde(rename = "list")]
        // #[ipld(rename = "list")]
        #[from(ignore)]
        List,

        ///
        #[serde(rename = "map")]
        // #[ipld(rename = "map")]
        #[from(ignore)]
        Map,

        ///
        #[serde(skip)] // TODO
        #[serde(rename = "link")]
        // #[ipld(rename = "link")]
        Link(Cid),
    }

    impl SelectedNode {
        /// The IPLD [Data Model]() [`Kind`] of the selected node.
        pub const fn kind(&self) -> Kind {
            match self {
                Self::Null => Kind::Null,
                Self::Bool(_) => Kind::Bool,
                Self::Int8(_)
                | Self::Int16(_)
                | Self::Int32(_)
                | Self::Int64(_)
                | Self::Int128(_)
                | Self::Uint8(_)
                | Self::Uint16(_)
                | Self::Uint32(_)
                | Self::Uint64(_)
                | Self::Uint128(_) => Kind::Int,
                Self::Float32(_) | Self::Float64(_) => Kind::Float,
                Self::String(_) => Kind::String,
                Self::Bytes(_) => Kind::Bytes,
                Self::List => Kind::List,
                Self::Map => Kind::Map,
                Self::Link(_) => Kind::Link,
            }
        }
    }

    impl From<Null> for SelectedNode {
        fn from(_: Null) -> Self {
            Self::Null
        }
    }

    impl<'a> From<&'a str> for SelectedNode {
        fn from(s: &'a str) -> Self {
            Self::String(s.into())
        }
    }

    impl<'a> From<&'a [u8]> for SelectedNode {
        fn from(bytes: &'a [u8]) -> Self {
            Self::Bytes(Bytes::copy_from_slice(bytes))
        }
    }

    // TODO: Vec<u8>?
    impl<T: Representation> From<crate::dev::List<T>> for SelectedNode {
        fn from(_: crate::dev::List<T>) -> Self {
            Self::List
        }
    }

    impl<K: Representation, V: Representation> From<crate::dev::Map<K, V>> for SelectedNode {
        fn from(_: crate::dev::Map<K, V>) -> Self {
            Self::Map
        }
    }

    impl<T: Representation> From<Link<T>> for SelectedNode {
        fn from(link: Link<T>) -> Self {
            Self::Link(link.into())
        }
    }

    impl<T: Representation> From<Option<T>> for SelectedNode
    where
        SelectedNode: From<T>,
    {
        fn from(opt: Option<T>) -> Self {
            match opt {
                None => Self::Null,
                Some(t) => <Self as From<T>>::from(t),
            }
        }
    }

    // impl From<Any> for SelectedNode {
    //     fn from(val: Any) -> Self {
    //         match val {
    //             Any::Null(_) => Self::Null,
    //             Any::Bool(inner) => Self::Bool(inner),
    //             Any::Int(inner) => Self::Int64(inner),
    //             Any::Float(inner) => Self::Float64(inner),
    //             Any::String(inner) => Self::String(inner),
    //             Any::Bytes(inner) => Self::Bytes(inner),
    //             Any::List(_) => Self::List,
    //             Any::Map(_) => Self::Map,
    //             Any::Link(link) => Self::Link(*link.as_ref().cid()),
    //         }
    //     }
    // }
}

mod field {
    use super::*;
    use std::borrow::Cow;

    /// Wrapper type for types that can be used as dag keys or indices.
    #[doc(hidden)]
    #[derive(Clone, Debug, Display, From, Hash, Eq, PartialEq, PartialOrd, Ord)]
    // #[from(forward)]
    pub enum Field<'a> {
        Key(Cow<'a, str>),
        // CidKey(&'a Cid),
        Index(usize),
    }

    impl<'a> Field<'a> {
        pub fn append_to_path(&self, path: &mut PathBuf) {
            // todo: escaping to prevent hijacking root?
            match self {
                Self::Key(s) => path.push(s.as_ref()),
                // Self::CidKey(c) => path.push(c.to_string()),
                Self::Index(idx) => path.push(idx.to_string()),
            }
        }

        pub fn into_owned<'b>(&'a self) -> Field<'b> {
            match self {
                Self::Key(s) => Field::Key(s.clone().into_owned().into()),
                Self::Index(idx) => Field::Index(*idx),
            }
        }

        pub(crate) fn as_key(&'a self) -> Option<&'a str> {
            match self {
                Self::Key(s) => Some(s.as_ref()),
                Self::Index(_) => None,
            }
        }

        pub(crate) fn as_usize(&self) -> Option<usize> {
            match self {
                Self::Index(idx) => Some(*idx),
                Self::Key(_) => None,
            }
        }

        pub fn is_key(&self, s: &str) -> bool {
            match self {
                Self::Key(f) => f.eq(s),
                _ => false,
            }
        }

        pub fn is_idx(&self, input: usize) -> bool {
            match self {
                Self::Index(idx) => input.eq(idx),
                _ => false,
            }
        }
    }

    // impl AsRef<Field<'_>> for &str {
    //     fn as_ref(&self) ->
    // }

    // impl AsRef<str> for Field<'_> {
    //     fn as_ref(&self) -> &str {
    //         match self {
    //             Self::Key(s) => s,
    //             _ => unreachable!(),
    //         }
    //     }
    // }

    // impl AsRef<usize> for Field<'_> {
    //     fn as_ref(&self) -> &usize {
    //         match self {
    //             Self::Index(idx) => idx,
    //             _ => unreachable!(),
    //         }
    //     }
    // }

    impl<'a> From<&'a str> for Field<'a> {
        fn from(inner: &'a str) -> Self {
            Self::Key(inner.into())
        }
    }

    // impl From<usize> for Field<'_> {
    //     fn from(inner: usize) -> Self {
    //         Self::Index(inner)
    //     }
    // }

    // impl From<isize> for Field<'_> {
    //     fn from(inner: isize) -> Self {
    //         Self::Index(inner as usize)
    //     }
    // }

    // impl<'a> TryInto<&'a str> for &Field<'a> {
    //     type Error = Error;
    //     fn try_into(self) -> Result<&'a str, Self::Error> {
    //         match self {
    //             Self::Key(inner)
    //             // _ => Err(Error::)
    //         }
    //     }
    // }

    impl TryInto<Int> for &Field<'_> {
        type Error = Error;
        fn try_into(self) -> Result<Int, Self::Error> {
            match self {
                Field::Index(idx) => Ok(*idx as i64),
                Field::Key(s) => Int::from_str(s).map_err(|err| Error::Decoder(err.into())),
            }
        }
    }

    impl Into<String> for Field<'_> {
        fn into(self) -> String {
            match self {
                Self::Index(idx) => idx.to_string(),
                Self::Key(s) => s.to_string(),
            }
        }
    }

    impl<'a> Into<Cow<'a, str>> for &Field<'a> {
        fn into(self) -> Cow<'a, str> {
            match self {
                Field::Key(s) => s.clone(),
                Field::Index(idx) => Cow::from(idx.to_string()),
            }
        }
    }
}

// pub trait Visit<C: Context>: Select<C> {
//     fn visit<F, T: Representation>(
//         &mut self,
//         selector: &Selector,
//         state: SelectorState,
//         ctx: &mut C,
//         op: F,
//     ) -> Result<(), Error>
//     // ) -> Result<Option<T>, Error>
//     where
//         F: Fn(&mut T, &mut C) -> Result<Option<T>, Error>,
//     {
//         unimplemented!()
//     }
//
//     fn flush(
//         &mut self,
//         selector: &Selector,
//         state: SelectorState,
//         ctx: &mut C,
//     ) -> Result<(), Error> {
//         unimplemented!()
//     }
// }

// impl<C, T> Select<C, T> for T
// where
//     C: Context,
//     T: Representation,
//     ContextSeed<'a, C, T, T>: for<'de> DeserializeSeed<'de, Value = Option<T>>,
// {
//     fn select<'a>(
//         selector: &Selector,
//         state: SelectorState,
//         ctx: &mut C,
//     ) -> Result<Option<T>, Error> {
//         let deserializer = ctx.path_decoder(state.path())?;
//         ContextSeed::<'i, C, T>::deserialize((selector, state, ctx).into(), deserializer)
//             .map_err(|err| Error::decoder(err.to_string()))
//     }
//
//     fn patch(
//         &mut self,
//         selector: &Selector,
//         state: SelectorState,
//         dag: T,
//         ctx: &mut C,
//     ) -> Result<(), Error> {
//         unimplemented!()
//     }
// }

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

// /// Wrapper type for visiting a `Deserializer` with a `Selector`.
//
//  TODO: serde impls for Selectors (for intelligently ignoring deserialized data)
//      - SelectorSeed, Into<SelectorSeed> for Selector
//      - impl DeserializeSeed<Value = T> for SelectorSeed for each type,
//      - IgnoredDag<T>, impl Visitor for IgnoredDag<T>
//          - which "validates" the types it receives against its schema before dropping the values
//      - impl Visitor for SelectorSeed
//          - mimics the type's default Visitor
//          - then, in any map/list type, call next_element_seed with SelectorSeed<InnerType>::from(selector)
// pub struct SelectorSeed<'a, V> {
//     selector: &Selector,
//     visitor: V,
// }

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::*;

    schema! {
        #[ipld_attr(internal)]
        #[derive(Clone, Debug)]
        type Nullish null;
    }

    schema! {
        #[ipld_attr(internal)]
        #[derive(Clone, Debug, PartialEq)]
        type Test struct {
            field1 Int,
            field2 String,
        };
    }

    // schema! {
    //     #[ipld_attr(internal)]
    //     // pub type ExploreUnion null;
    //     pub type ExploreUnion2 [nullable Selector];
    // }

    #[test]
    fn it_works() {
        let t = Test {
            field1: Int::from(0),
            field2: String::default(),
        };

        // let executor = Executor

        // let sel1 = selector! {
        //     #[ipld_attr(internal)]
        //     Test,
        //     match(
        //         label=("label")
        //     )
        // };

        // let sel1 = Selector::Matcher({ Matcher { label: None } });
        // let Selector::Matcher(matcher) = sel1;

        // let selection = <Test as Select<_, Matcher>>::select(t, &matcher);

        assert_eq!(true, true);
    }
}