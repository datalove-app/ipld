//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.

mod context;
mod executor;
mod impls;

pub use context::*;
pub use executor::*;

use crate::dev::*;
// use crate::selectors::args as Args;
use futures::{
    future::FutureExt,
    task::{Context as Cx, Poll},
};
use pin_utils::unsafe_pinned;
use std::{convert::TryFrom, fmt, pin::Pin};

/// TODO: represents Schema or Representation kind?
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    ///
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Bytes,
    List,
    Map,
    Link,
    Struct,
    Enum,
    Union,
    Copy,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Field<A> {
    /// Name of the `Representation` type contained within this field.
    pub type_name: &'static str,

    /// `Representation::Kind` of the field type.
    pub value: Kind,

    /// The serialized field name of this type.
    pub alias: A,
}

impl<A> Field<A> {
    // pub const fn new<T: Representation>(alias: A) -> Self {
    //     Field {
    //         type_name: T::NAME,
    //         value: T::KIND,
    //         alias,
    //     }
    // }
}

///
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Fields {
    None,
    List(Field<()>),
    Map {
        key: Field<()>,
        value: Field<()>,
    },
    Struct(&'static [(&'static str, Field<&'static str>)]),
    Enum {
        kind: Kind,
        fields: &'static [Field<()>],
    },
    // Union representations
    Envelope {
        discriminant_key: &'static str,
        fields: &'static [Field<&'static str>],
    },
    Inline {
        discriminant_key: &'static str,
        fields: &'static [Field<&'static str>],
    },
    Keyed(&'static [Field<&'static str>]),
    Kinded(&'static [Field<&'static str>]),
    Byteprefix(&'static [Field<&'static [u8; 1]>]),
}

///
///
/// Some types have in-memory representations distinct from their IPLD representations:
///     - Links can map to types, so they can represent both CIDs and the underling types
///     - Signed/encrypted payloads can be further resolved into native types after verifying the signature/performing decryption
///
///
/// TODO? == what are the requirements?
///     - serialize a type to a block
///         - serialize an ipld as this type to a block
///     - deserialize a type from a block
///         -
///         - deserialize an ipld as this type from a block
///     - focus a type to a value (typed or ipld?) based on a selector
///         - ? call a closure on the selected type?
///     - transform a value within a type based on a selector (? and a closure ?)
///         - return a typed value or ipld?
///
/// TODO? selection can only happen *accurately* against fully-resolved types and blocks
///
/// TODO: what to impl?
///     - focus<T>(&self, selector, context) -> Result<T>
///     - patch<T, F>(&mut self, selector, f: F, context) -> Result<()>
///         where F: Fn(&mut T, context);
///         - based on success of recursing, flags any link type as dirty
///     - flush(&self, context) -> Result<Selector>
///     TODO? << other impls >>
///     - validate_selector(selector)
///         - TODO: ? returns a stateful Visitor + DeserializeSeed?
///     - derive Serialize
///     - in focus<T>(...), impl Deserialize
///         - TODO: ? stateful visitor derived from selector + type?
///         - TODO: ? impl DeserializeSeed for selector?
///         - TODO: ? Representation::visitor(selector: &Selector)
pub trait Representation: Serialize + for<'de> Deserialize<'de> {
    /// The stringified name of the IPLD type.
    const NAME: &'static str;
    // /// The stringified IPLD typedef.
    // const SCHEMA: &'static str;
    // /// The IPLD Schema kind of the type.
    // const KIND: Kind;
    // /// The type's `Select`able field names and their IPLD Schema kinds, if a recursive type.
    // const FIELDS: Fields = Fields::None;
    // ///
    // const HAS_LINKS: bool = false;

    ///
    /// for unions, this delegates to the variant's type name
    fn name(&self) -> &'static str {
        Self::NAME
    }

    // ///
    // fn kind(&self) -> Kind {
    //     Self::KIND
    // }

    // ///
    // fn to_owned(&self) -> Self;
}

pub trait RepresentationExt<T: Representation>: Representation {
    const NAME: &'static str;
}

// TODO: impl the Entry pattern, so you can use matched results to update the tree
pub struct Selection {
    label: Option<String>,
    matched: Option<Box<dyn ObjectSafeRepresentation>>,
}

///
pub type SelectionResult = Result<Selection, Error>;

///
#[must_use = "SelectionStreams do nothing unless polled"]
pub struct SelectionStream {
    // TODO: pin vs box?
    inner: Pin<Box<dyn Stream<Item = SelectionResult>>>,
}

// impl Unpin for SelectionStream {}

impl SelectionStream {
    // TODO:
    unsafe_pinned!(inner: dyn Stream<Item = SelectionResult>);

