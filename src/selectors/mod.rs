//! IPLD Selectors
//!
//! TODO:
//!     - selectors are types that impl Representation (can be defined with `schema!`)
//!     - macro can compile selector string to a type
//!     - type implements Context

use crate::dev::*;
use ipld_macros::{ipld_macros_internal, schema};
use std::collections::BTreeMap;

schema! {
    ///
    /// TODO: serde impls for Selectors (for intelligently ignoring deserialized data)
    ///     - SelectorSeed, Into<SelectorSeed> for Selector
    ///     - impl DeserializeSeed<Value = T> for SelectorSeed for each type,
    ///     - IgnoredDag<T>, impl Visitor for IgnoredDag<T>
    ///         - which "validates" the types it receives against its schema before dropping the values
    ///     - impl Visitor for SelectorSeed
    ///         - mimics the type's default Visitor
    ///         - then, in any map/list type, call next_element_seed with SelectorSeed<InnerType>::from(selector)
    /// TODO: selector output ()
    #[ipld_macros_internal]
    pub type Selector null;
}

schema! {
    #[ipld_macros_internal]
    pub type ExploreAll null;
}

schema! {
    #[ipld_macros_internal]
    pub type Matcher null;
}

// schema! {
//     #[ipld_macros_internal]
//     type NewString string;
// }

// schema! {
//     #[ipld_macros_internal]
//     type NewEnum enum {
//         | Nope ("hell_no")
//         | Yep
//     };
// }

pub struct Selection<T: Representation>(Option<String>, T);

///
pub trait Select<Ctx, S>: Representation
where
    Ctx: Context,
    S: ISelector,
{
    ///
    type SelectorArgs;

    ///
    type Output;

    ///
    fn new_selector(args: Self::SelectorArgs) -> S;

    ///
    /// TODO? executor<'a, Ctx>, ...
    fn select(self, selector: &S) -> Result<Self::Output, ()>;

    //
    // fn patch<F>(&mut self, selector: &S, func: F) -> ()
    // where
    //     F: Fn(&mut Self::Output) -> ();

    //
    // fn flush(&mut self, selector: &S) -> Result<(), ()>;
}

impl<'a, Ctx, T> Select<Ctx, Selector> for T
where
    Ctx: Context,
    T: Representation,
{
    type SelectorArgs = ();
    type Output = ();

    // TODO:
    fn new_selector(args: Self::SelectorArgs) -> Selector {
        Selector
    }

    fn select(self, selector: &Selector) -> Result<Self::Output, ()> {
        unimplemented!()
    }
}

impl<'a, Ctx, T> Select<Ctx, Matcher> for T
where
    Ctx: Context,
    T: Representation,
{
    type SelectorArgs = Option<String>;
    type Output = Selection<Self>;

    // TODO:
    fn new_selector(args: Self::SelectorArgs) -> Matcher {
        Matcher
    }

    fn select(self, selector: &Matcher) -> Result<Self::Output, ()> {
        Ok(Selection(None, self))
    }
}

// impl<'a, Ctx, T> Select<Ctx, Matcher> for &'a T
// where
//     Ctx: Context,
//     for<'a>
//     T: Representation,
//     // &'a T: Representation,
// {
//     type SelectorArgs = Option<String>;
//     type Output = Selection<Self>;

//     fn new_selector(args: Self::SelectorArgs) -> Matcher {
//         // TODO?
//         Matcher
//     }

//     fn select(&self, selector: &Matcher) -> Result<Self::Output, ()> {
//         Ok(Selection(None, self))
//     }
// }

impl<S, const F: &'static str> ISelector for ExploreField<S, F> {}
pub struct ExploreField<S, const F: &'static str> {
    selector: std::marker::PhantomData<S>,
}

/// TODO? example impl for ExploreAll for a map:
/// impl<K, V, S> Select<ExploreAll> for Map<K, V>
/// where
///     V: Select<S>
/// {
///     type SelectorArgs = <V as Select<S>>::Args;
///     fn new_selector(args: Self::SelectorArgs) -> Selector {
///         let sel = <V as Select<S>>::new_selector(args);
///         Selector::from(ExploreAll::from(sel))
///     }
/// }
/// TODO? example impl for ExploreFields for a map:
/// impl<K, V, S> Select<ExploreFields> for Map<K, V>
/// where
///     V: Select<S>
/// {
///     type SelectorArgs = &[(&str, <V as Select<S>::Args)];
///     fn new_selector(args: Self::SelectorArgs) -> Selector {
///         let mut map = BTreeMap::new();
///
///         for (field, args) in args {
///             map.insert(field, <V as Select<S>>::new_selector(args));
///         }
///
///         Selector::from(ExploreFields::from(map))
///     }
/// }
///
/// TODO? example impl for ExploreFields for a struct:
/// impl<S> Select<ExploreField<"field1", S>> for Struct {
///     type SelectorArgs = (
///         ExploreFields,
///         <SomeInnerTypeOfStruct as Select<S>::Args,
///     );
///     fn new_selector(args: Self::SelectorArgs) -> Selector {
///         let sel = <SomeInnerTypeOfStruct as Select<S>>::new_selector(args.1);
///         args.0.insert("field1", sel);
///         Selector::from(args.0)
///     }
/// }

