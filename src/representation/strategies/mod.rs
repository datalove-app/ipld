mod bytesprefix;
// #[macro_use]
// mod listpairs;
mod stringjoin;
mod stringpairs;
mod stringprefix;
// #[macro_use]
// mod tuple;

pub use list::ListIterator;
pub use map::MapIterator;
// #[doc(hidden)]
// pub use tuple::*;

use crate::dev::*;
use maybestd::{fmt, iter::Peekable, marker::PhantomData, str::FromStr};

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

/// A wrapper around an [`Iterator`] of immutable references to a list dag's
/// elements.
#[derive(Debug)]
pub struct ListIter<'a, I> {
    iter: I,
    index: usize,
    _t: PhantomData<&'a ()>,
}

impl<'a, I> ListIter<'a, I> {
    pub fn new<T>(iter: I) -> Self
    where
        I: Iterator<Item = &'a T>,
        T: Representation + 'a,
    {
        Self {
            iter,
            index: 0,
            _t: PhantomData,
        }
    }
}

/// A wrapper around an [`Iterator`] of mutable references to a list dag's
/// elements.
#[derive(Debug)]
pub struct ListIterMut<'de, I> {
    iter: I,
    index: usize,
    _t: PhantomData<&'de ()>,
}

impl<'de, I> ListIterMut<'de, I> {
    pub fn new<T>(iter: I) -> Self
    where
        I: Iterator<Item = &'de mut T>,
        T: Representation + 'de,
    {
        Self {
            iter,
            index: 0,
            _t: PhantomData,
        }
    }
}

/// A wrapper around an [`Iterator`] of a map dag's key-value pairs.
// #[derive(Debug)]
pub struct MapIter<'de, I: Iterator> {
    iter: Peekable<I>,
    _t: PhantomData<&'de ()>,
}

impl<'de, I: Iterator> MapIter<'de, I> {
    pub fn new<K, V>(iter: I) -> Self
    where
        I: Iterator<Item = (&'de K, &'de V)>,
        K: Representation + 'de,
        V: Representation + 'de,
    {
        Self {
            iter: iter.peekable(),
            _t: PhantomData,
        }
    }
}

/// A wrapper around an [`Iterator`] of a map dag's key-value pairs.
// #[derive(Debug)]
pub struct MapIterMut<'de, I: Iterator> {
    iter: Peekable<I>,
    _t: PhantomData<&'de ()>,
}

impl<'de, I: Iterator> MapIterMut<'de, I> {
    pub fn new<K, V>(iter: I) -> Self
    where
        I: Iterator<Item = (&'de mut K, &'de mut V)>,
        K: Representation + 'de,
        V: Representation + 'de,
    {
        Self {
            iter: iter.peekable(),
            _t: PhantomData,
        }
    }
}

mod list {
    use super::*;

    /// An iterator over the elements of a list-like type, whether produced from
    /// an in-memory type or from an underlying [`Representation`].
    #[doc(hidden)]
    pub trait ListIterator<'de, T: Representation> {
        // type Item<'a>;

        ///
        fn size_hint(&self) -> Option<usize>;

        ///
        fn index(&self) -> usize;

        ///
        fn next_ignored<const MC: u64>(&mut self) -> Result<Option<AstResult<T>>, Error>;
        // where
        // 'a: 'b,
        // 'b: 'de,
        // Ctx: Context,
        // T: Select<Ctx>

        /// Returns `Ok(true)` if element was found and was successfully
        /// selected/ignored, and `Ok(false)` if iterator was already empty.
        fn next_element_seed<'b, const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'b, Ctx, T>,
        ) -> Result<Option<AstResult<T>>, Error>
        where
            // 'a: 'b,
            // 'b: 'de,
            'de: 'b,
            Ctx: Context + 'de,
            T: Select<Ctx>;
    }

    impl<'de, T: Representation, I: ListIterator<'de, T>> ListIterator<'de, T> for &mut I {
        fn size_hint(&self) -> Option<usize> {
            (**self).size_hint()
        }

        fn index(&self) -> usize {
            (**self).index()
        }

        fn next_ignored<const MC: u64>(&mut self) -> Result<Option<AstResult<T>>, Error>
