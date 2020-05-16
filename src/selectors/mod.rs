//! IPLD Selectors
//!
//! TODO:
//!     - selectors are types that impl Representation (can be defined with `schema!`)
//!     - macro can compile selector string to a type
//!     - type implements Context
#![allow(non_camel_case_types)]
use crate::dev::*;
use macros::derive_more::From;
use std::{marker::PhantomData, ops::Deref, rc::Rc};

//  TODO? example impl for ExploreAll for a map:
//  impl<K, V, S> Select<ExploreAll> for Map<K, V>
//  where
//      V: Select<S>
//  {
//      type SelectorArgs = <V as Select<S>>::Args;
//      fn new_selector(args: Self::SelectorArgs) -> Selector {
//          let sel = Selector::new_selector::<Ctx, S, V>(args);
//          ExploreAll::from(sel)
//      }
//  }
//  TODO? example impl for ExploreFields for a map:
//  impl<K, V, S> Select<ExploreFields> for Map<K, V>
//  where
//      V: Select<S>
//  {
//      type SelectorArgs = (ExploreFields, <V as Select<S>::Args)];
//      fn new_selector(args: Self::SelectorArgs) -> Selector {
//          let mut map = BTreeMap::new();
///
//          for (field, args) in args {
//              map.insert(field, <V as Select<S>>::new_selector(args));
//          }
///
//          Selector::from(ExploreFields::from(map))
//      }
//  }
///
//  TODO? example impl for ExploreFields for a struct:
//  impl<S> Select<ExploreField<"field1", S>> for Struct {
//      type SelectorArgs = (
//          ExploreFields,
//          <SomeInnerTypeOfStruct as Select<S>::Args,
//      );
//      fn new_selector(args: Self::SelectorArgs) -> Selector {
//          let sel = <SomeInnerTypeOfStruct as Select<S>>::new_selector(args.1);
//          args.0.insert("field1", sel);
//          Selector::from(args.0)
//      }
//  }

// impl From<SelectorEnvelope> for Selector;
// impl From<Selector> for SelectorEnvelope;

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
//     selector: &'a Selector,
//     visitor: V,
// }

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

schema! {
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type SelectorEnvelope union {
        | Selector "selector"
    } representation keyed;
}

schema! {
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type Selector union {
        ///
        | Matcher "."

        ///
        #[ipld_attr(wrapper = "Rc")]
        | ExploreAll "a"

        ///
        | ExploreFields "f"

        ///
        #[ipld_attr(wrapper = "Rc")]
        | ExploreIndex "i"

        ///
        #[ipld_attr(wrapper = "Rc")]
        | ExploreRange "r"

        ///
        #[ipld_attr(wrapper = "Rc")]
        | ExploreRecursive "R"

        ///
        | ExploreUnion "|"

        ///
        #[ipld_attr(wrapper = "Rc")]
        | ExploreConditional "&"

        ///
        | ExploreRecursiveEdge "@"
    } representation keyed;
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type ExploreAll struct {
        pub next Selector (rename ">"),
    };
}

schema! {
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type ExploreFields struct {
        // fields {String:Selector} (rename "f>"),
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type ExploreIndex struct {
        pub index Int (rename "i"),
        pub next Selector (rename ">"),
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type ExploreRange struct {
        pub start Int (rename "^"),
        pub end Int (rename "$"),
        pub next Selector (rename ">"),
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type ExploreRecursive struct {
        pub sequence Selector (rename ":>"),
        pub limit RecursionLimit (rename "l"),
        stopAt optional Condition (rename "!"),
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type RecursionLimit union {
        | RecursionLimit_None "none"
        | RecursionLimit_Depth "depth"
    } representation keyed;
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type RecursionLimit_None struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type RecursionLimit_Depth int;
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type ExploreRecursiveEdge struct {};
}

schema! {
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type ExploreUnion null;
    // TODO: pub type ExploreUnion [Selector];
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type ExploreConditional struct {
        pub condition Condition (rename "&"),
        pub next Selector (rename ">"),
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type Matcher struct {
        // onlyIf optional Condition,
        pub label optional String,
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug, From)]
    pub type Condition union {
        | Condition_HasField "hasField"
        | Condition_HasValue "="
        | Condition_HasKind "%"
        | Condition_IsLink "/"
        | Condition_GreaterThan "greaterThan"
        | Condition_LessThan "lessThan"
        | Condition_And "and"
        | Condition_Or "or"
    } representation keyed;
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_HasField struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_HasValue struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_HasKind struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_IsLink struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_GreaterThan struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_LessThan struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_And struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Debug)]
    pub type Condition_Or struct {};
}

pub enum Selection2<'a, S, T> {
    Complete(T),
    Partial { selector: &'a S, representation: T },
}

/// A thin, typed wrapper around a `&Selector` and the type the `Selector` is
/// being deserialized with via `DeserializeSeed`.
pub struct SelectorSeed<'a, T, S> {
    selector: &'a S,
    _type: PhantomData<T>,
}

impl<'a, T, S> SelectorSeed<'a, T, S>
where
    T: Select<S>,
    S: ISelector,
{
    fn into(self) -> &'a S {
        self.selector
    }
}

impl<'a, T, S> From<&'a S> for SelectorSeed<'a, T, S>
where
    T: Select<S>,
    S: ISelector,
{
    fn from(selector: &'a S) -> Self {
        Self {
            selector,
            _type: PhantomData,
        }
    }
}

