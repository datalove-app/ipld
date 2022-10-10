use crate::dev::*;
use macros::{
    derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator},
    impl_selector_seed_serde,
};
use maybestd::{borrow::Cow, fmt, str::FromStr};

pub use self::bool::Bool;
pub use self::bytes::Bytes;
pub use self::null::Null;
pub use self::num::*;
pub use self::string::IpldString;

/// Type alias for integers, which are represented as `i128`s.
pub type Int = Int64;
/// Type alias for floats, which are represented as `f64`s.
pub type Float = Float64;

mod null {
    use super::*;

    /// A nothing type.
    #[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq, Deserialize, Serialize)]
    pub struct Null;

    impl Representation for Null {
        type ReprKind = type_kinds::Null;

        const NAME: &'static str = "Null";
        const SCHEMA: &'static str = "type Null null";
        const DATA_MODEL_KIND: Kind = Kind::Null;

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_none()
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            <()>::deserialize(deserializer)?;
            Ok(Self)
        }
    }

    // impl_selector_seed_serde! { @codec_seed_visitor {} {} Null {
    //     #[inline]
    //     fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         write!(f, "{}", Null::NAME)
    //     }

    //     #[inline]
    //     fn visit_none<E>(self) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         self.0.select_primitive::<_C>(Null).map_err(E::custom)
    //     }

    //     #[inline]
    //     fn visit_unit<E>(self) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         self.visit_none()
    //     }
    // }}

    // impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} Null {} }

    impl_selector_seed_serde! { @codec_seed_visitor_rk Null T T
        { T: From<Null> + 'static } {  }
    {
        #[inline]
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "A nothing type {}", T::NAME)
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.0.select_primitive::<_C>(T::from(Null)).map_err(E::custom)
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_none()
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext_rk Null T T
        { T: From<Null> + 'static } {} {}
    }

    impl_selector_seed_serde! { @selector_seed_select {} {} Null }
}

mod bool {
    use super::*;

    /// A boolean type.
    pub type Bool = bool;

    impl Representation for bool {
        type ReprKind = type_kinds::Bool;

        const NAME: &'static str = "Bool";
        const SCHEMA: &'static str = "type Bool bool";
        const DATA_MODEL_KIND: Kind = Kind::Bool;

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Serialize::serialize(self, serializer)
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer)
        }
    }

    // impl_selector_seed_serde! { @codec_seed_visitor {} {} Bool {
    //     #[inline]
    //     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         write!(f, "{}, a boolean type", Bool::NAME)
    //     }

    //     #[inline]
    //     fn visit_bool<E>(self, v : bool) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         if self.0.selector.is_explore_union() {
    //             v.__select_in(self.0).map_err(E::custom)
    //         } else  {
    //             self.0.match_primitive::<_C>(v).map_err(E::custom)
    //         }
    //     }
    // }}

    // impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} Bool {} }

    impl_selector_seed_serde! { @codec_seed_visitor_rk Bool T T
        { T: From<Bool> + 'static } {  }
    {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "A boolean type {}", T::NAME)
        }

        #[inline]
        fn visit_bool<E>(self, v : bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if self.0.selector.is_explore_union() {
                T::from(v).__select_in(self.0).map_err(E::custom)
            } else  {
                self.0.match_primitive::<_C>(T::from(v)).map_err(E::custom)
            }
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext_rk Bool T T
        { T: From<Bool> + 'static } {} {}
    }

    impl_selector_seed_serde! { @selector_seed_select {} {} Bool }
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
            #[doc = concat!("a fixed-length number type represented as a(n)", stringify!($ty))]
            pub type $name = $ty;

            impl Representation for $ty {
                type ReprKind = type_kinds::$name;

                const NAME: &'static str = stringify!($name);
                const SCHEMA: &'static str = concat!("type ", stringify!($name), " int");
                const DATA_MODEL_KIND: Kind = Kind::$dm_kind;
                const SCHEMA_KIND: Kind = Kind::$name;
                // const REPR_KIND: Kind = Kind::$dm_kind;

                impl_ipld_num!(@field $dm_kind $ty);

                #[doc(hidden)]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    Serialize::serialize(self, serializer)
                }

                #[doc(hidden)]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    Deserialize::deserialize(deserializer)
                }
            }

            impl_selector_seed_serde! { @codec_seed_visitor_rk $name T T
                { T: From<$ty> + 'static } {  }
            {
                #[inline]
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f,
                        "{}, a fixed-length number type represented as a(n) {}",
                        <$ty>::NAME, stringify!($ty),
                    )
                }

                #[inline]
                fn $visit_fn<E>(self, v: $ty) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    self.0.select_primitive::<_C>(T::from(v)).map_err(E::custom)
                }

                $(
                    #[inline]
                    fn $other_visit_fn<E>(self, v: $other_ty) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let n = <$ty as Representation>::deserialize::<_C, _>(v.into_deserializer())?;
                        self.$visit_fn(n)
                    }
                )*
            }}

            impl_selector_seed_serde! { @codec_seed_visitor_ext_rk $name T T
                { T: From<$ty> + 'static } {} {}
            }

            impl_selector_seed_serde! { @selector_seed_select {} {} $ty }
        };
        (@field Int $ty:ty) => {
            fn as_field(&self) -> Option<Field<'_>> {
                usize::try_from(*self).ok().map(Field::Index)
            }
        };
        (@field Float $ty:ty) => {};
    }

    impl_ipld_num! (
        i8 : Int8 Int {
            // deserialize_i8
            visit_i8
            @conv {
                i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        i16 : Int16 Int {
            // deserialize_i16
            visit_i16
            @conv {
                i8:visit_i8 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        i32 : Int32 Int {
            // deserialize_i32
            visit_i32
            @conv {
                i8:visit_i8 i16:visit_i16 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        i64 : Int64 Int {
            // deserialize_i64
            visit_i64
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        i128 : Int128 Int {
            // deserialize_i128
            visit_i128
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        u8 : Uint8 Int {
            // deserialize_u8
            visit_u8
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        u16 : Uint16 Int {
            // deserialize_u16
            visit_u16
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        u32 : Uint32 Int {
            // deserialize_u32
            visit_u32
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        u64 : Uint64 Int {
            // deserialize_u64
            visit_u64
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        u128 : Uint128 Int {
            // deserialize_u128
            visit_u128
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64
            }
        }
    );
    impl_ipld_num! (
        f32 : Float32 Float {
            // deserialize_f32
            visit_f32
            @conv { f64:visit_f64 }
        }
    );
    impl_ipld_num! (
        f64 : Float64 Float {
            // deserialize_f64
            visit_f64
            @conv { f32:visit_f32 }
        }
    );
}

// TODO: unicode normalization? https://ipld.io/docs/data-model/kinds/#string-kind
mod string {
    use super::*;
    use unicode_normalization::UnicodeNormalization;

    ///
    #[derive(
        AsRef,
        AsMut,
        Clone,
        Debug,
        Default,
        Deref,
        DerefMut,
        Eq,
        Hash,
        Into,
        Index,
        IndexMut,
        // IntoIterator,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    #[as_ref(forward)]
    #[as_mut(forward)]
    #[deref(forward)]
    #[deref_mut(forward)]
    pub struct IpldString(String);

    impl IpldString {
        ///
        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl Representation for IpldString {
        type ReprKind = type_kinds::String;

        const NAME: &'static str = "String";
        const SCHEMA: &'static str = "type String string";
        const DATA_MODEL_KIND: Kind = Kind::String;

        fn as_field(&self) -> Option<Field<'_>> {
            Some(self.0.as_str().into())
        }

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Serialize::serialize(&self.0, serializer)
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self::from(<Cow<'_, str>>::deserialize(deserializer)?))
        }
    }

    impl_selector_seed_serde! { @codec_seed_visitor_rk String T T
        { T: From<IpldString> + 'static } {}
    {
        #[inline]
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "A string of type {}", T::NAME)
        }

        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.0.select_primitive::<_C>(T::from(IpldString::from(s))).map_err(E::custom)
        }

        #[inline]
        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(s.as_ref())
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext_rk String T T
        { T: From<IpldString> + 'static } {} {}
    }

    impl_selector_seed_serde! { @selector_seed_select {} {} IpldString }

    impl From<&str> for IpldString {
        #[inline]
        fn from(s: &str) -> Self {
            Self(s.nfc().collect::<String>())
        }
    }
    impl From<&mut str> for IpldString {
        fn from(s: &mut str) -> Self {
            Self::from(&*s)
        }
    }
    impl From<Box<str>> for IpldString {
        fn from(s: Box<str>) -> Self {
            Self::from(s.as_ref())
        }
    }
    impl<'a> From<Cow<'a, str>> for IpldString {
        fn from(s: Cow<'a, str>) -> Self {
            Self::from(s.as_ref())
        }
    }
    impl From<String> for IpldString {
        fn from(s: String) -> Self {
            Self::from(s.as_str())
        }
    }

    impl FromStr for IpldString {
        type Err = Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self::from(s))
        }
    }

    impl fmt::Display for IpldString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }
}

mod bytes {
    use super::*;

    /// A bytes type.
    pub type Bytes = crate::dev::bytes::Bytes;

    impl Representation for Bytes {
        type ReprKind = type_kinds::Bytes;

        const NAME: &'static str = "Bytes";
        const SCHEMA: &'static str = "type Bytes bytes";
        const DATA_MODEL_KIND: Kind = Kind::Bytes;

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[cfg(feature = "dag-json")]
            if C == DagJson::CODE {
                return DagJson::serialize_bytes(self.as_ref(), serializer);
            }

            Serialize::serialize(self.as_ref(), serializer)
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct BytesVisitor;
            impl<'de> Visitor<'de> for BytesVisitor {
                type Value = Bytes;
                #[inline]
                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(formatter, "A slice of bytes")
                }
                #[inline]
                fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Self::Value::copy_from_slice(bytes))
                }
                #[inline]
                fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Self::Value::from(bytes))
                }
            }

            #[cfg(feature = "dag-json")]
            if C == DagJson::CODE {
                return DagJson::deserialize_bytes(deserializer, BytesVisitor);
            }

            deserializer.deserialize_bytes(BytesVisitor)
        }
    }

    impl_selector_seed_serde! { @codec_seed_visitor_rk Bytes T T
        { T: From<Bytes> + 'static } {  }
    {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Bytes of type {}", T::NAME)
        }

        // #[inline]
        // fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
        // where
        //     E: de::Error,
        // {
        //     self.0.select_bytes::<_C>(Bytes::copy_from_slice(bytes)).map_err(E::custom)
        // }

        // #[inline]
        // fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
        // where
        //     E: de::Error,
        // {
        //     self.0.select_bytes::<_C>(Bytes::from(bytes)).map_err(E::custom)
        // }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext_rk Bytes T T
        { T: From<Bytes> + 'static } {} {}
    }

    impl_selector_seed_serde! { @selector_seed_select {} {} Bytes }

    impl<'a, Ctx> SelectorSeed<'a, Ctx, Bytes>
    where
        Ctx: Context,
        // impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
        // where
        //     Ctx: Context,
        //     T: Select<Ctx>,
    {
        ///
        #[inline]
        // TODO: should accept a slice of bytes, then do conversion
        pub fn select_bytes<const C: u64>(mut self, bytes: Bytes) -> Result<(), Error> {
            // if let Some(s) = self.selector.as_explore_union() {
            //     s.assert_matches_first::<Bytes>()?;
            //     bytes.__select_in(self)
            // } else {
            //     self.match_primitive::<C>(raw)
            // }

            if let Some(matcher) = self.selector.as_matcher() {
                let bytes = matcher
                    .subset
                    .as_ref()
                    .map(|slice| bytes.slice(slice.to_range()))
                    .unwrap_or(bytes);

                if self.is_node_select() {
                    self.select_node(bytes.into())?;
                } else if self.is_dag_select() {
                    self.select_dag(bytes)?;
                };

                return Ok(());
            }

            match self.selector {
                Selector::ExploreInterpretAs(_) => {
                    todo!("what reprs and ADLs are interpreted from byte nodes?")
                }
                selector => Err(Error::unsupported_selector::<Bytes>(&selector)),
            }
        }
    }
}

impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
where
    Ctx: Context,
    T: Representation + 'static,
    // TODO And<T::ReprKind, TypedScalar>: TypedScalar
{
    ///
    pub fn select_primitive<const C: u64>(self, raw: T) -> Result<(), Error>
    where
        T: Select<Ctx>,
    {
        if let Some(s) = self.selector.as_explore_union() {
            s.assert_matches_first::<T>()?;
            raw.__select_in(self)
        } else {
            self.match_primitive::<C>(raw)
        }
    }

    #[inline]
    fn match_primitive<'de, const C: u64>(mut self, dag: T) -> Result<(), Error> {
        self.selector.try_as_matcher()?;

        if self.is_node_select() {
            self.select_node(dag.to_selected_node())?;
        } else if self.is_dag_select() {
            self.select_dag(dag)?;
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn setup<T: Representation>(dag: &T) -> MemoryContext {
        const DEFAULT_MC: u64 = DagJson::CODE;
        const DEFAULT_MH: u64 = Multihash::SHA2_256;

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