where
            // 'a: 'b,
            // 'b: 'de,
            // Ctx: Context,
            // T: Select<Ctx>,
        {
            (**self).next_ignored::<MC>()
        }

        fn next_element_seed<'b, const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'b, Ctx, T>,
        ) -> Result<Option<AstResult<T>>, Error>
        where
            // 'a: 'b,
            // 'b: 'de,
            'de: 'b,
            Ctx: Context + 'de,
            T: Select<Ctx> + 'de,
        {
            (**self).next_element_seed::<MC, Ctx>(seed)
        }
    }

    impl<'de, T, I> ListIterator<'de, T> for ListIter<'de, I>
    where
        T: Representation + 'de,
        I: Iterator<Item = &'de T>,
    {
        // type Item<'a> = &'a T;

        fn size_hint(&self) -> Option<usize> {
            self.iter.size_hint().1
        }

        fn index(&self) -> usize {
            self.index
        }

        fn next_ignored<const MC: u64>(&mut self) -> Result<Option<AstResult<T>>, Error>
where
            // 'a: 'b,
            // 'b: 'de,
            // Ctx: Context,
            // T: Select<Ctx>,
        {
            if self.iter.next().is_some() {
                self.index += 1;
                Ok(Some(AstResult::Ok))
            } else {
                Ok(None)
            }
        }

        fn next_element_seed<'b, const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'b, Ctx, T>,
        ) -> Result<Option<AstResult<T>>, Error>
        where
            // 'a: 'b,
            // 'b: 'de,
            'de: 'b,
            Ctx: Context + 'de,
            T: Select<Ctx> + 'de,
        {
            let found = match self.iter.next() {
                None => false,
                Some(dag) => {
                    dag.__select(seed)?;
                    true
                }
            };

            if found {
                self.index += 1;
                Ok(Some(AstResult::Ok))
            } else {
                Ok(None)
            }
        }
    }

    impl<'de, T, I> ListIterator<'de, T> for ListIterMut<'de, I>
    where
        T: Representation + 'de,
        I: Iterator<Item = &'de mut T>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.iter.size_hint().1
        }

        fn index(&self) -> usize {
            self.index
        }

        fn next_ignored<const MC: u64>(&mut self) -> Result<Option<AstResult<T>>, Error>
where
            // 'a: 'b,
            // 'b: 'de,
            // Ctx: Context,
            // T: Select<Ctx>,
        {
            if self.iter.next().is_some() {
                self.index += 1;
                Ok(Some(AstResult::Ok))
            } else {
                Ok(None)
            }
        }

        fn next_element_seed<'b, const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'b, Ctx, T>,
        ) -> Result<Option<AstResult<T>>, Error>
        where
            // 'a: 'b,
            // 'b: 'de,
            'de: 'b,
            Ctx: Context + 'de,
            T: Select<Ctx> + 'de,
        {
            let found = match self.iter.next() {
                None => false,
                Some(dag) => {
                    dag.__patch(seed)?;
                    true
                }
            };

            if found {
                self.index += 1;
                Ok(Some(AstResult::Ok))
            } else {
                Ok(None)
            }
        }
    }
}

mod map {
    use super::*;

    /// An iterator over the keys and values of a map-like type, whether produced
    /// from an in-memory type or from an underlying [`Representation`].
    #[doc(hidden)]
    pub trait MapIterator<'a, K, V>
    where
        K: StringRepresentation,
        <K as FromStr>::Err: fmt::Display,
        V: Representation,
    {
        ///
        fn size_hint(&self) -> Option<usize>;

        ///
        fn next_key<const MC: u64>(
            &mut self,
            expected: Option<&Field<'static>>,
        ) -> Result<Option<AstResult<K>>, Error>;

        ///
        fn next_value_ignored<const MC: u64, Ctx>(&mut self) -> Result<AstResult<V>, Error>
        where
            Ctx: Context + 'a,
            V: Select<Ctx>;

        ///
        fn next_value_seed<const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'a, Ctx, V>,
        ) -> Result<AstResult<V>, Error>
        where
            Ctx: Context + 'a,
            V: Select<Ctx>;

