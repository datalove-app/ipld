use crate::dev::*;
use macros::derive_more::From;
use std::{cmp::Ord, collections::BTreeMap, ops::Deref, str::FromStr};

// List

#[derive(Debug, From)]
pub enum List<T> {
    Full(Vec<T>),
    // Selection(Selector, Vec<T>),
}

// impl<I> Representation for Vec<I>
// where
//     I: Representation + Send + Sync,
// {
//     const NAME: &'static str = "Vec<I>";
//     const SCHEMA: &'static str = "[I]";
//     const KIND: Kind = Kind::List;
//     const FIELDS: Fields = Fields::List(I::KIND);
// }

impl<T> AsRef<Vec<T>> for List<T> {
    fn as_ref(&self) -> &Vec<T> {
        match &self {
            Self::Full(vec) => vec,
            // Self::Selection(_, vec) => vec,
        }
    }
}

impl<T> Deref for List<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.as_ref()
    }
}

impl<I> Representation for Vec<I>
where
    I: Representation + Send + Sync,
{
    const NAME: &'static str = "Vec<I>";
    // const SCHEMA: &'static str = "[I]";
    // const KIND: Kind = Kind::List;
    // const FIELDS: Fields = Fields::List(Field::new::<I>(()));
}

// Map

#[derive(Debug, From)]
pub enum Map<K, V> {
    Full(BTreeMap<K, V>),
    // Selection(Selector, BTreeMap<K, V>),
}

// impl<K, V> Representation for Map<K, V>
// where
//     K: Representation + Send + Sync + Ord + ToString + FromStr,
//     V: Representation + Send + Sync,
// {
//     const NAME: &'static str = "Map<K, V>";
//     const SCHEMA: &'static str = "{K:V}";
//     const KIND: Kind = Kind::Map;
//     const FIELDS: Fields = Fields::Map { key: K::KIND, value: V::KIND };
// }

impl<K, V> AsRef<BTreeMap<K, V>> for Map<K, V> {
    fn as_ref(&self) -> &BTreeMap<K, V> {
        match &self {
            Self::Full(map) => map,
            // Self::Selection(_, map) => map,
        }
    }
}

impl<K, V> Representation for BTreeMap<K, V>
where
    K: Representation + Send + Sync + Ord + ToString + FromStr,
    V: Representation + Send + Sync,
{
    const NAME: &'static str = "BTreeMap<K, V>";
    // const SCHEMA: &'static str = "{K:V}";
    // const KIND: Kind = Kind::Map;
    // const FIELDS: Fields = Fields::Map {
    //     key: Field::new::<K>(()),
    //     value: Field::new::<V>(()),
    // };
}

// // impl<Ctx, CtxT, T> Representation<Ctx> for Option<T>
// where
//     Ctx: Context + Send + Sync,
//     CtxT: Context + FromContext<Ctx> + Send + Sync,
//     T: Representation<CtxT>,
// {
//     const NAME: &'static str = "optional T";

//     async fn resolve<'a>(
//         selection: &Selector,
//         executor: &'a Executor<'a, Ctx>,
//     ) -> Result<Self, ()> {
//         unimplemented!()
//     }

//     async fn resolve_field<'a, T>(
//         &'a self,
//         selection: &Selector,
//         executor: &'a Executor<'a, Ctx>,
//     ) -> Result<T, ()>
//     where
//         T: Representation<Ctx>,
//     {
//         unimplemented!()
//     }
// }

// // impl<Ctx, Co, R, W, T> Representation<Ctx, Co, R, W> for Vec<T>
// where
//     Ctx: Context<Co, R, W> + Send + Sync,
//     Co: CodecExt<Self> + CodecExt<T>,
//     R: Read + Seek + Unpin + Send,
//     W: Write + Seek + Unpin + Send,
//     T: Representation<Ctx, Co, R, W> + Send + Sync,
// {
//     async fn read(ctx: &Ctx) -> Result<Self, Error>
//     where
//         Co: 'async_trait,
//         R: 'async_trait,
//         W: 'async_trait,
//     {
//         use ResolveRange::*;
//         match ctx.resolve_range().await {
//             Full => {}
//         }

