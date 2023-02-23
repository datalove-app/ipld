use crate::dev::*;
use macros::{
    derive_more::{From, IsVariant, TryInto, Unwrap},
    repr_serde,
};
use maybestd::{fmt, rc::Rc};

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
        // #[from(ignore)]
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

// #[derive(
//     Clone,
//     Debug,
//     // From,
//     PartialEq,
//     // TryInto,
//     // IsVariant, Unwrap
//     Representation,
//     Select,
// )]
// #[ipld(internal)]
// // #[from(forward)]
// // #[try_into(owned, ref, ref_mut)]
// pub enum Any {
//     Null(Null),
//     Bool(Bool),
//     Int(Int),
//     Float(Float),
//     String(String),
//     Bytes(Bytes),
//     List(List<Any>),
//     Map(Map<String, Any>),
//     Link(Link<Rc<Any>>),
// }

// impl FromIterator for Any, (T: StringRepresentation, Any)

#[cfg(feature = "dep:rkyv")]
mod rkyv {}
