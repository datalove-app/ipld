//! A general `Value` type, representing all IPLD data model kinds.

// pub mod borrowed;
// mod canon;
mod link;
mod list;
mod map;
mod primitive;
// mod recursive;

use std::{boxed::Box, path::Path};

pub use _serde::IgnoredAny;
pub use link::Link;
pub use list::List;
pub use map::Map;
pub use primitive::*;

use crate::dev::*;
use macros::derive_more::{From, IsVariant, TryInto, Unwrap};

///
#[derive(Clone, Debug, IsVariant, Unwrap)]
pub enum Dag<T: Representation> {
    Value(Value),
    Type(T),
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From, PartialEq, TryInto, IsVariant, Unwrap)]
    // #[from(forward)]
    #[try_into(owned, ref, ref_mut)]
    // TODO: impl from(forward) and try_into for all unions and enums
    pub type Value union {
        #[from(ignore)]
        | Null null
        | Bool bool
        | Int int
        | Float float
        | String string
        | Bytes bytes
        | List<Value> list
        | Map<String, Value> map
        | Link<DEFAULT_MULTIHASH_SIZE, BoxedValue> link
    } representation kinded;
}

/// A shorthand type alias for any valid IPLD data model type.
pub type Any = Value;

///
pub type BoxedValue = Box<Value>;

/// A constant `Value::Null`.
pub const NULL_VALUE: Value = Value::Null;

impl Value {
    /// LookupByString looks up a child object in this node and returns it.
    /// The returned Node may be any of the Kind:
    /// a primitive (string, int64, etc), a map, a list, or a link.
    ///
    /// If the Kind of this Node is not Kind_Map, a nil node and an error
    /// will be returned.
    ///
    /// If the key does not exist, a nil node and an error will be returned.
    pub fn lookup_by_string(&self, key: &str) -> Result<&Self, Error> {
        match self {
            Self::Map(inner) => unimplemented!(),
            _ => Err(Error::Value("Value must be a map")),
        }
    }

    /// LookupByNode is the equivalent of LookupByString, but takes a reified Node
    /// as a parameter instead of a plain string.
    /// This mechanism is useful if working with typed maps (if the key types
    /// have constraints, and you already have a reified `schema.TypedNode` value,
    /// using that value can save parsing and validation costs);
    /// and may simply be convenient if you already have a Node value in hand.
    ///
    /// (When writing generic functions over Node, a good rule of thumb is:
    /// when handling a map, check for `schema.TypedNode`, and in this case prefer
    /// the LookupByNode(Node) method; otherwise, favor LookupByString; typically
    /// implementations will have their fastest paths thusly.)
    pub fn lookup_by_node<T>(&self, key: &T) -> Result<&Self, Error> {
        unimplemented!()
    }

    /// LookupByIndex is the equivalent of LookupByString but for indexing into a list.
    /// As with LookupByString, the returned Node may be any of the Kind:
    /// a primitive (string, int64, etc), a map, a list, or a link.
    ///
    /// If the Kind of this Node is not Kind_List, a nil node and an error
    /// will be returned.
    ///
    /// If idx is out of range, a nil node and an error will be returned.
    pub fn lookup_by_index(&self, idx: usize) -> Result<&Self, Error> {
        unimplemented!()
    }

    /// LookupBySegment is will act as either LookupByString or LookupByIndex,
    /// whichever is contextually appropriate.
    ///
    /// Using LookupBySegment may imply an "atoi" conversion if used on a list node,
    /// or an "itoa" conversion if used on a map node.  If an "itoa" conversion
    /// takes place, it may error, and this method may return that error.
    pub fn lookup_by_segment(&self, seg: &Path) -> Result<&Self, Error> {
        unimplemented!()
    }

    // /// MapIterator returns an iterator which yields key-value pairs
    // /// traversing the node.
    // /// If the node kind is anything other than a map, nil will be returned.
    // ///
    // /// The iterator will yield every entry in the map; that is, it
    // /// can be expected that itr.Next will be called node.Length times
    // /// before itr.Done becomes true.
    // pub fn map_iterator(&self) -> impl Iterator<Item = (&str, &Self)> {
    //     unimplemented!()
    // }

    // /// ListIterator returns an iterator which traverses the node and yields indicies and list entries.
    // /// If the node kind is anything other than a list, nil will be returned.
    // ///
    // /// The iterator will yield every entry in the list; that is, it
    // /// can be expected that itr.Next will be called node.Length times
    // /// before itr.Done becomes true.
    // ///
    // /// List iteration is ordered, and indices yielded during iteration will range from 0 to Node.Length-1.
    // /// (The IPLD Data Model definition of lists only defines that it is an ordered list of elements;
    // /// the definition does not include a concept of sparseness, so the indices are always sequential.)
    // pub fn list_iterator(&self) -> impl Iterator<Item = &Self> {
    //     unimplemented!()
    // }

    /// Length returns the length of a list, or the number of entries in a map,
    /// or -1 if the node is not of list nor map kind.
    pub fn len(&self) -> usize {
        unimplemented!()
    }

