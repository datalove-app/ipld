use crate::dev::*;
use macros::derive_more::{AsRef, From};

/// A nothing type.
pub type Null = ();

impl Representation for Null {
    const NAME: &'static str = "Null";
    const SCHEMA: &'static str = "type Null null";
    const KIND: Kind = Kind::Null;

    // fn r#match<'de, 'a, C, D>(
    //     seed: ContextSeed<'a, C, Self, Self>,
    //     deserializer: D,
    // ) -> Result<Self, D::Error>
    // where
    //     C: Context,
    //     D: Deserializer<'de>,
    // {
    //     let self_ = seed.deserialize(deserializer)?;
    // }
}

impl_ipld! { @visitor {} {} Null => Null {
    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Nothing")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.selector {
            Selector::Matcher(Matcher { label, .. }) => {
                match self.state.mode() {
                    SelectorState::DAG_MATCH_MODE => Ok(Some(())),
                    SelectorState::NODE_MODE => {
                        self.send_matched(().into(), label.clone()).map_err(E::custom)?;
                        Ok(None)
                    },
                    SelectorState::DAG_MODE => {
                        self.send_dag((), label.clone()).map_err(E::custom)?;
                        Ok(None)
                    },
                }
            },
            selector => Err(Error::unsupported_selector::<Null, Null>(selector)).map_err(E::custom)
        }
    }
}}

impl_ipld! { @deseed {} {} Null => Null {
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_unit(self)
    }
}}

impl_ipld! { @select_self {} {} Null }

schema! {
    /// A `bytes` type.
    #[ipld_attr(internal)]
    #[derive(AsRef, Clone, Debug, Eq, From, Hash, PartialEq)]
    #[as_ref(forward)]
    #[from(forward)]
    pub type Bytes bytes;
}

macro_rules! impl_ipld_native {
    (   $doc_str:expr ;
        $native_ty:ty : $name:ident $ipld_type:ident {
            $deserialize_fn:ident
            $visit_fn:ident
            $visit_arg:ident : $visit_ty:ty
        }
    ) => {
        #[doc = $doc_str]
        pub type $name = $native_ty;

        impl Representation for $native_ty {
            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str =
                concat!("type ", stringify!($name), " ", stringify!($ipld_type));
            const KIND: Kind = Kind::$name;
        }

        // impl<C> Select<C, Self> for $native_ty
        // where
        //     C: Context,
        // {
        //     #[inline]
        //     fn select(seed: SelectorSeed, ctx: &mut C) -> Result<(), Error>
        //     {
        //         // primitive_select::<C, Self>(seed, ctx)
        //     }
        //
        //     #[inline]
        //     fn select_dag(seed: SelectorSeed, ctx: &mut C) -> Result<Self, Error> {
        //         primitive_select_dag::<C, Self>(seed, ctx)
        //     }
        // }

        impl_ipld! { @visitor {} {} $native_ty => $native_ty {
            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, $doc_str)
            }

            #[inline]
            fn $visit_fn<E>(self, $visit_arg : $visit_ty) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match self.selector {
                    Selector::Matcher(Matcher { label, .. }) => {
                        match self.state.mode() {
                            SelectorState::DAG_MATCH_MODE => Ok(Some($visit_arg)),
                            SelectorState::NODE_MODE => {
                                self.send_matched($visit_arg.into(), label.clone()).map_err(E::custom)?;
                                Ok(None)
                            },
                            SelectorState::DAG_MODE => {
                                self.send_dag($visit_arg, label.clone()).map_err(E::custom)?;
                                Ok(None)
                            },
                        }
                    },
                    selector => Err(Error::unsupported_selector::<$name, $name>(selector)).map_err(E::custom)
                }
            }
        }}

        impl_ipld! { @deseed {} {} $native_ty => $native_ty {
            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.$deserialize_fn(self)
            }
        }}

        impl_ipld! { @select_self {} {} $native_ty }
    };
}

