//! A general `Value` type, representing all IPLD data model kinds.

// pub mod borrowed;
// mod canon;
mod link;
mod list;
mod map;
mod primitive;
// mod recursive;

pub use link::Link;
pub use list::List;
pub use map::Map;
pub use primitive::*;

use crate::dev::*;
use macros::derive_more::{Add, AsRef, From, Into, Mul, Sum};

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, PartialEq)]
    pub type Value union {
        | Null null
        | Bool bool
        | Int int
        | Float float
        | String string
        | Bytes bytes
        | List<Value> list
        | Map<String, Value> map
        | Link<Value> link
    } representation kinded;
}

schema! {
    /// The `null` type.
    #[ipld_attr(internal)]
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    pub type Null null;
}

schema! {
    /// The `bool` type.
    #[ipld_attr(internal)]
    #[derive(AsRef, Copy, Clone, Debug, Eq, From, Hash, PartialEq)]
    #[as_ref(forward)]
    #[from(forward)]
    pub type Bool bool;
}

schema! {
    /// A `bytes` type.
    #[ipld_attr(internal)]
    #[derive(AsRef, Clone, Debug, Eq, From, Hash, PartialEq)]
    #[as_ref(forward)]
    #[from(forward)]
    pub type Bytes bytes;
}

macro_rules! def_num {
    (@int $name:ident $type:ident $doc_str:expr) => {
        schema! {
            #[doc = $doc_str]
            #[ipld_attr(internal)]
            #[derive(AsRef, Copy, Clone, Debug, From, Hash, Into, Eq, PartialEq, Ord, PartialOrd, Add, Mul, Sum)]
            #[as_ref(forward)]
            pub type $name $type;
        }
    };
    (@float $name:ident $type:ident $doc_str:expr) => {
        schema! {
            #[doc = $doc_str]
            #[ipld_attr(internal)]
            #[derive(AsRef, Clone, Debug, From, Into, PartialEq, PartialOrd, Add, Mul, Sum)]
            #[as_ref(forward)]
            pub type $name $type;
        }
    };
}

/// A shorthand type alias for an `Int32`.
pub type Int = Int32;
/// A shorthand type alias for a `Float64`.
pub type Float = Float64;

def_num!(@int Int8 int8 "A fixed-length number type representing an int8");
def_num!(@int Int16 int16 "A fixed-length number type representing an int16");
def_num!(@int Int32 int32 "A fixed-length number type representing an int32");
def_num!(@int Int64 int64 "A fixed-length number type representing an int64");
def_num!(@int Int128 int128 "A fixed-length number type representing an int128");
def_num!(@int Uint8 uint8 "A fixed-length number type representing an uint8");
def_num!(@int Uint16 uint16 "A fixed-length number type representing an uint16");
def_num!(@int Uint32 uint32 "A fixed-length number type representing an uint32");
def_num!(@int Uint64 uint64 "A fixed-length number type representing an uint64");
def_num!(@int Uint128 uint128 "A fixed-length number type representing an uint128");
def_num!(@float Float32 float32 "A fixed-length number type representing a float32");
def_num!(@float Float64 float64 "A fixed-length number type representing a float64");