//         // let major = u8::read_cbor(ctx.reader()).await?;
//         // let len = read_list_len(ctx.reader(), major).await?;
//         // let mut list: Self = Vec::with_capacity(len);
//         // for idx in 0..len {
//         //     if ctx.try_handle(PushElement::from(idx)).await {
//         //         list.push(T::read(ctx).await?);
//         //         ctx.try_handle(PopElement).await;
//         //     } else {
//         //         T::seek_read(ctx).await?;
//         //     }
//         // }
//         // Ok(list)
//         Ok(Vec::new())
//     }

//     async fn write(&self, ctx: &Ctx) -> Result<(), Error>
//     where
//         Co: 'async_trait,
//         R: 'async_trait,
//         W: 'async_trait,
//     {
//         // write_u64(ctx.writer(), 4, self.len() as u64).await?;
//         // for (idx, value) in self.iter().enumerate() {
//         //     if ctx.try_handle(PushElement::new(idx)).await {
//         //         value.write(ctx).await?;
//         //         ctx.try_handle(PopElement).await;
//         //     } else {
//         //         value.seek_write(ctx).await?;
//         //     }
//         // }
//         Ok(())
//     }
// }

// // impl<'a, R, W, K, V> Representation<R, W> for BTreeMap<K, V>
// where
//     R: Read + Seek + Unpin + Send,
//     W: Write + Seek + Unpin + Send,
//     K: 'static + Ord + Clone + Into<IpldIndex<'static>> + Representation<R, W> + Send + Sync,
//     // NOTE: why 'static?
//     V: 'static + Representation<R, W> + Send + Sync,
// {
// //    async fn read_len<C>(ctx: &mut C) -> Result<usize, Error>
// //    where
// //        R: 'async_trait,
// //        W: 'async_trait,
// //        C: Context<R, W> + Send,
// //    {
// ////        match u8::read_cbor(ctx.reader()).await? {
// ////            0xf6 | 0xf7 => Ok(<()>::read_len(ctx).await?),
// ////            _ => Ok(T::read_len(ctx).await?),
// ////        }
// //    }
// //
// //    async fn write_len<C>(&self, ctx: &mut C) -> Result<usize, Error>
// //    where
// //        R: 'async_trait,
// //        W: 'async_trait,
// //        C: Context<R, W> + Send,
// //    {
// ////        match self {
// ////            Some(t) => T::write_len(t, ctx).await,
// ////            None => <()>::write_len(ctx).await,
// ////        }
// //    }

//     async fn read<C>(ctx: &mut C) -> Result<Self, Error>
//     where
//         R: 'async_trait,
//         W: 'async_trait,
//         C: Context<R, W> + Send,
//     {
//         let major = u8::read(ctx).await?;
//         let len = read_map_len(ctx.reader(), major).await?;
//         let mut map: Self = BTreeMap::new();
//         for _ in 0..len {
//             let key = K::read(ctx).await?;
//             if ctx.try_handle(PushElement::new(key.clone())).await {
//                 map.insert(key, V::read(ctx).await?);
//                 ctx.try_handle(PopElement).await;
//             } else {
//                 V::seek_read(ctx).await?;
//             }
//         }
//         Ok(map)
//     }

//     async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
//     where
//         R: 'async_trait,
//         W: 'async_trait,
//         C: Context<R, W> + Send,
//     {
//         write_u64(ctx.writer(), 5, self.len() as u64).await?;
//         for (key, value) in self {
//             if ctx.try_handle(PushElement::new(key.clone())).await {
//                 key.write(ctx).await?;
//                 value.write(ctx).await?;
//                 ctx.try_handle(PopElement).await;
//             } else {
//                 key.seek_write(ctx).await?;
//                 value.seek_write(ctx).await?;
//             }
//         }
//         Ok(())
//     }
// }

