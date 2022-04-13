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
mod state;

// pub use path::*;
pub use context::*;
pub use schema::*;
pub use state::*;

use crate::dev::*;
use serde::de::DeserializeSeed;
use std::path::Path;

///
/// TODO:
pub trait Select<C: Context = MemoryContext>: Representation + 'static {
    // type Visitor: Visitor<'de, Value = Self>;

    /// Attempts to match against this type directly.
    fn r#match(
        selector: &Selector,
        state: &mut SelectorState,
        ctx: &mut C,
    ) -> Result<Option<Self>, Error>;

    /// Produces a stream of [`Selection`]s.
    fn select<S: Select<C>>(
        selector: &Selector,
        state: &mut SelectorState,
        ctx: &mut C,
    ) -> Result<Option<S>, Error>;
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

// impl<C: Context, T: Representation, U: Representation> Select<C, U> for T
// where
//     // ContextSeed<C, T>: for<'de> DeserializeSeed<'de, Value = Option<T>>,
//     U: Select<C, U>,
//     ContextSeed<C, U>: for<'de> DeserializeSeed<'de, Value = Option<U>>,
// {
//     fn select(seed: SelectionProgress, ctx: &mut C) -> Result<Option<T>, Error> {
//         // let deserializer = ctx.path_decoder(seed.path())?;
//         // ContextSeed::<C, Self>::from(seed, ctx)
//         //     .deserialize(deserializer)
//         //     .map_err(|err| Error::Decoder(anyhow::anyhow!(err.to_string())))
//         <U as Select<C, T>>::select(seed, ctx)
//
//         unimplemented!()
//     }
//
//     fn patch(&mut self, seed: SelectionProgress, dag: T, ctx: &mut C) -> Result<(), Error> {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// pub enum SelectionResult<T> {
//     Continue(Selector, Option<T>),
//     End(Option<T>),
// }
//
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
//
//          for (field, args) in args {
//              map.insert(field, <V as Select<S>>::new_selector(args));
//          }
//
//          Selector::from(ExploreFields::from(map))
//      }
//  }
//
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

// pub enum Selection2<'a, S, T> {
//     Complete(T),
//     Partial { selector: &'a S, representation: T },
// }

// impl From<SelectorEnvelope> for Selector;
// impl From<Selector> for SelectorEnvelope;

macro_rules! impl_variant {
    ($variant:ident $selector_ty:ty | { $is:ident, $as:ident }) => {
        impl_variant!(@is $is -> $variant $selector_ty);
        impl_variant!(@as $as -> $variant $selector_ty);
    };

    (@is $fn:ident -> $variant:ident $selector_ty:ty) => {
        #[inline]
        pub const fn $fn(&self) -> bool {
            match self {
                Self::$variant(..) => true,
                _ => false,
            }
        }
    };
    (@as $fn:ident -> $variant:ident $selector_ty:ty) => {
        #[inline]
        pub fn $fn(&self) -> Option<&$selector_ty> {
            match self {
                Self::$variant(inner) => Some(inner),
                _ => None,
            }
        }
    };
    // (@try $fn:ident -> $variant:ident $selector_ty:ty) => {
    //     #[inline]
    //     pub fn $fn<T: Representation, U: Representation>(&self) -> Result<&$selector_ty, Error> {
    //         match self {
    //             Self::$variant(inner) => Ok(inner),
    //             _ => Err(Error::unsupported_selector::<T, U>(self)),
    //         }
    //     }
    // };
}

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
    //
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

    pub fn next(&self, field: Option<&Path>) -> Result<&Selector, Error> {
        match self {
            Self::Matcher(inner) => Ok(self),
            // Self::ExploreAll { next, .. } => Ok(next),
            // Self::ExploreFields { next, .. } => Ok(next),
            // Self::ExploreIndex { next, .. } => Ok(next),
            // Self::ExploreRange { next, .. } => Ok(next),
            _ => Err(Error::missing_next_selector(self)),
        }
    }

    pub fn assert_matcher<T: Representation, U: Representation>(&self) -> Result<&Matcher, Error> {
        match self {
            Self::Matcher(inner) if type_eq::<T, U>() => Ok(inner),
            _ => Err(Error::unsupported_selector::<T, U>(self)),
        }
    }

    // impl_variant!(@try try_matcher -> Matcher Matcher);

    impl_variant!(Matcher Matcher |
        {is_matcher, as_matcher});
    impl_variant!(ExploreAll ExploreAll |
        {is_explore_all, as_explore_all});
    impl_variant!(ExploreFields ExploreFields |
        {is_explore_fields, as_explore_fields});
    impl_variant!(ExploreIndex ExploreIndex |
        {is_explore_index, as_explore_index});
    impl_variant!(ExploreRange ExploreRange |
        {is_explore_range, as_explore_range});
    // impl_as!(as_explore_recursive -> ExploreRecursive ExploreRecursive);
    // impl_as!(as_explore_union -> ExploreUnion ExploreUnion);
    // impl_as!(as_explore_conditional -> ExploreConditional ExploreConditional);
    // impl_as!(as_explore_recursive_edge -> ExploreRecursiveEdge ExploreRecursiveEdge);
}

