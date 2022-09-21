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
pub use field::*;
pub use params::*;
pub use seed::*;
pub use selection::*;
pub use selectors::*;
pub use state::*;

use crate::dev::*;
use macros::derive_more::From;
use serde::de::DeserializeSeed;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    vec::IntoIter,
};

///
/// TODO: + 'static?
///     - Selectable?
pub trait Select<C: Context>: Representation {
    /// Produces a stream of [`Selection`]s of some type `T`.
    ///
    /// Under the hood, this serves as the entrypoint for deserialization of a
    /// block via a typed `ContextSeed`: a type that implements
    /// [`DeserializeSeed`] for each [`Select`]-able type, and uses the
    /// contained [`Selector`], the type's [`Representation`] and the provided
    /// [`Context`] to govern how to interpret the types found in blocks.
    ///
    ///
    /// TODO: update this interface, since ContextSeed is doing the work and it should be refactored a bit (borrow state, )
    fn select(params: Params<'_, C, Self>, ctx: &mut C) -> Result<(), Error>;

    /// Selects against the dag, loading more blocks from `C` if required.
    ///
    /// TODO
    fn select_in<T>(&self, params: Params<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
        unimplemented!()
    }

    /// Patches the dag according to the selector, loading more blocks from `C`
    /// if required.
    ///
    /// TODO
    fn patch_in<T>(&mut self, params: Params<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
        unimplemented!()
    }

    /// Flushes the dag according to the selector, writing blocks to `C` if
    /// flushing linked dags.
    ///
    /// TODO
    fn flush(&mut self, params: Params<'_, C, Self>, ctx: &mut C) -> Result<(), Error> {
        unimplemented!()
    }

    // fn patch<S: Select<C>>(seed: ContextSeed<'_, C, Self, S>) -> Result<(), Error> {
    //     unimplemented!()
    // }
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
    pub struct Params<'a, C, T = Any>
    where
        C: Context,
        T: Representation,
    {
        pub(crate) cid: Option<Cid>,
        pub(crate) selector: Option<&'a Selector>,
        pub(crate) max_path_depth: Option<usize>,
        pub(crate) max_link_depth: Option<usize>,
        pub(crate) callback: Callback<'a, C, T>,
    }

    impl<'a, C, T> Default for Params<'a, C, T>
    where
        C: Context,
        T: Representation,
    {
        fn default() -> Self {
            Self {
                cid: None,
                selector: None,
                max_path_depth: None,
                max_link_depth: None,
                callback: Default::default(),
            }
        }
    }

    impl<'a, C, T> Params<'a, C, T>
    where
        C: Context,
        T: Representation,
    {
        ///
        pub fn new_select(cid: Cid) -> Self {
            Self {
                cid: Some(cid),
                ..Self::default()
            }
        }

        ///
        pub fn with_root(mut self, cid: Cid) -> Self {
            self.cid.replace(cid);
            self
        }

        ///
        pub fn with_selector(mut self, selector: &'a Selector) -> Self {
            self.selector.replace(selector);
            self
        }

        ///
        pub fn with_max_path_depth(mut self, max_path_depth: usize) -> Self {
            self.max_path_depth.replace(max_path_depth);
            self
        }

        ///
        pub fn with_max_link_depth(mut self, max_link_depth: usize) -> Self {
            self.max_link_depth.replace(max_link_depth);
            self
        }

        ///
        pub fn into_node_iter(
            self,
            only_matched: bool,
            ctx: &mut C,
        ) -> Result<IntoIter<NodeSelection>, Error>
        where
            T: Select<C>,
        {
            let vec = RefCell::new(Vec::new());
            let params = Params {
                callback: Callback::SelectNode {
                    only_matched,
                    cb: Box::new(|node, _| {
                        vec.borrow_mut().push(node);
                        Ok(())
                    }),
                },
                ..self
            };

            T::select(params, ctx)?;
            Ok(vec.into_inner().into_iter())
        }

        ///
        /// TODO: make this more like an actual iterator, that can pause across links
        pub fn into_dag_iter(self, ctx: &mut C) -> Result<IntoIter<DagSelection>, Error>
        where
            T: Select<C>,
        {
            let vec = RefCell::new(Vec::new());
            let params = Params {
                callback: Callback::SelectDag {
                    cb: Box::new(|node, _| {
                        vec.borrow_mut().push(node);
                        Ok(())
                    }),
                },
                ..self
            };

            T::select(params, ctx)?;
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

mod selection {
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
        pub fn new<T>(path: &Path, node: T) -> Self
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
        pub fn new_match<T>(path: &Path, node: T, label: Option<&str>) -> Self
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

    ///
    pub struct DagSelection {
        pub path: PathBuf,
        pub dag: AnyRepresentation,
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
                dag: dag.into(),
                label: label.map(str::to_string),
            }
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

    ///
    #[derive(Clone, Debug, From, Deserialize, Serialize)]
    // #[from(forward)]
    pub enum SelectedNode {
        ///
        #[serde(rename = "null")]
        Null,

        ///
        #[serde(rename = "bool")]
        Bool(bool),

        ///
        #[serde(rename = "int8")]
        Int8(Int8),

        ///
        #[serde(rename = "int16")]
        Int16(Int16),

        ///
        #[serde(rename = "int32")]
        Int32(Int32),

        ///
        #[serde(rename = "int64")]
        Int64(Int64),

        ///
        #[serde(rename = "int")]
        Int128(Int128),

        ///
        #[serde(rename = "uint8")]
        Uint8(Uint8),

        ///
        #[serde(rename = "uint16")]
        Uint16(Uint16),

        ///
        #[serde(rename = "uint32")]
        Uint32(Uint32),

        ///
        #[serde(rename = "uint64")]
        Uint64(Uint64),

        ///
        #[serde(rename = "uint128")]
        Uint128(Uint128),

        ///
        #[serde(rename = "float32")]
        Float32(Float32),

        ///
        #[serde(rename = "float64")]
        Float64(Float64),

        ///
        #[serde(rename = "string")]
        String(IpldString),

        ///
        #[serde(rename = "bytes")]
        Bytes(Bytes),

        ///
        #[serde(rename = "list")]
        #[from(ignore)]
        List,

        ///
        #[serde(rename = "map")]
        #[from(ignore)]
        Map,

        ///
        #[serde(rename = "link")]
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

    impl<T: Representation> From<List<T>> for SelectedNode {
        fn from(_: List<T>) -> Self {
            Self::List
        }
    }

    impl<K: Representation, V: Representation> From<Map<K, V>> for SelectedNode {
        fn from(_: Map<K, V>) -> Self {
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

    impl From<Any> for SelectedNode {
        fn from(val: Any) -> Self {
            match val {
                Any::Null(_) => Self::Null,
                Any::Bool(inner) => Self::Bool(inner),
                Any::Int(inner) => Self::Int64(inner),
                Any::Float(inner) => Self::Float64(inner),
                Any::String(inner) => Self::String(inner),
                Any::Bytes(inner) => Self::Bytes(inner),
                Any::List(_) => Self::List,
                Any::Map(_) => Self::Map,
                Any::Link(link) => Self::Link(*link.as_ref().cid()),
            }
        }
    }
}

mod field {
    use super::*;

    /// Wrapper type for types that can be used as dag keys or indices.
    pub(crate) enum Field<'a> {
        Key(&'a str),
        // CidKey(&'a Cid),
        Index(usize),
    }

    impl<'a> Field<'a> {
        pub fn append_to_path(&self, path: &mut PathBuf) {
            match self {
                Self::Key(s) => path.push(s),
                // Self::CidKey(c) => path.push(c.to_string()),
                Self::Index(idx) => path.push(idx.to_string()),
            }
        }
    }

    impl<'a> AsRef<str> for Field<'a> {
        fn as_ref(&self) -> &str {
            match self {
                Self::Key(s) => s,
                _ => unreachable!(),
            }
        }
    }

    impl<'a> AsRef<usize> for Field<'a> {
        fn as_ref(&self) -> &usize {
            match self {
                Self::Index(idx) => idx,
                _ => unreachable!(),
            }
        }
    }

    impl<'a> From<&'a str> for Field<'a> {
        fn from(inner: &'a str) -> Self {
            Self::Key(inner)
        }
    }

    impl<'a> From<usize> for Field<'a> {
        fn from(inner: usize) -> Self {
            Self::Index(inner)
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
    use crate::prelude::*;

    schema! {
        #[ipld_attr(internal)]
        #[derive(Debug)]
        type Nullish null;
    }

    schema! {
        #[ipld_attr(internal)]
        #[derive(Debug, PartialEq)]
        type Test struct {
            field1 Int,
            field2 String,
        };
    }

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
