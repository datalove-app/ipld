//! A general `Value` type, representing all IPLD data model kinds.

use crate::dev::*;
use macros::derive_more::From;

mod any;
// pub mod borrowed;
// mod canon;
mod link;
mod list;
mod map;
mod primitive;
// mod recursive;

pub use any::Any;
pub use link::Link;
pub use list::List;
pub use map::Map;
pub use primitive::*;

/// Wrapper type to connect [`serde::Serialize`] to the underlying type's
/// [`Representation::serialize`] codec-specific implementation.
#[doc(hidden)]
#[derive(Debug, From)]
pub struct SerializeWrapper<'a, const C: u64, T>(pub &'a T);
impl<'a, const C: u64, T> Serialize for SerializeWrapper<'a, C, T>
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

/// Wrapper type to connect [`serde::Deserialize`] to the underlying type's
/// [`Representation::deserialize`] codec-specific implementation.
#[doc(hidden)]
#[derive(Debug)]
pub struct DeserializeWrapper<const C: u64, T>(std::marker::PhantomData<T>);
impl<const C: u64, T> DeserializeWrapper<C, T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<'de, const C: u64, T> DeserializeSeed<'de> for DeserializeWrapper<C, T>
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
