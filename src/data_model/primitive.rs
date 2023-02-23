use crate::dev::*;
use macros::{
    derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator},
    repr_serde,
};
use maybestd::{borrow::Cow, fmt, ops, str::FromStr};

pub use self::bool::Bool;
pub use self::bytes::Bytes;
pub use self::null::Null;
pub use self::num::*;
// pub use self::string::IpldString;

/// Default type for IPLD integers, which aliases to [`i64`].
pub type Int = i64;

/// Default type for IPLD floats, which aliases to [`f64`].
pub type Float = f64;

mod null {
    use super::*;

    /// A nothing type.
    #[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialOrd, PartialEq, Representation)]
    #[ipld(internal)]
    pub struct Null;

    impl<'a, 'de, const MC: u64, Ctx> Visitor<'de> for AstWalk<'a, MC, Ctx, Null>
    where
        'a: 'de,
        Ctx: Context,
    {
        type Value = AstResult<Null>;

        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "A nothing type `{}`", <Null>::NAME)
        }
        #[inline]
        fn visit_none<Er>(self) -> Result<Self::Value, Er>
        where
            Er: de::Error,
        {
            self.into_inner()
                .select_scalar::<MC>(Null)
                .map_err(Er::custom)
        }
        #[inline]
        fn visit_unit<Er>(self) -> Result<Self::Value, Er>
        where
            Er: de::Error,
        {
            self.visit_none()
        }
    }
    impl<'a, 'de, const MC: u64, Ctx> LinkVisitor<'de, MC> for AstWalk<'a, MC, Ctx, Null>
    where
        'a: 'de,
        Ctx: Context,
    {
    }
    impl<Ctx: Context> Select<Ctx> for Null {
        type Walker<'a, const MC: u64> = AstWalk<'a, MC, Ctx, Self> where Ctx: 'a;
    }

    impl From<()> for Null {
        #[inline]
        fn from(_: ()) -> Self {
            Self
        }
    }
}

mod bool {
    use super::*;

    /// Type alias for [`bool`].
    ///
    /// [`bool`]: crate::maybestd::primitive::bool
    pub type Bool = bool;

    impl Representation for bool {
        const NAME: &'static str = "Bool";
        const SCHEMA: &'static str = "type Bool bool";
        const DATA_MODEL_KIND: Kind = Kind::Bool;

        fn to_selected_node(&self) -> SelectedNode {
            SelectedNode::Bool(*self)
        }

        #[doc(hidden)]
        #[inline]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Serialize::serialize(self, serializer)
        }

        #[doc(hidden)]
        #[inline]
        fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let res = AstWalk::<'de, MC, (), Self>::default().deserialize(deserializer)?;
            Ok(res.unwrap_val())
        }
    }

    repr_serde! { @select for bool }
    repr_serde! { @visitors for bool {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("A boolean type")
        }

        #[inline]
        fn visit_bool<Er>(self, v : bool) -> Result<Self::Value, Er>
        where
            Er: de::Error,
        {
            self.into_inner().select_scalar::<MC>(v).map_err(Er::custom)
        }
    }}
}

mod num {
    use super::*;

    /// Implements IPLD traits for native number types.
    macro_rules! impl_ipld_num {
        (   $ty:ident : $name:ident $dm_kind:ident {
                // $deserialize_fn:ident
                $visit_fn:ident
                @conv { $($other_ty:ty : $other_visit_fn:ident)* }
            }
        ) => {
            #[doc = concat!("Type alias for [`", stringify!($ty), "`]s.")]
            ///
            #[doc = concat!("[`", stringify!($ty), "`]: ")]
            #[doc = concat!("crate::maybestd::primitive::", stringify!($ty))]
            pub type $name = $ty;

            impl Representation for $ty {
                const NAME: &'static str = stringify!($name);
                const SCHEMA: &'static str = concat!("type ", stringify!($name), " int");
                const DATA_MODEL_KIND: Kind = Kind::$dm_kind;
                const SCHEMA_KIND: Kind = Kind::$name;
                const REPR_KIND: Kind = Kind::$name;

                impl_ipld_num!(@field $dm_kind $ty);

                fn to_selected_node(&self) -> SelectedNode {
                    SelectedNode::$name(*self)
                }

                #[doc(hidden)]
                #[inline]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    Serialize::serialize(self, serializer)
                }

                #[doc(hidden)]
                #[inline]
                fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    // Deserialize::deserialize(deserializer)
                    let res = AstWalk::<'de, MC, (), Self>::default().deserialize(deserializer)?;
                    Ok(res.unwrap_val())
                }
            }