impl_ipld_native! (
    "A boolean type" ;
    bool : Bool bool {
        deserialize_bool
        visit_bool
        v: bool
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an int8";
    i8 : Int8 int8 {
        deserialize_i8
        visit_i8
        v: i8
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an int16" ;
    i16 : Int16 int16 {
        deserialize_i16
        visit_i16
        v: i16
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an int32" ;
    i32 : Int int32 {
        deserialize_i32
        visit_i32
        v: i32
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an int64" ;
    i64 : Int64 int64 {
        deserialize_i64
        visit_i64
        v: i64
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an int128" ;
    i128 : Int128 int128 {
        deserialize_i128
        visit_i128
        v: i128
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an uint8" ;
    u8 : Uint8 uint8 {
        deserialize_u8
        visit_u8
        v: u8
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an uint16" ;
    u16 : Uint16 uint16 {
        deserialize_u16
        visit_u16
        v: u16
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an uint32" ;
    u32 : Uint32 uint32 {
        deserialize_u32
        visit_u32
        v: u32
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an uint64" ;
    u64 : Uint64 uint64 {
        deserialize_u64
        visit_u64
        v: u64
    }
);
impl_ipld_native! (
    "A fixed-length number type representing an uint128" ;
    u128 : Uint128 uint128 {
        deserialize_u128
        visit_u128
        v: u128
    }
);
impl_ipld_native! (
    "A fixed-length number type representing a float32" ;
    f32 : Float32 float32 {
        deserialize_f32
        visit_f32
        v: f32
    }
);
impl_ipld_native! (
    "A fixed-length number type representing a float64" ;
    f64 : Float float64 {
        deserialize_f64
        visit_f64
        v: f64
    }
);

mod string {
    use crate::dev::*;

    impl Representation for String {
        const NAME: &'static str = "String";
        const SCHEMA: &'static str = "type String string";
        const KIND: Kind = Kind::String;
    }

    impl_ipld! { @visitor {} {} String => String {
        #[inline]
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "A UTF-8 string")
        }

        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_string(s.into())
        }

        #[inline]
        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match self.selector {
                Selector::Matcher(Matcher { label, .. }) => {
                    match self.state.mode() {
                        SelectorState::DAG_MATCH_MODE => Ok(Some(s)),
                        SelectorState::NODE_MODE => {
                            self.send_matched(s.into(), label.clone()).map_err(E::custom)?;
                            Ok(None)
                        },
                        SelectorState::DAG_MODE => {
                            self.send_dag(s, label.clone()).map_err(E::custom)?;
                            Ok(None)
                        },
                    }
                },
                selector => Err(Error::unsupported_selector::<String, String>(selector)).map_err(E::custom)
            }
        }
    }}

    impl_ipld! { @deseed {} {} String => String {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_string(self)
        }
    }}

    impl_ipld! { @select_self {} {} String }

    // impl<'a> Representation for &'a str {
    //     const NAME: &'static str = "String";
    //     const SCHEMA: &'static str = "type String string";
    //     const KIND: Kind = Kind::String;
    // }
}

// /// Implements selection (matching) for primitive types.
// #[inline]
// pub fn primitive_select<'a, 'de, C, T, S>(
//     selector: &Selector,
//     state: SelectorState,
//     ctx: &mut C
// ) -> Result<Option<S>, Error>
// where
//     C: Context,
//     T: Representation,
//     S: Representation,
//     ContextSeed<'a, C, S, S>: DeserializeSeed<'de, Value = Option<S>>,
//     // Node: From<T>,
// {
//     // if type_eq::<T, S>() {
//     //     T::r#match(selector, state, ctx)
//     // } else {
//     //     Err(Error::invalid_type_selection::<T, S>())
//     // }

//     // const are_eq: bool = type_eq::<T, S>();
//     // static_assertions::const_assert!(type_eq::<T, S>());

//     // unimplemented!()
// }

// /// Implements patch selection for primitive types.
// #[inline]
// pub fn primitive_patch<C, T>(
//     self_: &mut T,
//     selector: &Selector,
//     _state: SelectorState,
//     dag: T,
//     _ctx: &mut C,
// ) -> Result<(), Error>
// where
//     T: Select<C>,
//     C: Context,
//     Node: From<T>,
// {
//     match selector {
//         Selector::Matcher(Matcher { .. }) => {
//             *self_ = dag;
//             Ok(())
//         }
//         _ => Err(Error::unsupported_selector::<T>(selector)),
//     }
// }

// fn primitive_select_base<T, C, F, R>(
//     seed: SelectorState,
//     ctx: &mut C,
//     on_matched: F,
// ) -> Result<R, Error>
// where
//     T: Select<C, T>,
//     C: Context,
//     F: Fn(&SelectorState, T, &Option<String>) -> Result<R, Error>,
//     Node: From<T>,
// {
//     let deserializer = ctx.path_decoder(seed.path())?;
//     let selector = seed.as_selector();

//     match selector {
//         Selector::Matcher(Matcher { ref label, .. }) => {
//             let inner = <T>::deserialize(deserializer)
//                 .map_err(|err| Error::Decoder(anyhow::anyhow!(err.to_string())))?;

//             on_matched(&seed, inner, label)
//         }
//         _ => Err(Error::UnsupportedSelector {
//             type_name: T::NAME,
//             selector_name: selector.name(),
//         }),
//     }
// }

// impl_root_select!(Matcher {
//     impl<Ctx, T> Select<Selector, Ctx> for Option<T>
//     where
//         Ctx: Context,
//         T: Representation + 'static
// });

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