// impl ISelector for Matcher {}

// impl ISelector for ExploreAll {
//     #[inline]
//     fn inner_selector(&self) -> Option<&Selector> {
//         Some(&self.next)
//     }

//     fn inner_selector_at<I: Into<usize>>(&self, _index: I) -> Option<&Selector> {
//         self.inner_selector()
//     }
// }

// impl ISelector for ExploreFields {
//     #[inline]
//     fn inner_selector_for<F: AsRef<str>>(&self, field: F) -> Option<&Selector> {
//         unimplemented!()
//     }
// }

// impl ISelector for ExploreIndex {}
// impl ISelector for ExploreRange {}
// impl ISelector for ExploreRecursive {}
// impl ISelector for ExploreUnion {}
// impl ISelector for ExploreConditional {}
// impl ISelector for ExploreRecursiveEdge {}

// impl ExploreIndex {
//     ///
//     #[inline]
//     pub fn selector_at<I: Into<usize>>(&self, index: I) -> Option<&Selector> {
//         unimplemented!()
//     }
// }

// impl ExploreRange {
//     ///
//     #[inline]
//     pub fn selector(&self) -> Option<&Selector> {
//         unimplemented!()
//     }
// }

// impl ExploreUnion {
//     ///
//     #[inline]
//     pub fn selector<I: Into<usize>>(&self, index: I) -> Option<&Selector> {
//         unimplemented!()
//     }
// }

// impl ExploreRecursive {
//     ///
//     #[inline]
//     pub fn selector(&self) -> Option<&Selector> {
//         unimplemented!()
//     }
// }

// ///
// impl<Ctx, T> Select<Matcher, Ctx> for T
// where
//     Ctx: Context,
//     T: Representation + 'static,
// {
//     // TODO: handle conditionals, probably using same similar macro to impl_select!
//     #[inline]
//     fn select<'a>(
//         &'a self,
//         selector: &Matcher,
//         // context: &Ctx,
//         // executor: &Executor<'a, Ctx>,
//     ) -> SelectionStream<'a, Selection<'a>> {
//         SelectionStream::ok(Selection::new(self, selector.label.clone()))
//     }
//
//     // TODO: handle conditionals
//     #[inline]
//     fn decode<'de, D>(_selector: &'de Matcher, decoder: D) -> Result<Self, D::Error>
//     where
//         D: Decoder<'de>,
//         Self: Deserialize<'de>,
//     {
//         T::deserialize(decoder)
//     }
// }
//
// TODO: how to decode? cache decoded value?
//
// impl<Ctx, T> Select<ExploreUnion, Ctx> for T
// where
//     Ctx: Context,
//     T: Representation + 'static,
// {
//     #[inline]
//     fn select(
//         self,
//         selector: &Matcher,
//         // context: &Ctx,
//         // executor: &Executor<'a, Ctx>,
//     ) -> SelectionStream {
//         // TODO: handle condition, probably using same similar macro to impl_select!
//         SelectionStream::ok(Ok(Selection {
//             label: selector.label.clone(),
//             matched: Some(Box::new(self)),
//         }))
//     }
//
//     // TODO: support conditionals
//     #[inline]
//     fn decode<'de, D>(_selector: &'de Matcher, decoder: D) -> Result<Self, D::Error>
//     where
//         D: Decoder<'de>,
//     {
//         T::deserialize(decoder)
//     }
// }
//
// impl<Ctx, T> Select<Matcher, Ctx> for T
// where
//     T: Representation + 'static,
//     Ctx: Context,
// {
//     #[inline]
//     fn select<'a>(
//         &'a self,
//         selector: &Matcher,
//         // context: &Ctx,
//         // executor: &Executor<'a, Ctx>,
//     ) -> SelectionStream<'a, Selection<'a>> {
//         // use $crate::{selectors::*, Error, Select, SelectionStream};
//         // match selector {
//
//         //     $(Selector::$ISelector(sel) => {
//         //         <Self as Select<$ISelector, Ctx>>::select(self, sel)
//         //     },)*
//         //     sel => SelectionStream::err(Error::unsupported_selector::<Self, Selector>(sel)),
//         // }
//         unimplemented!()
//     }
//     /// Delegates directly to the `ISelector` contained within the given
//     /// `Selector`. See [`Select::decode`]() and [`serde::de::DeserializeSeed`]() for more information.
//     #[inline]
//     fn decode<'de, D>(selector: &'de Matcher, decoder: D) -> Result<Self, D::Error>
//     where
//         D: Decoder<'de>,
//         Self: crate::dev::Deserialize<'de>,
//     {
//         // use $crate::{dev::serde::de, selectors::*, Error};
//         // match selector {
//         //     $(Selector::$ISelector(sel) => {
//         //         <Self as Select<$ISelector, Ctx>>::decode(sel, decoder)
//         //     },)*
//         //     sel => Err(de::Error::custom(
//         //         Error::unsupported_selector::<Self, Selector>(sel)
//         //     )),
//         // }
//         unimplemented!()
//     }
// }