    /// Absent nodes are returned when traversing a struct field that is
    /// defined by a schema but unset in the data.  (Absent nodes are not
    /// possible otherwise; you'll only see them from `schema.TypedNode`.)
    /// The absent flag is necessary so iterating over structs can
    /// unambiguously make the distinction between values that are
    /// present-and-null versus values that are absent.
    ///
    /// Absent nodes respond to `Kind()` as `ipld.Kind_Null`,
    /// for lack of any better descriptive value; you should therefore
    /// always check IsAbsent rather than just a switch on kind
    /// when it may be important to handle absent values distinctly.
    pub fn is_absent(&self) -> bool {
        unimplemented!()
    }

    pub fn as_bool(&self) -> Result<bool, Error> {
        unimplemented!()
    }
    pub fn as_int(&self) -> Result<&Int, Error> {
        unimplemented!()
    }
    pub fn as_float(&self) -> Result<&Float, Error> {
        unimplemented!()
    }
    pub fn as_string(&self) -> Result<&str, Error> {
        unimplemented!()
    }
    pub fn as_bytes(&self) -> Result<&Bytes, Error> {
        unimplemented!()
    }
    pub fn as_link(&self) -> Result<Link<DEFAULT_MULTIHASH_SIZE, Self>, Error> {
        unimplemented!()
    }

    // Prototype returns a NodePrototype which can describe some properties of this node's implementation,
    // and also be used to get a NodeBuilder,
    // which can be use to create new nodes with the same implementation as this one.
    //
    // For typed nodes, the NodePrototype will also implement schema.Type.
    //
    // For Advanced Data Layouts, the NodePrototype will encapsulate any additional
    // parameters and configuration of the ADL, and will also (usually)
    // implement NodePrototypeSupportingAmend.
    //
    // Calling this method should not cause an allocation.
    // Prototype() NodePrototype
}

mod _serde {
    use crate::dev::*;
    use std::marker::PhantomData;

    ///
    #[derive(Copy, Clone, Debug, Default)]
    pub struct Ignored<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Deserialize<'de> for Ignored<T> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // deserializer.deserialize_ignored_any(Self)
            // T::deserialize
            unimplemented!()
        }
    }

    ///
    #[derive(Copy, Clone, Debug, Default)]
    pub struct IgnoredAny;

    impl<'de> Visitor<'de> for IgnoredAny {
        type Value = IgnoredAny;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("anything at all")
        }

        #[inline]
        fn visit_bool<E>(self, x: bool) -> Result<Self::Value, E> {
            let _ = x;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_i64<E>(self, x: i64) -> Result<Self::Value, E> {
            let _ = x;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_u64<E>(self, x: u64) -> Result<Self::Value, E> {
            let _ = x;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_f64<E>(self, x: f64) -> Result<Self::Value, E> {
            let _ = x;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let _ = s;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            IgnoredAny::deserialize(deserializer)
        }

        #[inline]
        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            IgnoredAny::deserialize(deserializer)
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(IgnoredAny) = seq.next_element()? {
                // Gobble
            }
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            while let Some((IgnoredAny, IgnoredAny)) = map.next_entry()? {
                // Gobble
            }
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let _ = bytes;
            Ok(IgnoredAny)
        }

        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: EnumAccess<'de>,
        {
            data.variant::<IgnoredAny>()?.1.newtype_variant()
        }
    }

    impl Serialize for IgnoredAny {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            unreachable!()
        }
    }

    impl<'de> Deserialize<'de> for IgnoredAny {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<IgnoredAny, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_ignored_any(Self)
        }
    }

    impl Representation for IgnoredAny {
        const NAME: &'static str = "IgnoredAny";
        const SCHEMA: &'static str = "type IgnoredAny null";
        const KIND: Kind = Kind::Null;
    }

    // TODO:
    // impl<C: Context> Select<C, IgnoredAny> for IgnoredAny {
    //     // create seed for root block
    //     // select_in block, producing a list of selections
    //     // if selector is not a matcher, selection must continue
    //     //
    //     #[inline]
    //     fn select(seed: SelectorSeed, ctx: &mut C) -> Result<(), Error> {
    //         unimplemented!()
    //     }

    //     fn select_dag(seed: SelectorSeed, ctx: &mut C) -> Result<Self, Error> {
    //         unimplemented!()
    //     }

    //     fn patch(&mut self, seed: SelectorSeed, dag: Self, ctx: &mut C) -> Result<(), Error> {
    //         unimplemented!()
    //     }
    // }

    // impl<T: Select<>, C: Context> Select<IgnoredAny, C> for IgnoredAny {
    //     // create seed for root block
    //     // select_in block, producing a list of selections
    //     // if selector is not a matcher, selection must continue
    //     //
    //     #[inline]
    //     fn select(seed: SelectorSeed, ctx: &mut C) -> Result<(), Error> {
    //         Null::select(seed, ctx)
    //     }
    // }
}
