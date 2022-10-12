use crate::dev::*;
use macros::{
    derive_more::{From, IsVariant, TryInto, Unwrap},
    repr_serde,
};
use maybestd::{fmt, rc::Rc};
use std::path::Path;

// ///
// #[derive(Clone, Debug, IsVariant, Unwrap)]
// pub enum Dag<T: Representation> {
//     Value(Any),
//     Type(T),
// }

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From, PartialEq, TryInto,
        // IsVariant, Unwrap
    )]
    // #[from(forward)]
    #[try_into(owned, ref, ref_mut)]
    // TODO: impl from(forward) and try_into for all unions and enums
    pub type Any union {
        ///
        #[from(ignore)]
        | Null null
        ///
        | Bool bool
        ///
        | Int int
        ///
        | Float float
        ///
        | String string
        ///
        | Bytes bytes
        ///
        | List list
        ///
        | Map map
        ///
        // todo? should this be listed above? decoding untagged variants is attempted in def order
        #[ipld_attr(wrapper = "Rc")]
        | Link link
    } representation kinded;
}

// repr_serde! { @visitor T T { type_kinds::Any }
//     { T: TryFrom<Any> + 'static } {}
// {
//     #[inline]
//     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}, a boolean type", Bool::NAME)
//     }
// }}

// repr_serde! { @visitor_ext T T { type_kinds::Any } { T: TryFrom<Any> + 'static } {} {}}

/// TODO: convert these to a Node trait, that all types implement
impl Any {
    /// LookupByString looks up a child object in this node and returns it.
    /// The returned Node may be any of the Kind:
    /// a primitive (string, int64, etc), a map, a list, or a link.
    ///
    /// If the Kind of this Node is not Kind_Map, a nil node and an error
    /// will be returned.
    ///
    /// If the key does not exist, a nil node and an error will be returned.
    pub fn lookup_by_string(&self, key: &str) -> Result<&Self, Error> {
        // match self {
        //     Self::Map(inner) => unimplemented!(),
        //     _ => Err(Error::Value("Value must be a map")),
        // }
        unimplemented!()
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
    pub fn as_link(&self) -> Result<Link<Self>, Error> {
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