// #[inline]
// pub(crate) async fn read_list_len<R>(r: &mut R, major: u8) -> Result<usize, Error>
// where
//     R: Read + Unpin + Send,
// {
//     let len = match major {
//         0x80..=0x97 => major as usize - 0x80,
//         0x98 => u8::read_cbor(r).await? as usize,
//         0x99 => u16::read_cbor(r).await? as usize,
//         0x9a => u32::read_cbor(r).await? as usize,
//         0x9b => {
//             let len = u64::read_cbor(r).await?;
//             if len > usize::max_value() as u64 {
//                 return Err(Error::Cbor(CborError::LengthOutOfRange));
//             }
//             len as usize
//         }
//         _ => return Err(Error::Cbor(CborError::UnexpectedCode)),
//     };
//     Ok(len)
// }

// #[inline]
// pub(crate) async fn read_map_len<R>(r: &mut R, major: u8) -> Result<usize, Error>
// where
//     R: Read + Unpin + Send,
// {
//     let len = match major {
//         0xa0..=0xb7 => major as usize - 0xa0,
//         0xb8 => u8::read_cbor(r).await? as usize,
//         0xb9 => u16::read_cbor(r).await? as usize,
//         0xba => u32::read_cbor(r).await? as usize,
//         0xbb => {
//             let len = u64::read_cbor(r).await?;
//             if len > usize::max_value() as u64 {
//                 return Err(Error::Cbor(CborError::LengthOutOfRange));
//             }
//             len as usize
//         }
//         _ => return Err(Error::Cbor(CborError::UnexpectedCode)),
//     };
//     Ok(len)
// }

// // impl<Ctx, Co, R, W, T> Representation<Ctx, Co, R, W> for Option<T>
// where
//     Co: CodecExt<Self> + CodecExt<()> + CodecExt<T>,
//     R: Read + Seek + Unpin + Send,
//     W: Write + Seek + Unpin + Send,
//     Ctx: Context<Co, R, W> + Send + Sync,
//     T: Representation<Ctx, Co, R, W> + Sync,
// {
//     #[inline]
//     async fn read(ctx: &Ctx) -> Result<Self, Error>
//     where
//         Co: 'async_trait,
//         R: 'async_trait,
//         W: 'async_trait,
//     {
//         use ResolveRange::*;
//         // let (prefix)
//         // if <()>::read(ctx).await? == () {
//         //     return Ok(None);
//         // };

//         // TODO: get lengths first
//         let (prefix, len, suffix) = codec_read_offsets!(ctx, T);
//         if let Ok(t) = T::read(ctx).await {
//             return Ok(Some(t));
//         }

//         ctx.reader().seek(SeekFrom::Current())
//         Err(Error::Context("".into()))

//         // TODO: else, rewind the len and try again

//         // match u8::read_cbor(ctx.reader()).await? {
//         //     0xf6 | 0xf7 => Ok(None),
//         //     _ => match T::read(ctx).await {
//         //         Ok(t) => Ok(Some(t)),
//         //         Err(Error::Cbor(CborError::UnexpectedCode)) => Ok(None),
//         //         Err(err) => Err(err),
//         //     },
//         // }
//     }

//     #[inline]
//     async fn write(&self, ctx: &Ctx) -> Result<(), Error>
//     where
//         Co: 'async_trait,
//         R: 'async_trait,
//         W: 'async_trait,
//     {
//         match self {
//             Some(value) => value.write(ctx).await,
//             None => {
//                 <()>::write(&(), ctx).await?;
//                 Ok(())
//             }
//         }
//     }
// }

// list

