//! A general `Value` type, representing all IPLD data model kinds.

// pub mod borrowed;
// mod canon;
// mod link;
mod list;
mod map;
mod primitive;
mod recursive;

// pub use link::Link;
pub use list::List;
pub use map::Map;

use crate::dev::*;
use macros::derive_more::{Add, AsRef, From, Into, Mul, Sum};

// schema! {
//     ///
//     #[ipld_attr(internal)]
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

schema! {
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub type Null null;
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(AsRef, Clone, Debug, Eq, From, Hash, PartialEq)]
    #[as_ref(forward)]
    #[from(forward)]
    pub type Bytes bytes;
}

macro_rules! def_num {
    (@int $name:ident $type:ident) => {
        schema! {
            #[ipld_attr(internal)]
            #[derive(AsRef, Debug, Eq, From, Hash, Into, PartialEq, Ord, PartialOrd, Add, Mul, Sum)]
            #[as_ref(forward)]
            pub type $name $type;
        }
    };
    (@float $name:ident $type:ident) => {
        schema! {
            #[ipld_attr(internal)]
            #[derive(AsRef, Debug, From, Into, PartialEq, PartialOrd, Add, Mul, Sum)]
            #[as_ref(forward)]
            pub type $name $type;
        }
    };
}

def_num!(@int Int int32);
def_num!(@int Int8 int8);
def_num!(@int Int16 int16);
def_num!(@int Int32 int32);
def_num!(@int Int64 int64);
def_num!(@int Int128 int128);
def_num!(@int Uint8 uint8);
def_num!(@int Uint16 uint16);
def_num!(@int Uint32 uint32);
def_num!(@int Uint64 uint64);
def_num!(@int Uint128 uint128);
def_num!(@float Float float64);
def_num!(@float Float32 float32);
def_num!(@float Float64 float64);
