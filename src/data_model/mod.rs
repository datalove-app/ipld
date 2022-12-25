//! A general `Value` type, representing all IPLD data model kinds.

use crate::dev::*;
use macros::derive_more::From;
use maybestd::{borrow::Borrow, marker::PhantomData};

mod any;
// pub mod borrowed;
// mod canon;
mod link;
mod list;
mod map;
// mod node;
#[macro_use]
mod primitive;
// mod recursive;

pub use any::Any;
pub use link::Link;
pub use list::List;
pub use map::Map;
// pub use node::Node;
pub use primitive::*;

/// Wrapper type to connect [`serde::Serialize`] to the underlying type's
/// [`Representation::serialize`] codec-specific implementation.
#[doc(hidden)]
#[derive(Debug, From)]
pub struct SerializeRepr<'a, const C: u64, T>(pub &'a T);
// impl<'a, const C: u64, T> SerializeWrapper<C, T> {
//     pub const fn new(dag: &'a T) -> Self {
//         Self(dag)
//     }
// }
impl<'a, const C: u64, T> Serialize for SerializeRepr<'a, C, T>
where
    T: Representation,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Representation::serialize::<C, S>(self.0, serializer)
    }
}

// pub struct SWrapper<'a, const C: u64, T: Borrow<Q>, Q>(pub &'a Q, PhantomData<T>);
// impl<'a, const C: u64, T: Representation + Borrow<Q>, Q> Borrow<SWrapper<'a, C, T, Q>> for T {
//     fn borrow(&self) -> &SWrapper<'a, C, T, Q> {
//         &SWrapper::<'a, C, T, Q>(self.borrow(), PhantomData)
//     }
// }

/// Wrapper type to connect [`serde::Deserialize`] to the underlying type's
/// [`Representation::deserialize`] codec-specific implementation.
#[doc(hidden)]
#[derive(Debug)]
pub struct DeserializeRepr<const C: u64, T>(PhantomData<T>);
impl<const C: u64, T> DeserializeRepr<C, T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
impl<'de, const C: u64, T> DeserializeSeed<'de> for DeserializeRepr<C, T>
where
    T: Representation,
{
    type Value = T;
    #[inline(always)]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <T as Representation>::deserialize::<C, D>(deserializer)
    }
}

// /// Wrapper type to connect [`serde::DeserializeSeed`]s to the underlying type's
// /// [`Select::__select_de`] codec-specific implementation.
// #[doc(hidden)]
// #[derive(Debug)]
// pub struct DeserializeSelect<const C: u64, Ctx, S, T>(S, PhantomData<(Ctx, T)>);
// impl<const C: u64, Ctx, S, T> DeserializeSelect<C, Ctx, S, T> {
//     pub const fn from(seed: S) -> Self {
//         Self(seed, PhantomData)
//     }
// }
// impl<'a, 'de, const C: u64, Ctx, T> DeserializeSeed<'de>
//     for DeserializeSelect<C, Ctx, SelectorSeed<'a, Ctx, T>, T>
// where
//     Ctx: Context,
//     // S: SeedType<T>, TODO:
//     T: Select<Ctx>,
// {
//     type Value = ();
//     #[inline(always)]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         T::__select_de::<C, D>(self.0, deserializer)
//     }
// }
