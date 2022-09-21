//! A general `Value` type, representing all IPLD data model kinds.

use crate::dev::*;

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
#[derive(Debug)]
pub struct EncoderElem<'a, const C: u64, T>(pub &'a T);
impl<'a, const C: u64, T: Representation> Serialize for EncoderElem<'a, C, T> {
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
pub struct DecoderElem<const C: u64, T>(std::marker::PhantomData<T>);
impl<const C: u64, T> Default for DecoderElem<C, T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<'de, const C: u64, T: Representation> DeserializeSeed<'de> for DecoderElem<C, T> {
    type Value = T;
    #[inline(always)]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <T as Representation>::deserialize::<C, D>(deserializer)
    }
}