        ///
        fn next_entry_seed<const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'a, Ctx, V>,
        ) -> Result<Option<AstResult<V>>, Error>
        where
            Ctx: Context + 'a,
            V: Select<Ctx>,
        {
            match self.next_key::<MC>(None)? {
                None => Ok(None),
                Some(_) => self.next_value_seed::<MC, Ctx>(seed).map(Some),
            }
        }

        /*
        // fn next_value_ignored<'a, const C: u64, Ctx: Context, V>(&mut self) -> Result<Option<AstResult<V>>, Error>
        // where V: Select<Ctx> {}

        // fn next_value<const C: u64>(&mut self, field: &Field<'_>) -> Result<V, Error> {
        //     Err(Error::Custom(anyhow::Error::msg("unimplemented")))
        // }

        // fn next_value_seed<'a, const C: u64, Ctx: Context>(
        //     &mut self,
        //     seed: SelectorSeed<'a, Ctx, V>,
        // ) -> Result<AstResult<V>, Error>
        // where
        //     V: Select<Ctx>;

        // fn next_entry<const C: u64>(&mut self) -> Result<Option<(K, V)>, Error> {
        //     Err(Error::Custom(anyhow::Error::msg("unimplemented")))
        // }

        // fn next_entry_seed<'a, const C: u64, Ctx: Context + 'a, F>(
        //     &mut self,
        //     seeder: F,
        // ) -> Result<Option<AstResult<V>>, Error>
        // where
        //     V: Select<Ctx>,
        //     F: FnOnce(&str) -> Result<Option<SelectorSeed<'a, Ctx, V>>, Error>,
        // {
        //     Err(Error::Custom(anyhow::Error::msg("unimplemented")))
        //
        //     // let key = if let Some(key) = self.next_key::<C>(None)? {
        //     //     key
        //     // } else {
        //     //     return Ok(true);
        //     // };
        //
        //     // // todo
        //     // let include = filter.map(|pred| pred(&key.to_string())).unwrap_or(true);
        //     // if include {
        //     //     self.next_value_seed::<C, Ctx>(seed)?;
        //     //     Ok(false)
        //     // } else {
        //     //     // self.next_value_ignored(&field)?;
        //     //     Ok(false)
        //     // }
        // }
         */

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
        //     &mut self,
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

    impl<'de, K, V, I> MapIterator<'de, K, V> for MapIter<'de, I>
    where
        K: StringRepresentation + 'de,
        <K as FromStr>::Err: fmt::Display,
        V: Representation + 'de,
        I: Iterator<Item = (&'de K, &'de V)>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.iter.size_hint().1
        }

        fn next_key<const MC: u64>(
            &mut self,
            expected: Option<&Field<'static>>,
        ) -> Result<Option<AstResult<K>>, Error> {
            match self.iter.peek() {
                None => Ok(None),
                // todo: assert match expected
                Some((k, _)) => {
                    // self.field.replace(
                    //     k.as_field()
                    //         .ok_or(Error::explore_key_failure::<K>(expected))?,
                    // );
                    Ok(Some(AstResult::Ok))
                }
            }
        }

        fn next_value_ignored<const MC: u64, Ctx>(&mut self) -> Result<AstResult<V>, Error>
        where
            Ctx: Context + 'de,
            V: Select<Ctx>,
        {
            // let field = self
            //     .field
            //     .take()
            //     .ok_or_else(|| Error::explore_key_failure::<K>(None))?;
            // self.iter
            //     .next()
            //     .ok_or_else(|| Error::explore_value_failure::<V>(&field));
            Ok(AstResult::Ok)
        }

        fn next_value_seed<const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'de, Ctx, V>,
        ) -> Result<AstResult<V>, Error>
        where
            Ctx: Context + 'de,
            V: Select<Ctx>,
        {
            // let field = self
            //     .field
            //     .take()
            //     .ok_or_else(|| Error::explore_key_failure::<K>(None))?;
            // let (_, dag) = self
            //     .iter
            //     .next()
            //     .ok_or_else(|| Error::explore_value_failure::<V>(&field))?;

            // dag.__select(seed)?;
            Ok(AstResult::Ok)
        }
    }

    impl<'de, K, V, I> MapIterator<'de, K, V> for MapIterMut<'de, I>
    where
        K: StringRepresentation + 'de,
        <K as FromStr>::Err: fmt::Display,
        V: Representation + 'de,
        I: Iterator<Item = (&'de mut K, &'de mut V)>,
    {
        fn size_hint(&self) -> Option<usize> {
            self.iter.size_hint().1
        }

        fn next_key<const MC: u64>(
            &mut self,
            expected: Option<&Field<'static>>,
        ) -> Result<Option<AstResult<K>>, Error> {
            match self.iter.peek() {
                None => Ok(None),
                // todo: assert match expected
                Some((k, _)) => {
                    // self.field.replace(
                    //     k.as_field()
                    //         .ok_or(Error::explore_key_failure::<K>(expected))?,
                    // );
                    Ok(Some(AstResult::Ok))
                }
            }
        }

        fn next_value_ignored<const MC: u64, Ctx>(&mut self) -> Result<AstResult<V>, Error>
        where
            Ctx: Context + 'de,
            V: Select<Ctx>,
        {
            // let field = self
            //     .field
            //     .take()
            //     .ok_or_else(|| Error::explore_key_failure::<K>(None))?;
            // self.iter
            //     .next()
            //     .ok_or_else(|| Error::explore_value_failure::<V>(&field));
            Ok(AstResult::Ok)
        }

        fn next_value_seed<const MC: u64, Ctx>(
            &mut self,
            seed: SelectorSeed<'de, Ctx, V>,
        ) -> Result<AstResult<V>, Error>
        where
            Ctx: Context + 'de,
            V: Select<Ctx>,
        {
            // let field = self
            //     .field
            //     .take()
            //     .ok_or_else(|| Error::explore_key_failure::<K>(None))?;
            // let (k, dag) = self
            //     .iter
            //     .next()
            //     .ok_or_else(|| Error::explore_value_failure::<V>())?;

            // dag.__patch(seed)?;
            Ok(AstResult::Ok)
        }
    }
}
