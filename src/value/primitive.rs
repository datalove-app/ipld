use crate::dev::*;

impl Representation for () {
    const NAME: &'static str = "Null";
}
impl_root_select!(() => Matcher);

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! def_primitive {
    ($type:ident: $kind:ident, $schema:expr) => {
        impl Representation for $type {
            const NAME: &'static str = $schema;
            // const SCHEMA: &'static str = $schema;
            // const KIND: Kind = Kind::$kind;
        }

        $crate::impl_root_select!($type => Matcher);
    };
}

def_primitive!(bool: Boolean, "Boolean");
def_primitive!(i8: Integer, "Int8");
def_primitive!(i16: Integer, "Int16");
def_primitive!(i32: Integer, "Int32");
def_primitive!(i64: Integer, "Int64");
def_primitive!(i128: Integer, "Int128");
def_primitive!(u8: Integer, "Uint8");
def_primitive!(u16: Integer, "Uint16");
def_primitive!(u32: Integer, "Uint32");
def_primitive!(u64: Integer, "Uint64");
def_primitive!(u128: Integer, "Uint128");
def_primitive!(f32: Float, "Float32");
def_primitive!(f64: Float, "Float64");
def_primitive!(String: String, "String");

impl Representation for &str {
    const NAME: &'static str = "str";
}

impl<T> Representation for Option<T> {
    const NAME: &'static str = "Null";
}
impl_root_select!(Matcher {
    impl<Ctx, T> Select<Selector, Ctx> for Option<T>
    where
        Ctx: Context,
        T: Representation + 'static
});

// TODO: cid

