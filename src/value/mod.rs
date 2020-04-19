//! A general `Value` type, representing all IPLD data model kinds.

// pub mod borrowed;
// mod canon;
mod primitive;
mod recursive;

// pub use recursive::{List, Map};

use crate::dev::macros::derive_more::{Add, AsRef, From, Into, Mul, Sum};
use ipld_macros::schema;

schema! {
    #[ipld_attr(internal)]
    #[derive(Debug, Eq, Hash, PartialEq)]
    pub type Null null;
}

schema! {
    #[ipld_attr(internal)]
    #[derive(AsRef, Debug, Eq, From, Hash, Into, PartialEq, Ord, PartialOrd, Add, Mul, Sum)]
    // #[from(forward)]
    #[as_ref(forward)]
    pub type Int int;
}

schema! {
    #[ipld_attr(internal)]
    #[derive(AsRef, Debug, From, Into, PartialEq, PartialOrd, Add, Mul, Sum)]
    #[as_ref(forward)]
    // #[from(forward)]
    pub type Float float;
}

schema! {
    #[ipld_attr(internal)]
    #[derive(AsRef, Debug, From, Eq, Hash, PartialEq)]
    #[as_ref(forward)]
    pub type Bytes bytes;
}

// schema! {
//     #[ipld_macros_internal]
//     pub type Value union {
//         | Null null
//         | Int int
//         | Float float
//         | String string
//         | Bytes bytes
//         | List list
//         | Map map
//         | Link link
//     } representation kinded;
// }
