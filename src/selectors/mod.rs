//! IPLD Selectors
//!
//! TODO:
//!     - selectors are types that impl Representation (can be defined with `schema!`)
//!     - macro can compile selector string to a type
//!     - type implements Context
#![allow(non_camel_case_types)]

// mod path;
mod context;
mod schema;
mod seed;
mod state;

pub use context::*;
pub use field::*;
pub use schema::*;
pub use seed::*;
pub use state::*;

use crate::dev::*;
use macros::derive_more::From;
use serde::de::DeserializeSeed;
use std::path::{Path, PathBuf};

///
/// TODO:
pub trait Select<C: Context>: Representation + 'static {
    // ///
    // fn selectable<S: Select<C>>() -> bool {
    //     type_eq2::<true, Self, S>()
    // }

    /// Produces a stream of [`Selection`]s of some type `S`.
    ///
    /// General impl flow:
    ///     - select is given a context (seed) that can provide a block
    ///     - grab the current block deserializer, use the seed
    ///         - until reaching a link, everything is in serde
    ///
    fn select(
        // selector: &Selector,
        // state: &mut SelectorState,
        // params: SelectionParams<'_, C, Self, S>,
        // ctx: &mut C,
        // seed: ContextSeed<'_, C, Self>,
        params: SelectionParams<'_, C, Self>,
        ctx: &mut C,
    ) -> Result<(), Error>;

    // fn patch<S: Select<C>>(seed: ContextSeed<'_, C, Self, S>) -> Result<(), Error> {
    //     unimplemented!()
    // }
}

pub fn select_from_seed<const S: usize, C, T>(
    params: SelectionParams<'_, S, C, T>,
    mut ctx: &mut C,
) -> Result<(), Error>
where
    C: Context,
    T: Representation + 'static,
    for<'a, 'de> ContextSeed<'a, C, T>: DeserializeSeed<'de, Value = ()>,
{
    let SelectionParams {
        cid,
        selector,
        max_path_depth,
        max_link_depth,
        callback,
    } = params;
    let mut state = SelectionState {
        max_path_depth,
        max_link_depth,
        ..Default::default()
    };
    let seed = ContextSeed {
        selector: &selector,
        state: &mut state,
        callback,
        ctx: &mut ctx,
    };
    {
        seed.read(&cid)?;
    }

    Ok(())
}

pub struct SelectionParams<'a, C, T> {
    pub cid: Cid,
    pub selector: &'a Selector,
    pub max_path_depth: Option<usize>,
    pub max_link_depth: Option<usize>,
    pub(crate) callback: SelectionCallback<'a, C, T>,
}

impl<'a, C, T> SelectionParams<'a, C, T>
where
    C: Context,
    T: Representation,
{
    // pub fn into_seed(self, ctx: &'a mut C) -> ContextSeed<'a, C, T> {
    //     ContextSeed {
    //         selector,
    //         callback,
    //         ctx,
    //     }
    // }

    //     pub fn new_node_config<F>(cid: CidGeneric<S>, callback: F) -> Self
    //     where F: SelectNodeOp<C> {
    //         Self {
    //             cid,
    //             max_path_depth: None,
    //             max_link_depth: None,
    //             callback: Selection,
    //         }
    //     }

    //     pub fn new_dag_config(cid: CidGeneric<S>) -> Self {
    //         Self {
    //             cid,
    //             max_path_depth: None,
    //             max_link_depth: None,
    //             callback: Default::default(),
    //         }
    //     }

    //     ///
    //     #[inline]
    //     pub fn with_max_path_depth(mut self, max_path_depth: usize) -> Self {
    //         self.max_path_depth = Some(max_path_depth);
    //         self
    //     }

    //     ///
    //     #[inline]
    //     pub fn with_max_link_depth(mut self, max_link_depth: usize) -> Self {
    //         self.max_link_depth = Some(max_link_depth);
    //         self
    //     }
}

///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectedNode {
    pub path: PathBuf,
    pub node: Node,
    pub matched: bool,
    pub label: Option<String>,
}

///
#[derive(Debug)]
pub struct SelectedDag {
    path: PathBuf,
    dag: Box<dyn ErasedRepresentation>,
    label: Option<String>,
}

impl SelectedDag {
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
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
    pub fn downcast_as<T>(&self) -> Option<&T>
    where
        T: Representation + 'static,
    {
        (*self.dag).as_any().downcast_ref()
    }
}

impl<T> Into<(PathBuf, Option<T>, Option<String>)> for SelectedDag
where
    T: Representation + 'static,
{
    fn into(self) -> (PathBuf, Option<T>, Option<String>) {
        let dag = self.dag.into_any().downcast().ok().map(|t| *t);
        (self.path, dag, self.label)
    }
}

///
#[derive(Clone, Debug, From, Deserialize, Serialize)]
// #[from(forward)]
pub enum Node {
    ///
    #[serde(rename = "null")]
    Null,

    ///
    #[serde(rename = "bool")]
    Bool(bool),

    ///
    #[serde(rename = "int8")]
    Int8(i8),

    ///
    #[serde(rename = "int16")]
    Int16(i16),

    ///
    #[serde(rename = "int")]
    Int(Int),

    ///
    #[serde(rename = "int64")]
    Int64(i64),

    ///
    #[serde(rename = "int128")]
    Int128(i128),

    ///
    #[serde(rename = "uint8")]
    Uint8(u8),

    ///
    #[serde(rename = "uint16")]
    Uint16(u16),

    ///
    #[serde(rename = "uint32")]
    Uint32(u32),

    ///
    #[serde(rename = "uint64")]
    Uint64(u64),

    ///
    #[serde(rename = "uint128")]
    Uint128(u128),

    ///
    #[serde(rename = "float32")]
    Float32(f32),

    ///
    #[serde(rename = "float")]
    Float(Float),

    ///
    #[serde(rename = "string")]
    String(String),

    ///
    #[serde(rename = "bytes")]
    Bytes(crate::dev::Bytes),

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

impl<T: Representation> From<Link<T>> for Node {
    fn from(link: Link<T>) -> Self {
        Self::Link(link.into())
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

mod field {
    use super::*;

    /// Wrapper type for types that can be used as dag keys or indices.
    pub(crate) enum Field<'a> {
        Key(&'a str),
        Index(usize),
    }

    impl<'a> Field<'a> {
        pub fn append_to_path(&self, path: &mut PathBuf) {
            match self {
                Self::Key(s) => path.push(s),
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

    impl<'a> Into<Field<'a>> for &'a str {
        fn into(self) -> Field<'a> {
            Field::Key(self)
        }
    }

    impl<'a> Into<Field<'a>> for usize {
        fn into(self) -> Field<'a> {
            Field::Index(self)
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