            repr_serde! { @select for $ty }
            repr_serde! { @visitors for $ty {
                #[inline]
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f,
                        "`{}`, a fixed-length number type represented as a(n) {}",
                        stringify!($name), stringify!($ty),
                    )
                }
                #[inline]
                fn $visit_fn<Er>(self, v: $ty) -> Result<Self::Value, Er>
                where
                    Er: de::Error,
                {
                    // if self.mode() == SelectionMode::Decode {
                    //     return Ok(AstResult::Value(v));
                    // }

                    self.into_inner().select_scalar::<MC>(v).map_err(Er::custom)
                }

                $(  #[inline]
                    fn $other_visit_fn<Er>(self, v: $other_ty) -> Result<Self::Value, Er>
                    where
                        Er: de::Error,
                    {
                        let de = v.into_deserializer();
                        let n: $ty = Representation::deserialize::<MC, _>(de)?;
                        self.$visit_fn(n)
                    }
                )*
            }}
        };
        (@field Int $ty:ty) => {
            fn as_field(&self) -> Option<Field<'_>> {
                usize::try_from(*self).ok().map(Field::Index)
            }
        };
        (@field Float $ty:ty) => {};
    }

    impl_ipld_num! (i8 : Int8 Int { visit_i8 @conv {
        i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (i16 : Int16 Int { visit_i16 @conv {
        i8:visit_i8 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (i32 : Int32 Int { visit_i32 @conv {
        i8:visit_i8 i16:visit_i16 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (i64 : Int64 Int { visit_i64 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (i128 : Int128 Int { visit_i128 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (u8 : Uint8 Int { visit_u8 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (u16 : Uint16 Int { visit_u16 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u32:visit_u32 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (u32 : Uint32 Int { visit_u32 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u64:visit_u64 u128:visit_u128
    }});
    impl_ipld_num! (u64 : Uint64 Int { visit_u64 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u128:visit_u128
    }});
    impl_ipld_num! (u128 : Uint128 Int { visit_u128 @conv {
        i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
        u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64
    }});
    impl_ipld_num! (f32 : Float32 Float { visit_f32 @conv { f64:visit_f64 } });
    impl_ipld_num! (f64 : Float64 Float { visit_f64 @conv { f32:visit_f32 } });
}

// TODO: unicode normalization? https://ipld.io/docs/data-model/kinds/#string-kind
mod string {
    use super::*;
    use unicode_normalization::*;

    // ? Cow<'a, str>
    // struct NfcChars<'a>(Recompositions<Chars<'a>>);
    // impl<'a> ToOwned for <'a> {
    //     type Owned = String;
    //     fn to_owned(&self) -> Self::Owned {
    //
    //     }
    // }
    //
    // ///
    // #[derive(
    //     AsRef,
    //     AsMut,
    //     Clone,
    //     Debug,
    //     Default,
    //     Deref,
    //     DerefMut,
    //     Eq,
    //     Hash,
    //     Into,
    //     Index,
    //     IndexMut,
    //     // IntoIterator,
    //     Ord,
    //     PartialEq,
    //     PartialOrd,
    // )]
    // #[as_ref(forward)]
    // #[as_mut(forward)]
    // #[deref(forward)]
    // #[deref_mut(forward)]
    // pub struct IpldString(String);
    //
    // impl IpldString {
    //     ///
    //     pub fn as_str(&self) -> &str {
    //         self.0.as_str()
    //     }
    // }

    impl Representation for String {
        const NAME: &'static str = "String";
        const SCHEMA: &'static str = "type String string";
        const DATA_MODEL_KIND: Kind = Kind::String;

        fn as_field(&self) -> Option<Field<'_>> {
            Some(self.as_str().into())
        }

        fn to_selected_node(&self) -> SelectedNode {
            SelectedNode::String(self.nfc().collect())
        }

        #[doc(hidden)]
        #[inline]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_str(&self.as_str().nfc())
        }

        #[doc(hidden)]
        #[inline]
        fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // let s = <Cow<'_, str>>::deserialize(deserializer)?;
            // Ok(s.nfc().collect())
            let res = AstWalk::<'de, MC, (), Self>::default().deserialize(deserializer)?;
            Ok(res.unwrap_val())
        }
    }

    repr_serde! { @select for String }
    repr_serde! { @visitors for String {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("A string of UTF-8 characters")
        }

        #[inline]
        fn visit_str<Er>(self, s: &str) -> Result<Self::Value, Er>
        where
            Er: de::Error,
        {

            // if self.mode() == SelectionMode::Decode {
            //     return Ok(AstResult::Value(s.nfc().collect()));
            // }

            self.into_inner()
                .select_scalar::<MC>(s.nfc().collect())
                .map_err(Er::custom)
        }

        #[inline]
        fn visit_string<Er>(self, s: String) -> Result<Self::Value, Er>
        where
            Er: de::Error,
        {
            if self.mode() == SelectionMode::Decode {
                return Ok(AstResult::Value(s));
            }

            self.visit_str(s.as_ref())
        }
    }}

    // impl From<&str> for IpldString {
    //     #[inline]
    //     fn from(s: &str) -> Self {
    //         Self(s.nfc().collect::<String>())
    //     }
    // }
    // impl From<&mut str> for IpldString {
    //     fn from(s: &mut str) -> Self {
    //         Self::from(&*s)
    //     }
    // }
    // impl From<Box<str>> for IpldString {
    //     fn from(s: Box<str>) -> Self {
    //         Self::from(s.as_ref())
    //     }
    // }
    // impl<'a> From<Cow<'a, str>> for IpldString {
    //     fn from(s: Cow<'a, str>) -> Self {
    //         Self::from(s.as_ref())
    //     }
    // }
    // impl From<String> for IpldString {
    //     fn from(s: String) -> Self {
    //         Self::from(s.as_str())
    //     }
    // }
    //
    // impl FromStr for IpldString {
    //     type Err = Error;
    //     fn from_str(s: &str) -> Result<Self, Self::Err> {
    //         Ok(Self::from(s))
    //     }
    // }
    //
    // impl fmt::Display for IpldString {
    //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         self.0.fmt(f)
    //     }
    // }
}

mod bytes {
    use super::*;
    use crate::dev::bytes::Bytes as InnerBytes;

    /// A `bytes` type, which thinly wraps [`bytes::Bytes`].
    ///
    /// TODO: mutability
    /// [`bytes::Bytes`]: bytes::Bytes
    #[derive(
        AsRef,
        // AsMut,
        Clone,
        Debug,
        Default,
        Deref,
        Eq,
        From,
        Hash,
        // Index,
        Into,
        IntoIterator,
        Ord,
        PartialOrd,
        PartialEq,
    )]
    #[as_ref(forward)]
    #[deref(forward)]
    pub struct Bytes(InnerBytes);

    impl Bytes {
        ///
        pub const fn new() -> Self {
            Self(InnerBytes::new())
        }

        ///
        pub fn copy_from_slice(bytes: &[u8]) -> Self {
            Self(InnerBytes::copy_from_slice(bytes))
        }

        ///
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        ///
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        ///
        pub fn slice(&self, range: impl ops::RangeBounds<usize>) -> Self {
            Self(self.0.slice(range))
        }

        ///
        pub fn clear(&mut self) {
            self.0.clear()
        }
    }

    impl From<&[u8]> for Bytes {
        fn from(bytes: &[u8]) -> Self {
            Self::copy_from_slice(bytes)
        }
    }
    impl From<Vec<u8>> for Bytes {
        fn from(bytes: Vec<u8>) -> Self {
            Self(InnerBytes::from(bytes))
        }
    }

    // impl<R: ops::RangeBounds<usize>> ops::Index<R> for Bytes {
    //     type Output = InnerBytes;
    //     fn index(&self, index: R) -> &Self::Output {
    //         self.0.slice(index)
    //     }
    // }

    impl Representation for Bytes {
        const NAME: &'static str = "Bytes";
        const SCHEMA: &'static str = "type Bytes bytes";
        const DATA_MODEL_KIND: Kind = Kind::Bytes;

        fn to_selected_node(&self) -> SelectedNode {
            SelectedNode::Bytes(self.clone())
        }

        #[doc(hidden)]
        fn serialize<const MC: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Multicodec::serialize_bytes::<MC, S>(self.as_ref(), serializer)
        }

        #[doc(hidden)]
        fn deserialize<'de, const MC: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // struct BytesVisitor;
            // impl<'de> Visitor<'de> for BytesVisitor {
            //     type Value = Bytes;
            //     #[inline]
            //     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            //         write!(f, "A slice of bytes of type {}", Self::Value::NAME)
            //     }
            //     #[inline]
            //     fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            //     where
            //         E: de::Error,
            //     {
            //         Ok(Self::Value::copy_from_slice(bytes))
            //     }
            //     #[inline]
            //     fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
            //     where
            //         E: de::Error,
            //     {
            //         Ok(Self::Value::from(bytes))
            //     }
            // }
            // impl<'de, const MC: u64> LinkVisitor<'de, MC> for BytesVisitor {}

            // Multicodec::deserialize_bytes::<MC, D, _>(deserializer, BytesVisitor)

            let res = Multicodec::deserialize_bytes(
                deserializer,
                AstWalk::<'de, MC, (), Self>::default(),
            )?;
            Ok(res.unwrap_val())
        }
    }

    repr_serde! { @select for Bytes }
    repr_serde! { @visitors for Bytes {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "bytes of type `{}`", <Bytes>::NAME)
        }
        #[inline]
        fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if self.mode() == SelectionMode::Decode {
                return Ok(AstResult::Value(Bytes::copy_from_slice(bytes)));
            }

            // let bytes = T::try_from(bytes).map_err(E::custom)?;
            // self.0.select_bytes::<C>(bytes).map_err(E::custom)
            unimplemented!()
        }
        #[inline]
        fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // let bytes = T::try_from(bytes).map_err(E::custom)?;
            // self.0.select_bytes::<C>(bytes).map_err(E::custom)
            unimplemented!()
        }
    }}

    // impl<const N: usize> Representation for [u8; N] {
    //     const NAME: &'static str = "Bytes";
    //     const SCHEMA: &'static str = "type Bytes bytes";
    //     const DATA_MODEL_KIND: Kind = Kind::Bytes;
    //
    //     fn to_selected_node(&self) -> SelectedNode {
    //         SelectedNode::Bytes(self.clone())
    //     }
    //
    //     #[doc(hidden)]
    //     fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //     where
    //         S: Serializer,
    //     {
    //         Multicodec::serialize_bytes::<C, S>(self.as_ref(), serializer)
    //     }
    //
    //     #[doc(hidden)]
    //     fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: Deserializer<'de>,
    //     {
    //         struct BytesVisitor;
    //         impl<'de> Visitor<'de> for BytesVisitor {
    //             type Value = Bytes;
    //             #[inline]
    //             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //                 write!(f, "A slice of bytes of type {}", Self::Value::NAME)
    //             }
    //             #[inline]
    //             fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    //             where
    //                 E: de::Error,
    //             {
    //                 Ok(Self::Value::copy_from_slice(bytes))
    //             }
    //             #[inline]
    //             fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
    //             where
    //                 E: de::Error,
    //             {
    //                 Ok(Self::Value::from(bytes))
    //             }
    //         }
    //         impl<'de> LinkVisitor<'de> for BytesVisitor {}
    //
    //         Multicodec::deserialize_bytes::<C, D, _>(deserializer, BytesVisitor)
    //     }
    // }
    //
    // repr_serde! { @select for Bytes }
    // repr_serde! { @visitors for Bytes {
    //     #[inline]
    //     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         write!(f, "bytes of type `{}`", <Bytes>::NAME)
    //     }
    //     #[inline]
    //     fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         // let bytes = T::try_from(bytes).map_err(E::custom)?;
    //         // self.0.select_bytes::<C>(bytes).map_err(E::custom)
    //         unimplemented!()
    //     }
    //     #[inline]
    //     fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         // let bytes = T::try_from(bytes).map_err(E::custom)?;
    //         // self.0.select_bytes::<C>(bytes).map_err(E::custom)
    //         unimplemented!()
    //     }
    // }}

    // TODO: be generic over T
    // impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
    // where
    //     Ctx: Context,
    //     T: Select<Ctx>,
    impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
    where
        Ctx: Context,
        T: Representation,
    {
        ///
        #[inline]
        // TODO: should accept a slice of bytes, then do conversion
        pub fn select_bytes<'de, const MC: u64>(
            mut self,
            bytes: &[u8],
        ) -> Result<AstResult<Bytes>, Error>
        where
            T: TryFrom<&'de [u8]>,
        {
            unimplemented!()
        }

        ///
        #[inline]
        // TODO: should accept a slice of bytes, then do conversion
        pub fn select_byte_buf<'de, const MC: u64>(
            mut self,
            bytes: Vec<u8>,
        ) -> Result<AstResult<Bytes>, Error>
        where
            T: TryFrom<Vec<u8>>,
        {
            // if let Some(s) = self.selector.as_explore_union() {
            //     s.assert_matches_first::<Bytes>()?;
            //     bytes.__select_in(self)
            // } else {
            //     self.match_scalar::<MC>(raw)
            // }

            if let Some(matcher) = self.selector.as_matcher() {
                // let bytes = matcher
                //     .subset
                //     .as_ref()
                //     .map(|slice| bytes.slice(slice.to_range()))
                //     .unwrap_or(bytes);

                // if self.is_select_node() {
                //     self.handle_node(bytes.into())?;
                // } else if self.is_select_dag() {
                //     self.handle_dag(bytes)?;
                // };
                // return Ok(());

                // return self.cover_dag(bytes);
            }

            match self.selector {
                Selector::ExploreInterpretAs(_) => {
                    todo!("what reprs and ADLs are interpreted from bytes?")
                }
                selector => Err(Error::unsupported_selector::<Bytes>(&selector)),
            }
        }
    }
}

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Select<Ctx> + 'static,
    // TODO And<T::ReprKind, TypedScalar>: TypedScalar
{
    ///
    #[inline]
    pub fn select_scalar<const MC: u64>(mut self, raw: T) -> Result<AstResult<T>, Error> {
        match self.selector {
            Selector::Matcher(_) => self.select_dag(raw),
            Selector::ExploreUnion(s) => {
                // todo: split into params + ctx, T::select for each
                // raw.__select(self)

                Ok(AstResult::Ok)
            }
            s => Err(Error::unsupported_selector::<T>(s)),
        }
    }

    #[inline]
    pub fn patch_scalar<const MC: u64>(mut self, raw: &mut T) -> Result<AstResult<T>, Error> {
        self.selector.try_as_matcher()?;

        // todo
        // self.

        Ok(AstResult::Ok)
    }

    // #[inline]
    // pub fn match_scalar<'de, const MC: u64>(mut self, dag: T) -> Result<(), Error> {
    //     self.selector.try_as_matcher()?;

    //     if self.is_node_select() {
    //         self.handle_node(dag.to_selected_node())?;
    //     } else if self.is_dag_select() {
    //         self.handle_dag(dag)?;
    //     };

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn setup<T: Representation>(dag: &T) -> MemoryContext {
        const DEFAULT_MC: u64 = Multicodec::DAG_JSON;
        const DEFAULT_MH: u64 = Multihasher::SHA2_256;

        let mut ctx = MemoryContext::default();
        ctx.add_block(Version::V1, DEFAULT_MC, DEFAULT_MH, vec![])
            .unwrap();
        ctx
    }

    #[test]
    fn test_mull_match() {
        let mut ctx = setup(&Null);
    }
}