// impl<'a, T, S> SelectorSeed<'a, T, S> {
//     pub const fn into(self) -> &'a S {
//         self.selector
//     }

//     pub const fn from(selector: &'a S) -> Self {
//         Self {
//             selector,
//             _type: PhantomData,
//         }
//     }
// }

/// Blanket implementation that directly delegates to `Select::decode`.
impl<'de, T, S> DeserializeSeed<'de> for SelectorSeed<'de, T, S>
where
    T: Select<S> + Deserialize<'de>,
    S: ISelector,
{
    // TODO: make this a Complete(T)/Partial(T, &'a S) type
    type Value = T;
    // TODO: support conditionals
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <T as Select<S>>::decode(self.into(), deserializer)
    }
}

/// Sealed marker trait for types that can be used as `Selector`s.
#[doc(hidden)]
pub trait ISelector: Representation + private::Sealed {}

impl ISelector for Selector {}
impl ISelector for Matcher {}
impl ISelector for ExploreAll {}
impl ISelector for ExploreFields {}
impl ISelector for ExploreIndex {}
impl ISelector for ExploreRange {}
impl ISelector for ExploreRecursive {}
impl ISelector for ExploreUnion {}
impl ISelector for ExploreConditional {}
impl ISelector for ExploreRecursiveEdge {}

impl Selector {
    // TODO: handle ExploreFields and ExploreUnion iteratively, directly delegate the rest
    // fn new<Ctx, T, S>(args: T::SelectorArgs) -> Self
    // where
    //     Ctx: Context, // TODO FromContext?
    //     // S: ISelector,
    //     T: Select<Ctx, S>,
    //     Self: From<S>,
    // {
    //     Self::from(T::new_selector(args))
    // }

    // fn select<Ctx, T, S>(&self, T) -> Result<S::Output, ()>
    // where
    //     Ctx: Context,
    //     S: ISelector,
    //     T: Select<Ctx, S>,
    // {
    //     match self {
    //         // Self::Matcher
    //         // Self::ExploreAll
    //         // Self::ExploreFields
    //         // Self::ExploreIndex
    //         // Self::ExploreRange
    //         // Self::ExploreRecursive
    //         // Self::ExploreUnion
    //         // Self::ExploreConditional
    //         // Self::ExploreRecursiveEdge
    //     }
    // }
}

// pub mod args {
//     use ipld_macros_internals::dev::derive_more::{Deref, From};
//     use std::collections::HashMap;

//     #[derive(Debug, From)]
//     pub enum SelectorArgs {
//         Selector(super::Selector),
//         Matcher(Matcher),
//         ExploreAll(ExploreAll),
//         ExploreFields(ExploreFields),
//         ExploreIndex(ExploreIndex),
//         ExploreRange(ExploreRange),
//         ExploreRecursive(ExploreRecursive),
//         ExploreUnion(ExploreUnion),
//         ExploreConditional(ExploreConditional),
//         ExploreRecursiveEdge(ExploreRecursiveEdge),
//     }

//     #[derive(Debug, Deref, From)]
//     pub struct Matcher(pub Option<String>);
//     #[derive(Debug, Deref, From)]
//     pub struct ExploreAll(pub Box<SelectorArgs>);
//     #[derive(Debug, Deref, From)]
//     pub struct ExploreFields(pub HashMap<String, SelectorArgs>);
//     #[derive(Debug)]
//     pub struct ExploreIndex {
//         pub index: usize,
//         pub selector: Box<SelectorArgs>,
//     }
//     #[derive(Debug)]
//     pub struct ExploreRange {
//         pub start: usize,
//         pub end: usize,
//         pub selector: Box<SelectorArgs>,
//     }
//     #[derive(Debug)]
//     pub struct ExploreRecursive {
//         pub sequence: Box<SelectorArgs>,
//         pub limit: super::RecursionLimit,
//         pub stop_at: super::Condition,
//     }
//     #[derive(Debug, Deref, From)]
//     pub struct ExploreUnion(Vec<SelectorArgs>);
//     #[derive(Debug)]
//     pub struct ExploreConditional {
//         pub conditional: super::Condition,
//         pub selector: Box<SelectorArgs>,
//     }
//     #[derive(Debug)]
//     pub struct ExploreRecursiveEdge;
// }

mod private {
    use crate::dev::*;

    /// Sealed marker trait for `Selector` types.
    #[doc(hidden)]
    pub trait Sealed {}

    impl Sealed for Selector {}
    impl Sealed for Matcher {}
    impl Sealed for ExploreAll {}
    impl Sealed for ExploreFields {}
    impl Sealed for ExploreIndex {}
    impl Sealed for ExploreRange {}
    impl Sealed for ExploreRecursive {}
    impl Sealed for ExploreUnion {}
    impl Sealed for ExploreConditional {}
    impl Sealed for ExploreRecursiveEdge {}
}

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

        let sel1 = selector! {
            #[ipld_attr(internal)]
            Test,
            match(
                label=("label")
            )
        };

        // let sel1 = Selector::Matcher({ Matcher { label: None } });
        // let Selector::Matcher(matcher) = sel1;

        // let selection = <Test as Select<_, Matcher>>::select(t, &matcher);

        assert_eq!(true, true);
    }
}
