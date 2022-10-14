mod bytesprefix;
#[macro_use]
mod listpairs;
mod stringjoin;
mod stringpairs;
mod stringprefix;
#[macro_use]
mod tuple;

#[doc(hidden)]
pub use tuple::*;

use crate::dev::*;
use maybestd::marker::PhantomData;

///
#[repr(u8)]
pub enum Strategy {
    DataModel = 'd' as u8,
    BytesPrefix = 'b' as u8,
    Envelope = 'e' as u8,
    Inline = 'i' as u8,
    Keyed = 'k' as u8,
    ListPairs = 'l' as u8,
    StringJoin = 'j' as u8,
    StringPairs = 'p' as u8,
    StringPrefix = 's' as u8,
    Tuple = 't' as u8,
}

impl Strategy {
    // pub const fn dm_kind<const S: u8>(repr_kind: Kind) -> Kind {
    //     match repr_kind {
    //         Kind::Union if S == Self::BytesPrefix as u8 => Kind::Map,
    //         Kind::Union if S == Self::BytesPrefix as u8 => Kind::Map,
    //         Kind::Union if S == Self::BytesPrefix as u8 => Kind::Map,
    //         _ if S == Self::StringPairs as u8 => Kind::String,
    //         Kind::Union if S == Self::StringPrefix as u8 => Kind::Map,
    //         _ if S == Self::Tuple as u8 => Kind::List,
    //         _ => panic!(),
    //     }
    // }

    // pub const fn schema_kind<const S: u8>() -> Kind {
    //     match S {
    //         _ if S == Self::BytesPrefix as u8 => Kind::Union,
    //         _ if S == Self::ListPairs as u8 => Kind::Struct,
    //         _ if S == Self::StringJoin as u8 => Kind::Struct,
    //         _ if S == Self::StringPairs as u8 => Kind::Struct,
    //         _ if S == Self::StringPrefix as u8 => Kind::Union,
    //         _ if S == Self::Tuple as u8 => Kind::Struct,
    //         _ => panic!(),
    //     }
    // }

    // pub const fn repr_kind<const S: u8>() -> Kind {
    //     match S {
    //         _ if S == Self::BytesPrefix as u8 => Kind::Bytes,
    //         _ if S == Self::ListPairs as u8 => Kind::List,
    //         _ if S == Self::StringJoin as u8 => Kind::String,
    //         _ if S == Self::StringPairs as u8 => Kind::String,
    //         _ if S == Self::StringPrefix as u8 => Kind::String,
    //         _ if S == Self::Tuple as u8 => Kind::List,
    //         _ => panic!(),
    //     }
    // }
}

/// An iterator over the elements of a list-like type, whether produced from an
/// in-memory type or from an underlying [`Representation`].
#[doc(hidden)]
pub trait ListIterator<T> {
    fn size_hint(&self) -> Option<usize>;

    fn field(&self) -> Field<'_>;

    fn next_ignored(&mut self) -> Result<bool, Error>;

    fn next<const C: u64>(&mut self) -> Result<Option<T>, Error>
    where
        T: Representation,
    {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    }

    fn next_ref<const C: u64>(&mut self) -> Result<Option<&T>, Error>
    where
        T: Representation,
    {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    }

    fn next_ref_mut<const C: u64>(&mut self) -> Result<Option<&mut T>, Error>
    where
        T: Representation,
    {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    }

    fn next_seed<'a, const C: u64, Ctx: Context>(
        &mut self,
        seed: SelectorSeed<'a, Ctx, T>,
    ) -> Result<bool, Error>
    where
        T: Select<Ctx>;
}

/// An iterator over the keys and values of a map-like type, whether produced
/// from an in-memory type or from an underlying [`Representation`].
#[doc(hidden)]
pub trait MapIterator<K, V> {
    fn size_hint(&self) -> Option<usize>;

    fn field(&self) -> Field<'_>;

    fn next_key<const C: u64>(
        &mut self,
        expected_field_name: Option<&'static str>,
    ) -> Result<Option<K>, Error>
    where
        K: Representation;

    fn next_value_ignored(&mut self, field: &Field<'_>) -> Result<(), Error>;

    fn next_value<const C: u64>(&mut self, field: &Field<'_>) -> Result<V, Error>
    where
        V: Representation,
    {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    }

    fn next_value_seed<'a, const C: u64, Ctx: Context>(
        &mut self,
        seed: SelectorSeed<'a, Ctx, V>,
        // field: &Field<'_>,
    ) -> Result<(), Error>
    where
        K: Representation,
        V: Select<Ctx>;

    // fn next_entry<'a, const C: u64>(
    //     &mut self,
    //     // seed: SelectorSeed<'a, Ctx, V>,
    //     // expected_field_name: Option<F: FnMut(&K) -> bool>
    //     // field: &Field<'_>,
    // ) -> Result<Option<(K, V)>, Error>
    // where
    //     K: Representation,
    // {
    //     unimplemented!()
    // }

    // fn next_entry_seed<'a, const C: u64, T, Ctx: Context>(
    //     &mut self,
    //     root_seed: &mut SelectorSeed<'a, Ctx, T>,
    //     // seeder: F,
    //     // expected_field_name: Option<F: FnOnce(&K) -> bool>
    //     // field: &Field<'_>,
    // ) -> Result<bool, Error>
    // where
    //     T: Representation,
    //     K: Representation,
    //     V: Select<Ctx>,
    //     // F: FnOnce(K) -> Result<Option<SelectorSeed<'a, Ctx, V>>, Error>,
    // {
    //     let key = if let Some(key) = self.next_key::<C>(None)? {
    //         key
    //     } else {
    //         return Ok(true);
    //     };

    //     root_seed.select_field::<C, K, V>(root_seed.)

    //     Ok(false)
    // }

    // ///
    // /// impl DeSeed and Visitor::visit_seq for some type
    // ///     if key matches, create field select seed and call V::__select_map
    // ///     else, ignore value
    // /// return is_empty
    // fn next_entry_seed<'a, const C: u64, Ctx: Context, F>(
    //     &'a mut self,
    //     // seed: SelectorSeed<'a, Ctx, V>,
    //     seeder: F,
    //     // expected_field_name: Option<F: FnOnce(&K) -> bool>
    //     // field: &Field<'_>,
    // ) -> Result<bool, Error>
    // where
    //     Ctx: 'a,
    //     K: Representation,
    //     V: Select<Ctx>,
    //     F: FnOnce(K) -> Result<SelectorSeed<'a, Ctx, V>, Error>,
    // {
    //     let key = if let Some(key) = self.next_key::<C>(None)? {
    //         key
    //     } else {
    //         return Ok(true);
    //     };

    //     self.next_value_seed::<C, Ctx>(seeder(key)?)?;
    //     Ok(false)
    // }
}

// impl<'a, const C: u64, F, Ctx, T, O> RepresentationIterator<'a, C, F, Ctx, T, O>
// where
//     F: FnMut(SelectorSeed<'a, Ctx, T>) -> Result<O, Error>,
//     Ctx: Context,
//     T: Select<Ctx>,
// {
//     // fn next<U: Representation>(&mut self, seed: SelectorSeed<'a, Ctx, T>) -> Result<(), Error> {

//     // }

//     fn next_with_seed(&mut self, seed: SelectorSeed<'a, Ctx, T>) -> Result<O, Error> {
//         (self.func)(seed)
//     }
// }
