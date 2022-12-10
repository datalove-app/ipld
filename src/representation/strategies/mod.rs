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
use maybestd::{fmt, str::FromStr};

///
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Strategy {
    // map/struct
    Listpairs = 'l' as u8,
    Stringjoin = 'j' as u8,
    Stringpairs = 'p' as u8,
    Tuple = 't' as u8,
    // union
    Envelope = 'e' as u8,
    Inline = 'i' as u8,
    Keyed = 'y' as u8,
    Kinded = 'k' as u8,
    Bytesprefix = 'b' as u8,
    Stringprefix = 's' as u8,
    // special
    Basic = '1' as u8,
    Ignored = '0' as u8,
    Advanced = 'a' as u8,
}

impl Strategy {
    ///
    pub const fn is_basic(&self) -> bool {
        *self as u8 == Self::Basic as u8
    }

    ///
    pub const fn is_ignored(&self) -> bool {
        *self as u8 == Self::Ignored as u8
    }

    ///
    pub const fn is_advanced(&self) -> bool {
        *self as u8 == Self::Advanced as u8
    }
}

/// An iterator over the elements of a list-like type, whether produced from an
/// in-memory type or from an underlying [`Representation`].
#[doc(hidden)]
pub trait ListIterator<T: Representation> {
    fn size_hint(&self) -> Option<usize>;

    // fn next_ignored(&mut self) -> Result<bool, Error>;

    // fn next<const C: u64>(&mut self) -> Result<Option<T>, Error> {
    //     Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    // }

    // fn next_ref<const C: u64>(&mut self) -> Result<Option<&T>, Error> {
    //     Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    // }

    // fn next_ref_mut<const C: u64>(&mut self) -> Result<Option<&mut T>, Error> {
    //     Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    // }

    // fn next_seed<'a, const C: u64, Ctx: Context>(
    //     &mut self,
    //     seed: SelectorSeed<'a, Ctx, T>,
    // ) -> Result<bool, Error>
    // where
    //     T: Select<Ctx>;

    /// Returns `Ok(true)` if element was found and was successfully
    /// selected/ignored, and `Ok(false)` if iterator was already empty.
    fn next_element_seed<'a, const C: u64, Ctx: Context + 'a, F>(
        &mut self,
        seeder: F,
    ) -> Result<bool, Error>
    where
        T: Select<Ctx>,
        F: FnOnce(usize) -> Result<Option<SelectorSeed<'a, Ctx, T>>, Error>,
    {
        unimplemented!()
    }
}

/// An iterator over the keys and values of a map-like type, whether produced
/// from an in-memory type or from an underlying [`Representation`].
#[doc(hidden)]
pub trait MapIterator<K, V>
where
    K: StringRepresentation,
    <K as FromStr>::Err: fmt::Display,
    V: Representation,
{
    fn size_hint(&self) -> Option<usize>;

    fn next_key<const C: u64>(
        &mut self,
        expected_field_name: Option<&'static str>,
    ) -> Result<Option<K>, Error>;

    fn next_value_ignored(&mut self, field: &Field<'_>) -> Result<(), Error>;

    fn next_value<const C: u64>(&mut self, field: &Field<'_>) -> Result<V, Error> {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    }

    fn next_value_seed<'a, const C: u64, Ctx: Context>(
        &mut self,
        seed: SelectorSeed<'a, Ctx, V>,
    ) -> Result<(), Error>
    where
        V: Select<Ctx>;

    fn next_entry<const C: u64>(&mut self) -> Result<Option<(K, V)>, Error> {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))
    }

    fn next_entry_seed<'a, const C: u64, Ctx: Context + 'a, F>(
        &mut self,
        seeder: F,
    ) -> Result<bool, Error>
    where
        V: Select<Ctx>,
        F: FnOnce(&str) -> Result<Option<SelectorSeed<'a, Ctx, V>>, Error>,
    {
        Err(Error::Custom(anyhow::Error::msg("unimplemented")))

        // let key = if let Some(key) = self.next_key::<C>(None)? {
        //     key
        // } else {
        //     return Ok(true);
        // };

        // // todo
        // let include = filter.map(|pred| pred(&key.to_string())).unwrap_or(true);
        // if include {
        //     self.next_value_seed::<C, Ctx>(seed)?;
        //     Ok(false)
        // } else {
        //     // self.next_value_ignored(&field)?;
        //     Ok(false)
        // }
    }

    /*
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
     */
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