    ///
    pub fn of(matched: SelectionResult) -> Self {
        // let matched = matched.into();
        SelectionStream::from(async { matched }.into_stream())
    }

    ///
    pub fn from<S>(inner: S) -> Self
    where
        S: Stream<Item = SelectionResult> + 'static,
    {
        SelectionStream {
            inner: Box::pin(inner),
        }
    }
}

impl Stream for SelectionStream {
    type Item = SelectionResult;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Cx<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

///
/// TODO: possibly look at erased-serde to complete this "hack"
#[doc(hidden)]
pub trait ObjectSafeRepresentation {}
impl<T: Representation> ObjectSafeRepresentation for T {}

///
///

///
pub trait Select<S, Ctx = DefaultContext>: Representation
where
    S: ISelector,
    Ctx: Context,
{
    /// TODO? executor<'a, Ctx>, ...
    fn select(
        self,
        selector: &S,
        // context: &Ctx,
        // executor: &Executor<'a, Ctx>,
    ) -> SelectionStream;

    /// `Deserialize`s a selection of the given type against a `Selector`.
    #[inline]
    fn decode<'de, D>(selector: &'de S, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>;

    ///
    /// TODO
    #[inline]
    #[doc(hidden)]
    fn validate(selector: &S) -> Result<(), Error> {
        Ok(())
    }
}

///
impl<Ctx, T> Select<Matcher, Ctx> for T
where
    Ctx: Context,
    T: Representation + 'static,
{
    #[inline]
    fn select(
        self,
        selector: &Matcher,
        // context: &Ctx,
        // executor: &Executor<'a, Ctx>,
    ) -> SelectionStream {
        // TODO: handle condition, probably using same similar macro to impl_select!
        SelectionStream::of(Ok(Selection {
            label: selector.label.clone(),
            matched: Some(Box::new(self)),
        }))
    }

    // TODO: support conditionals
    #[inline]
    fn decode<'de, D>(_selector: &'de Matcher, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        T::deserialize(decoder)
    }
}

/// Helper macro that `impl Select<Selector, Ctx> for T`.
///
/// Takes as parameters the type name, optional type bounds on `Ctx`, and the
/// `Selector`s for which the type already implements `Select`.
#[macro_export]
macro_rules! impl_root_select {
    // TODO: support additional bounds
    // shorthand syntax
    ($name:ident => $($ISelector:ident),*) => {
        $crate::impl_root_select!($name, Ctx: => $($ISelector),*);
    };
    // main
    ($name:ident, Ctx : $($ctx:ident),* => $($ISelector:ident),*) => {
        $crate::impl_root_select!(@select $name, Ctx: $($ctx),* => $($ISelector),*);
        // $crate::impl_root_select!(@de_seed $name => $($ISelector)*);
    };
    // select def
    (@select $name:ident, Ctx : $($ctx:ident),* => $($ISelector:ident),*) => {
        impl<Ctx> $crate::Select<$crate::Selector, Ctx> for $name
        where
            Ctx: $crate::Context $(+ $ctx)*,
        {
            $crate::impl_root_select!(@select $name => $($ISelector),*);
            $crate::impl_root_select!(@decode $name => $($ISelector),*);
            $crate::impl_root_select!(@validate $name => $($ISelector),*);
        }
    };
    (@select $name:ident => $($ISelector:ident),*) => {
        // fn select<Ctx: FromContext<NewCtx>>(
        #[inline]
        fn select(
            self,
            selector: &$crate::Selector,
            // context: &Ctx,
            // executor: &Executor<'a, Ctx>,
        ) -> $crate::SelectionStream {
            use $crate::{selectors::*, Error, Select, SelectionStream};
            match selector {
                $(Selector::$ISelector(sel) => {
                    <$name as Select<$ISelector, Ctx>>::select(self, sel)
                },)*
                sel => SelectionStream::of(Err(Error::unsupported_selector::<$name, Selector>(sel))),
            }
        }
    };
    (@decode $name:ident => $($ISelector:ident),*) => {
        #[inline]
        fn decode<'de, D>(selector: &'de $crate::Selector, decoder: D) -> Result<Self, D::Error>
        where
            D: $crate::Decoder<'de>,
        {
            use $crate::{dev::serde::de, selectors::*, Error};
            match selector {
                $(Selector::$ISelector(sel) => {
                    <$name as Select<$ISelector, Ctx>>::decode(sel, decoder)
                },)*
                sel => Err(de::Error::custom(
                    Error::unsupported_selector::<$name, Selector>(sel)
                )),
            }
        }
    };
    (@validate $name:ident => $($ISelector:ident),*) => {
        #[inline]
        fn validate(selector: &$crate::Selector) -> Result<(), $crate::Error> {
            use $crate::{selectors::*, Error, Select};
            match selector {
                $(Selector::$ISelector(sel) => {
                    <$name as Select<$ISelector, Ctx>>::validate(sel)
                },)*
                sel => Err(Error::unsupported_selector::<$name, Selector>(sel)),
            }
        }
    };
}

