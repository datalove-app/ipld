//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.

mod context;
mod executor;
mod impls;

pub use context::*;
pub use executor::*;

use crate::dev::*;
use downcast_rs::{impl_downcast, Downcast};
// use crate::selectors::args as Args;
use futures::{
    future::FutureExt,
    task::{Context as Cx, Poll},
};
use pin_utils::unsafe_pinned;
use std::{any::Any, convert::TryFrom, fmt, pin::Pin};

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
pub trait Representation {
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

impl<'a, T> Representation for &'a T
where
    T: Representation,
{
    const NAME: &'static str = T::NAME;
}

impl<'a, T> Representation for &'a mut T
where
    T: Representation,
{
    const NAME: &'static str = T::NAME;
}

// #[async_trait]
pub trait RepresentationExt<T: Representation>: Representation {
    // fn resolve(self)
}

///
/// TODO: possibly look at erased-serde to complete this "hack"
#[doc(hidden)]
pub trait ObjectSafeRepresentation: Downcast {}
impl<T: Representation + 'static> ObjectSafeRepresentation for T {}
impl_downcast!(ObjectSafeRepresentation);

// TODO: impl the Entry pattern, so you can use matched results to update the tree
// TODO rename to Match
// TODO refactor to hold a ref
pub struct Selection<'a> {
    label: Option<String>,
    matched: &'a dyn ObjectSafeRepresentation,
}

impl<'a> Selection<'a> {
    ///
    #[inline]
    pub fn new<T>(matched: &'a T, label: Option<String>) -> Self
    where
        T: Representation + 'static,
    {
        Selection { label, matched }
    }

    ///
    #[inline]
    pub fn label(&self) -> &Option<String> {
        &self.label
    }

    #[inline]
    pub fn downcast<T>(&self) -> Option<&'a T>
    where
        T: Representation + 'static,
    {
        self.matched.downcast_ref::<T>()
    }
}

impl<'a, T> From<&'a T> for Selection<'a>
where
    T: Representation + 'static,
{
    #[inline]
    fn from(matched: &'a T) -> Self {
        Selection {
            label: None,
            matched,
        }
    }
}

pub struct SelectionMut<'a> {
    label: Option<String>,
    matched: &'a mut dyn ObjectSafeRepresentation,
}

///
#[must_use = "Streams do nothing unless polled"]
pub struct SelectionStream<'a, T> {
    // TODO: pin vs box?
    inner: Pin<Box<dyn Stream<Item = Result<T, Error>> + 'a>>,
}

// impl Unpin for SelectionStream {}

impl<'a, T: 'a> SelectionStream<'a, T> {
    // TODO: requires that the stream be wrapped in a pinbox - why?
    unsafe_pinned!(inner: dyn Stream<Item = Result<T, Error>>);

    ///
    pub fn ok(t: T) -> Self {
        SelectionStream::from(async { Ok(t) }.into_stream())
    }

    pub fn err(err: Error) -> Self {
        SelectionStream::from(async { Err(err) }.into_stream())
    }

    ///
    pub fn from<S>(inner: S) -> Self
    where
        S: Stream<Item = Result<T, Error>> + 'a,
    {
        SelectionStream {
            inner: Box::pin(inner),
        }
    }
}

impl<'a, T> Stream for SelectionStream<'a, T> {
    type Item = Result<T, Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Cx<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

///
///

///
pub trait Select<S = Selector, Ctx = DefaultContext>: Representation + 'static
where
    S: ISelector,
    Ctx: Context,
{
    /// Selects zero or more ...
    ///
    /// for link
    /// -
    /// for everything else
    /// -
    ///
    /// TODO? executor<'a, Ctx>, ...
    fn select<'a>(
        &'a self,
        selector: &S,
        // context: &Ctx,
        // executor: &Executor<'a, Ctx>,
    ) -> SelectionStream<'a, Selection<'a>>;

    // fn select_mut(
    //     &mut self,
    //     selector: &S,
    //     // context: &Ctx,
    //     // executor: &Executor<'a, Ctx>,
    // ) -> SelectionStream<&mut dyn ObjectSafeRepresentation> {
    //     unimplemented!()
    // }

    // ///
    // /// for link:
    // /// - fails if ...
    // /// - creates a stub encoder (wrapping a Write and an Encoder)
    // /// - then recurses
    // /// for everything else
    // /// - either match and serialize itself to encoder
    // /// - or decide where to go next
    // #[inline]
    // fn encode<'a, E>(&self, selector: &'a S, encoder: E) -> Result<E::Ok, E::Error>
    // where
    //     E: Encoder,
    // {
    //     unimplemented!()
    // }

    /// `Deserialize`s a selection of the type from a `Decoder` using a `Selector`.
    #[inline]
    fn decode<'de, D>(selector: &'de S, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
        Self: Deserialize<'de>;

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
    // TODO: handle conditionals, probably using same similar macro to impl_select!
    #[inline]
    fn select<'a>(
        &'a self,
        selector: &Matcher,
        // context: &Ctx,
        // executor: &Executor<'a, Ctx>,
    ) -> SelectionStream<'a, Selection<'a>> {
        SelectionStream::ok(Selection::new(self, selector.label.clone()))
    }

    // TODO: handle conditionals
    #[inline]
    fn decode<'de, D>(_selector: &'de Matcher, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
        Self: Deserialize<'de>,
    {
        T::deserialize(decoder)
    }
}

// TODO: how to decode? cache decoded value?
///
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

//     // TODO: support conditionals
//     #[inline]
//     fn decode<'de, D>(_selector: &'de Matcher, decoder: D) -> Result<Self, D::Error>
//     where
//         D: Decoder<'de>,
//     {
//         T::deserialize(decoder)
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