// /// Default implementations that delegate directly to the underlying
// /// `ReadCbor`/`WriteCbor`.
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! primitive_representation_impl {
//     ($name:tt : $type:tt) => {
//         //         impl Representation for $type
//         where
//             Ctx: Context + Sync,
//         {
//             const NAME: &'static str = ::std::stringify!($type);
// const SCHEMA: &'static str = "";
//             const HAS_LINKS: bool = false;
//             const IS_LINK: bool = false;

//             async fn resolve<'a>(selection: &Selector, executor: &'a Executor<'a, Ctx>) -> Result<$type, ()> {
//                 unimplemented!()
//             }

//             async fn resolve_field<'a, T>(
//                 &'a self,
//                 selection: &Selector,
//                 executor: &'a Executor<'a, Ctx>,
//             ) -> Result<T, ()>
//             where
//                 T: Representation<Ctx>
//             {
//                 unimplemented!()
//             }
//         }

//    //    impl<Ctx, Co, R, W> Representation<Ctx, Co, R, W> for $type
//    where
//        Co: CodecExt<Self>,
//        R: Read + Seek + Unpin + Send,
//        W: Write + Seek + Unpin + Send,
//        Ctx: Context<Co, R, W> + Send + Sync,
//    {
//        #[inline]
//        async fn read(ctx: &Ctx) -> Result<Self, Error>
//        where
//            Co: 'async_trait,
//            R: 'async_trait,
//            W: 'async_trait,
//        {
//            use ResolveRange::*;
//            match ctx.resolve_range().await {
//                Full => Ok(codec_read!(ctx, $type)),
//                None => Ok(Self::default()),
//                Seek => {
//                    codec_read_seek_to_end!(ctx, $type);
//                    Ok(Self::default())
//                }
//                range => Err(Error::Context(::std::format!(
//                    "Cannot use range `{:?}` to read into {}",
//                    range,
//                    ::std::stringify!($type),
//                ))),
//            }
//        }

//        #[inline]
//        async fn write(&self, ctx: &Ctx) -> Result<(), Error>
//        where
//            Co: 'async_trait,
//            R: 'async_trait,
//            W: 'async_trait,
//        {
//            use ResolveRange::*;
//            match ctx.resolve_range().await {
//                Full => Ok(codec_write!(self, ctx, $type)),
//                None => Ok(()),
//                Seek => {
//                    codec_write_seek_to_end!(self, ctx, $type);
//                    Ok(())
//                }
//                range => Err(Error::Context(::std::format!(
//                    "Cannot use range `{:?}` to write into {}",
//                    range,
//                    ::std::stringify!($type),
//                ))),
//            }
//        }
//    }
//    };
//    (String) => {
//        primitive_representation_impl!(@range String);
//    };
//    (Bytes) => {
//        primitive_representation_impl!(@range Bytes);
//    };
//    (@range $type:tt) => {
//        //        impl<Ctx, Co, R, W> Representation<Ctx, Co, R, W> for $type
//        where
//            Co: CodecExt<Self>,
//            R: Read + Seek + Unpin + Send,
//            W: Write + Seek + Unpin + Send,
//            Ctx: Context<Co, R, W> + Send + Sync,
//        {
//            ///
//            #[inline]
//            async fn read(ctx: &Ctx) -> Result<Self, Error>
//            where
//                Co: 'async_trait,
//                R: 'async_trait,
//                W: 'async_trait,
//            {
//                use ResolveRange::*;
//                match ctx.resolve_range().await {
//                    Full => Ok(codec_read!(ctx, $type)),
//                    None => Ok(Self::default()),
//                    Seek => {
//                        codec_read_seek_to_end!(ctx, $type);
//                        Ok(Self::default())
//                    }
//                    Range { range, location } => {
//                        unimplemented!()
//                        //                let (prefix, len, suffix) = codec_read_offsets!(ctx, String);
//                        //                let Range { start, end } = range;

//                        //                if end == start {
//                        //                    return codec_read_seek_to_end!(ctx, String);
//                        //                } else if (end - start) == len && location == 0 {
//                        //                    return Ok(codec_read!(ctx, String));
//                        //                } else if range.contains(&len) {
//                        //                    return Err(Error::Context(
//                        //                        "Range encompasses more than the String length"
//                        // .into(),
//                        //                    ))?;
//                        //                }

//                        //                if start != 0 {
//                        //                    ctx.reader().seek(Current(start.try_into()?)).await?;
//                        //                }
//                        //                let mut buf = Vec::with_capacity(len);
//                        //                ctx.reader()
//                        //                    .take((end - start).try_into()?)
//                        //                    .read_to_end(&mut buf)
//                        //                    .await?;

//                        //                if len != end {
//                        //                    ctx.reader().seek(Current((len - end).try_into()?)).await?;
//                        //                }

//                        //                Ok(String::from_utf8_lossy(&buf[start..end]).into())
//                    }
//                    range => Err(Error::Context(::std::format!(
//                        "Cannot use range `{:?}` to read into {}",
//                        range,
//                        ::std::stringify!($type),
//                    ))),
//                }
//            }
//            //
//            ///
//            /// If provided with the right `Context`, you can choose to `Write` the
//            /// type into an existing byte stream (skipping `Write`ing this type
//            /// altogether), or `Write` a part of it's bytes to a particular location
//            /// in the stream.
//            #[inline]
//            async fn write(&self, ctx: &Ctx) -> Result<(), Error>
//            where
//                Co: 'async_trait,
//                R: 'async_trait,
//                W: 'async_trait,
//            {
//                match ctx.resolve_range().await {
//                    Resolve::Full => Ok(codec_write!(self, ctx, $type)),
//                    Resolve::None => Ok(()),
//                    Resolve::Seek => {
//                        codec_write_seek_to_end!(self, ctx, $type);
//                        Ok(())
//                    }
//                    Resolve::Range { range, location } => {
//                        unimplemented!()

//                        //                let (prefix, len, suffix) = codec_write_offsets!(self, ctx, String);
//                        //                let Range { start, end } = range;
//                        //                let desired_len = end - start;

//                        //                if end == start {
//                        //                    return codec_write_seek_to_end!(self, ctx, String);
//                        //                } else if (end - start) == len {
//                        //                    return Ok(codec_write!(self, ctx, String));
//                        //                } else if range.contains(&len) {
//                        //                    return Err(Error::Context(
//                        //                        "Range encompasses more than the string length"
//                        // .into(),
//                        //                    ))?;
//                        //                }

//                        // TODO: read existing offsets to know where to start skipping
//                        //                let actual_len = prefix as usize + desired_len + suffix as usize;
//                        //                let (r_prefix, r_len, r_suffix) = codec_read_offsets!(ctx, String);
//                        //                if (r_prefix as usize + r_len + r_suffix as usize) ==
//                        //                    actual_len || {
//                        //                    return Err(Error::CodecExt(format!()))
//                        //                }
//                        //
//                        //                let mut buf = Vec::with_capacity(actual_len);
//                        //                <Co as CodecExt<Self>>::write(self, &mut buf)
//                        //                    .await
//                        //                    .map_err(|err| Error::CodecExt(err.to_string()))?

//                        //    ctx.writer()
//                        //        .seek(Current(prefix + start.try_into()?))
//                        //        .await?;
//                        //    ctx.writer()
//                        //        .write_all(&buf[(prefix as usize + start)..len])
//                        //        .await?;
//                        //    ctx.writer()
//                        //        .seek(Current(suffix + (len - end).try_into()?))
//                        //        .await?;

//                        //    Ok(())
//                    }
//                    range => Err(Error::Context(::std::format!(
//                        "Cannot use range `{:?}` to write into {}",
//                        range,
//                        ::std::stringify!($type),
//                    ))),
//                }
//            }
//        }
//    };
// }

// #[cfg(test)]
// mod test {
//     struct Sample {
//         a: bool,
//         b: u8,
//     }

//     derive_ipld_for_struct!(
//         struct Sample {
//             a: bool,
//             b: u8,
//         }
//     );
// }

////impl FromIpld for () {
//    type Context = ();
//
//    async fn from_ipld<Ctx>(ipld: Ipld, _ctx: Ctx) -> Result<Self, Error>
//    where
//        Ctx: Into<Self::Context>
//    {
//        match ipld {
//            Ipld::Null => Ok(()),
//            _ => Err(Error::Ipld(IpldError::NotNull)),
//        }
//    }
//}
//
////impl ToIpld for () {
//    type Context = ();
//
//    async fn to_ipld<Ctx>(&self, _ctx: Ctx) -> Result<BorrowedIpld, Error>
//    where
//        Ctx: Into<Self::Context>
//    {
//        Ok(BorrowedIpld::Null)
//    }
//}

// primitive_representation_impl!(Null: ());
// primitive_representation_impl!(Bool: bool);
// primitive_representation_impl!(Int8: i8);
// primitive_representation_impl!(Int16: i16);
// primitive_representation_impl!(Int32: i32);
// primitive_representation_impl!(Int64: i64);
// primitive_representation_impl!(Int128: i128);
// primitive_representation_impl!(Uint8: u8);
// primitive_representation_impl!(Uint16: u16);
// primitive_representation_impl!(Uint32: u32);
// primitive_representation_impl!(Uint64: u64);
// primitive_representation_impl!(Uint128: u128);
// primitive_representation_impl!(Float32: f32);
// primitive_representation_impl!(Float64: f64);
//primitive_representation_impl!(Bytes);
//primitive_representation_impl!(String);
//primitive_representation_impl!(Cid);