// impl From<SelectorEnvelope> for Selector;
// impl From<Selector> for SelectorEnvelope;

// pub trait IExploreRange {}

// pub trait IExploreField<const FIELD: &'static str, InnerType> {
//     // const FIELDS: HashSet<(&'static str, Type)>;
//     // const FIELDS: &'static [(&'static str, Type)];
//     // const FIELD: &'static str;
//     // const TYPE: Type;
//     // const fn is_field(s: &str) -> bool {
//     //     s == Self::FIELD
//     // }
//     fn insert<S>(explore_fields: &mut ExploreFields, selector: S) {
//         // TODO? in the repr macro:
//         // match selector variants the inner type supports
//         // then call <InnerType as ISelectorTrait>::
//         // explore_fields.insert(FIELD, <Inner as Select<S>>::from(selector))

//         unimplemented!()
//     }
// }

// /// TODO: impl this for all Representations that support ExploreFields
// ///
// pub trait ExploreFields {
//     fn add_field(
//         mut selectors: BTreeMap<String, Selector>,
//         field_name: String,
//         selector: Selector,
//     ) -> BTreeMap<String, Selector> {
//         selectors.insert(field_name, selector);
//         selectors
//     }
// }

// /// TODO: impl this for all Representations that support ExploreUnion
// ///
// pub trait UnionExplorer {
//     fn add_union(mut selectors: Vec<Selector>, selector: Selector) -> Vec<Selector> {
//         selectors.push(selector);
//         selectors
//     }
// }

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

// /// Wrapper type for visiting a `Deserializer` with a `Selector`.
// pub struct SelectorVisitor<'a, V> {
//     selector: &'a Selector,
//     visitor: V,
// }

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

// schema! {
//     #[ipld_macros_internal]
//     type Selector union {
//         | Matcher "."
//         | ExploreAll "a"
//         | ExploreFields "f"
//         | ExploreIndex "i"
//         | ExploreRange "r"
//         | ExploreRecursive "R"
//         | ExploreUnion "|"
//         | ExploreConditional "&"
//         | ExploreRecursiveEdge "@"
//     } representation keyed;
// }

// schema! {
//     #[ipld_macros_internal]
//     type ExploreAll struct {
//         next: Selector (rename ">"),
//     };
// }

// schema! {
//     #[ipld_macros_internal]
//     type ExploreFields struct {
//         fields: {String:Selector} (rename "f>"),
//     };
// }

// schema! {
//     #[ipld_macros_intenral]
//     type ExploreIndex struct {
//         index: Int (rename "i"),
//         next: Selector (rename ">"),
//     };
// }

// schema! {
//     #[ipld_macros_intenral]
//     type ExploreRange struct {
//         start: Int (rename "^"),
//         end: Int (rename "$"),
//         next: Selector (rename ">"),
//     };
// }

// schema! {
//     #[ipld_macros_intenral]
//     type ExploreRecursive struct {
//         sequence: Selector (rename ":>"),
//         limit: RecursionLimit (rename "l"),
//         stopAt: optional Condition (rename "!"),
//     };
// }

// schema!(
//     #[ipld_macros_internal]
//     type RecursionLimit union {
//         | RecursionLimit_None "none"
//         | RecursionLimit_Depth "depth"
//     } representation keyed;
// }
// schema! {
//     #[ipld_macros_internal]
//     type RecursionLimit_None struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type RecursionLimit_Depth int;
// }

// schema! {
//     #[ipld_macros_internal]
//     type ExploreRecursiveEdge struct {};
// }

// schema! {
//     #[ipld_macros_internal]
//     type ExploreUnion [Selector];
// }

// schema! {
//     #[ipld_macros_internal]
//     type ExploreConditional struct {
//         condition: Condition (rename "&"),
//         next: Selector (rename ">"),
//     };
// }

// schema! {
//     #[ipld_macros_internal]
//     type Matcher struct {
//         onlyIf: optional Condition,
//         label: optional String,
//     };
// }

// schema! {
//     #[ipld_macros_internal]
//     type Condition union {
//         | Condition_HasField "hasField"
//         | Condition_HasValue "="
//         | Condition_HasKind "%"
//         | Condition_IsLink "/"
//         | Condition_GreaterThan "greaterThan"
//         | Condition_LessThan "lessThan"
//         | Condition_And "and"
//         | Condition_Or "or"
//     } representation keyed;
// }

// schema! {
//     #[ipld_macros_internal]
//     type Condition_HasField struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_HasValue struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_HasKind struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_IsLink struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_GreaterThan struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_LessThan struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_And struct {};
// }
// schema! {
//     #[ipld_macros_internal]
//     type Condition_Or struct {};
// }

/// Sealed marker trait for `Selector` types.
trait ISelector {}
impl ISelector for Selector {}
impl ISelector for Matcher {}
impl ISelector for ExploreAll {}
// impl ISelector for ExploreFields {}
// impl ISelector for ExploreIndex {}
// impl ISelector for ExploreRange {}
// impl ISelector for ExploreRecursive {}
// impl ISelector for ExploreUnion {}
// impl ISelector for ExploreConditional {}
// impl ISelector for ExploreRecursiveEdge {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