//impl<'a, C, T> TryFrom<BorrowedIpld<'a, C>> for Vec<T>
//where
//    C: CodecExt,
//    T: TryFrom<BorrowedIpld<'a, C>>,
//    <T as TryFrom<BorrowedIpld<'a, C>>>::Error: Into<Error>,
//    for<'b> &'b T: TryFrom<BorrowedIpld<'a, C>>,
//    for<'b> <&'b T as TryFrom<BorrowedIpld<'a, C>>>::Error: Into<Error>
//{
//    type Error = Error;
//
//    fn try_from(ipld: BorrowedIpld<'a, C>) -> Result<Self, Self::Error> {
//        match ipld {
//            BorrowedIpld::List(BorrowedIpldListIter::Slice(iter)) => {
//                let vec = iter
//                    .into_inner()
//                    .map(|ipld| <BorrowedIpld<'a, C>>::try_into(ipld))
//                    .collect::<Result<Vec<T>, _>>()
//                    .map_err(Into::into)?;
//                Ok(vec)
//            }
//            BorrowedIpld::List(BorrowedIpldListIter::Vec(iter)) => {
//                let vec = iter
//                    .into_inner()
//                    .map(T::try_from)
//                    .collect::<Result<Vec<T>, _>>()
//                    .map_err(Into::into)?;
//                Ok(vec)
//            }
//            _ => Err(Error::Ipld(IpldError::NotList)),
//        }
//    }
//}
//
//impl<'a, C, T> TryInto<BorrowedIpld<'a, C>> for &'a Vec<T>
//where
//    C: CodecExt,
//    for<'b> &'b T: TryInto<BorrowedIpld<'a, C>>,
//    for<'b> <&'b T as TryInto<BorrowedIpld<'a, C>>>::Error: Into<Error>,
//{
//    type Error = Error;
//
//    fn try_into(self) -> Result<BorrowedIpld<'a, C>, Self::Error> {
//        let list = self
//            .iter()
//            .map(|t| t.try_into().or(Err(Error::Ipld(IpldError::NotKey))))
//            .collect::<Result<Vec<BorrowedIpld<'a, C>>, Self::Error>>()?;
//        Ok(BorrowedIpld::List(list))
//    }
//}

// map

//impl<'a, C, K, V, L, M> TryFrom<BorrowedIpld<'a, C>> for BTreeMap<K, V>
//where
//    C: CodecExt,
//    K: FromStr + Ord,
//    <K as FromStr>::Err: Into<Error>,
//    V, L, M: TryFrom<BorrowedIpld<'a, C>>,
//    <V a, L, Ms TryFrom<BorrowedIpld<'a, C>>>::Error: Into<Error>,
//{
//    type Error = Error;
//
//    fn try_from(ipld: BorrowedIpld<'a, C>) -> Result<Self, Self::Error> {
//        match ipld {
//            BorrowedIpld::Map(map) => map.into_iter().fold(
//                Ok(Self::new()),
//                |map_res: Result<Self, Self::Error>, (k, v)| {
//                    map_res.and_then(|mut map| {
//                        map.insert(
//                            k.parse().map_err(Into::into)?,
//                            V::try_from(v).map_err(Into::into)?,
//                        );
//                        Ok(map)
//                    })
//                },
//            ),
//            _ => Err(Error::Ipld(IpldError::NotMap)),
//        }
//    }
//}
//
//impl<'a, C, V> TryInto<BorrowedIpld<'a, C>> for &'a BTreeMap<&'a str, V>
//where
//    C: CodecExt,
//    for<'b> &'b V: TryInto<BorrowedIpld<'a, C>>,
//    for<'b> <&'b V as TryInto<BorrowedIpld<'a, C>>>::Error: Into<Error>,
//{
//    type Error = Error;
//
//    fn try_into(self) -> Result<BorrowedIpld<'a, C>, Self::Error> {
//        let map = self.into_iter().fold(
//            Ok(BTreeMap::new()),
//            |map_res: Result<BTreeMap<&'a str, BorrowedIpld<'a, C>>, Self::Error>, (k, v)| {
//                map_res.and_then(|mut map| {
//                    map.insert(*k, v.try_into().or(Err(Error::Ipld(IpldError::NotKey)))?);
//                    Ok(map)
//                })
//            },
//        )?;
//        Ok(BorrowedIpld::Map(map))
//    }
//}