// impl<Ctx, T> Select<Ctx, Selector> for T
// where
//     Ctx: Context,
//     T: ObjectSafeRepresentation + 'static,
// {
//     fn select(
//         self,
//         selector: &Selector,
//         // executor: &Executor<'a, Ctx>,
//     ) -> SelectionStream {
//         macro_rules! match_selector {
//             ($variant:ident) => {
//                 if $crate::dev::macros::impls::impls!(T: Select<NewCtx, $variant>) {
//                     if let Selector::$variant(sel) = selector {
//                         return <T as Select<Ctx, $variant>>::select(self, sel);
//                     }
//                 }
//             };
//         }

//         match_selector!(Matcher);
//         match_selector!(ExploreAll);
//         match_selector!(ExploreFields);
//         match_selector!(ExploreIndex);
//         match_selector!(ExploreRange);
//         match_selector!(ExploreRecursive);
//         match_selector!(ExploreUnion);
//         match_selector!(ExploreConditional);
//         match_selector!(ExploreRecursiveEdge);

//         SelectionStream::of::<T>(Err(()))
//     }
// }

// /// TODO: this blanket impl might be better implemented by each type individually

//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//

// pub trait Dag<Ctx> {
//     type Repr: Representation<Ctx>;
// }

// impl<Ctx, T> Serialize for Dag<Ctx, Repr = T> where T: Representation<Ctx> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         T::serialize()
//     }
// }

// impl<'de, Ctx, T> Deserialize<'de> for Dag<Ctx, Repr = T> where T: Representation<Ctx> {}

// pub trait RepresentationExt<'a, Ctx>: Representation<Ctx> {
//     fn serialize_ipld<S>(ipld: &BorrowedIpld<'a>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer;

//     fn deserialize_ipld<D>(deserializer: D) -> Result<BorrowedIpld<'a>, D::Error>
//     where
//         D: Deserializer<'a>,
// }

// /// An interface for `Encode`ing and `Decode`ing an IPLD Representation.
// ///
// /// Types that have `Representation`s generally follow the same few steps when
// /// encoding (in reverse for decoding):
// ///     - pre-processing, i.e.:
// ///         fetching codecs
// ///         generating signatures
// ///         converting bytes to hex
// ///     - (? optionally) conversion of the type to an Ipld-like
// ///         helpful for ensuring canonicalization
// ///     - serializing the Ipld-like type with a provided Codec
// /// decoding:
// ///     - pre-processing, i.e.:
// ///         fetching blocks
// ///     - deserializing either:
// ///         - to an Ipld-like type, then conversion to native type
// ///         - to a native type directly
// ///
// /// The supplied execution `Context` provides `Codec` to use, and can also:
// ///     - dictate which fields to `Read`/`Write`,
// ///     - provide a source/sink of bytes for a particular `Cid`/`Block`
// #[async_trait]
// pub trait Representation: Sized {
//     type Context = Context;

//     /// Encodes a type to a provided `Context`.
//     ///
//     /// By default, creates an IPLD data type representation from the type, then
//     /// encodes the `Ipld` with the provided `Codec`.
//     async fn encode(
//         &self,
//         ctx: &Self::Context,
//     ) -> Result<Option<Cid>, <Self::Context as Context>::Error> {
//         //        let dag = self.to_ipld(ctx).await?;
//         //        ctx.codec().encode(dag)?
//     }

//     /// `Read` a type from a provided `Context`.
//     async fn decode(
//         bytes: &[u8],
//         ctx: &Self::Context,
//     ) -> Result<Self, <Self::Context as Context>::Error> {
//         //        let dag = ctx.codec().decode(bytes).await?;
//         //        Self::from_ipld(dag, ctx)
//     }

//     //    /// `Read` a type from a provided `Context`.
//     //    async fn read_with_ctx<NewCtx>(ctx: &Ctx) -> Result<Self, Error>
//     //    where
//     //        NewCtx: FromContext<Ctx>,
//     //        Self: Representation<NewCtx>;

//     //    /// `Write` a type to a provided `Context`.
//     //    async fn write_with_ctx<NewCtx>(&self, ctx: &Ctx) -> Result<(), Error>
//     //    where
//     //        Co: 'async_trait,
//     //        R: 'async_trait,
//     //        W: 'async_trait,
//     //        NewCtx: FromContext<Ctx>;
// }