/// Helper macro that `impl Select<Selector, Ctx> for T where T: Representation`.
///
/// Takes as parameters the type name, optional type bounds on `Ctx`, and the
/// `Selector`s for which the type already implements `Select`.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_root_select {
    // TODO: support additional bounds
    // select def
    ($name:ty => $($ISelector:ident),*) => {
        impl<Ctx> $crate::Select<$crate::Selector, Ctx> for $name
        where
            Ctx: $crate::Context,
        {
            $crate::impl_root_select!(@methods $($ISelector),*);
        }
    };
    // generic def, where you write your own impl header
    ($($ISelector:ident),* { $($header:tt)* }) => {
        $($header)* {
            $crate::impl_root_select!(@methods $($ISelector),*);
        }
    };
    (@methods $($ISelector:ident),*) => {
        $crate::impl_root_select!(@select $($ISelector),*);
        $crate::impl_root_select!(@decode $($ISelector),*);
        $crate::impl_root_select!(@validate $($ISelector),*);
    };
    (@select $($ISelector:ident),*) => {
        /// Delegates directly to the `ISelector` contained within the given
        /// `Selector`. See [`Select::select`]() for more information.
        ///
        /// [`Select::select`]: crate::Select::select
        /// todo fn select<Ctx: FromContext<NewCtx>>(
        #[inline]
        fn select<'a>(
            &'a self,
            selector: &$crate::Selector,
            // context: &Ctx,
            // executor: &Executor<'a, Ctx>,
        ) -> $crate::SelectionStream<'a, Selection<'a>> {
            use $crate::{selectors::*, Error, Select, SelectionStream};
            match selector {
                $(Selector::$ISelector(sel) => {
                    <Self as Select<$ISelector, Ctx>>::select(self, sel)
                },)*
                sel => SelectionStream::err(Error::unsupported_selector::<Self, Selector>(sel)),
            }
        }
    };
    (@decode $($ISelector:ident),*) => {
        /// Delegates directly to the `ISelector` contained within the given
        /// `Selector`. See [`Select::decode`]() and [`serde::de::DeserializeSeed`]() for more information.
        #[inline]
        fn decode<'de, D>(selector: &'de $crate::Selector, decoder: D) -> Result<Self, D::Error>
        where
            D: $crate::Decoder<'de>,
            Self: $crate::dev::Deserialize<'de>
        {
            use $crate::{dev::serde::de, selectors::*, Error};
            match selector {
                $(Selector::$ISelector(sel) => {
                    <Self as Select<$ISelector, Ctx>>::decode(sel, decoder)
                },)*
                sel => Err(de::Error::custom(
                    Error::unsupported_selector::<Self, Selector>(sel)
                )),
            }
        }
    };
    (@validate $($ISelector:ident),*) => {
        /// Delegates directly to the `ISelector` contained within the given
        /// `Selector`. See [`Select::validate`]() for more information.
        #[inline]
        fn validate(selector: &$crate::Selector) -> Result<(), $crate::Error> {
            use $crate::{selectors::*, Error, Select};
            match selector {
                $(Selector::$ISelector(sel) => {
                    <Self as Select<$ISelector, Ctx>>::validate(sel)
                },)*
                sel => Err(Error::unsupported_selector::<Self, Selector>(sel)),
            }
        }
    };
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
