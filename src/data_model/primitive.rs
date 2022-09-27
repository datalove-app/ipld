use crate::dev::*;
use macros::{
    derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator},
    impl_selector_seed_serde,
};
use std::{borrow::Cow, fmt, ops::RangeBounds, str::FromStr};

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
        const NAME: &'static str = "Null";
        const SCHEMA: &'static str = "type Null null";
        const DATA_MODEL_KIND: Kind = Kind::Null;

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // TODO: none or unit?
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

    impl_selector_seed_serde! { @codec_seed_visitor {} {} Null {
        #[inline]
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "Null")
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.match_primitive(Null)
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_none()
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} Null {} }

    impl_selector_seed_serde! { @selector_seed_codec_deseed {} {} Null {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_unit(self)
        }
    }}

    impl_selector_seed_serde! { @selector_seed_select {} {} Null }
}

mod bool {
    use super::*;

    /// A boolean type.
    pub type Bool = bool;

    impl Representation for bool {
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

    impl_selector_seed_serde! { @codec_seed_visitor {} {} Bool {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "A boolean type")
        }

        #[inline]
        fn visit_bool<E>(self, v : bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.match_primitive(v)
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} Bool {} }

    impl_selector_seed_serde! { @selector_seed_codec_deseed {} {} Bool {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bool(self)
        }
    }}

    impl_selector_seed_serde! { @selector_seed_select {} {} Bool }
}

mod num {
    use super::*;

    /// Implements IPLD traits for native number types.
    macro_rules! impl_ipld_num {
        (   $doc_str:expr ;
            $ty:ident : $name:ident $kind:ident $ipld_type:ident {
                $deserialize_fn:ident
                $visit_fn:ident
                @conv { $($other_ty:ty : $other_visit_fn:ident)* }
            }
        ) => {
            #[doc = $doc_str]
            pub type $name = $ty;

            impl Representation for $ty {
                const NAME: &'static str = stringify!($name);
                const SCHEMA: &'static str =
                    concat!("type ", stringify!($name), " ", stringify!($ipld_type));
                const DATA_MODEL_KIND: Kind = Kind::$kind;

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

            impl_selector_seed_serde! { @codec_seed_visitor {} {} $ty {
                #[inline]
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, $doc_str)
                }

                #[inline]
                fn $visit_fn<E>(self, v: $ty) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    self.match_primitive(v)
                }

                $(
                    #[inline]
                    fn $other_visit_fn<E>(self, v: $other_ty) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let n = <$ty as Deserialize<'_>>::deserialize(v.into_deserializer())?;
                        self.match_primitive(n)
                    }
                )*
            }}

            impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} $ty {} }

            impl_selector_seed_serde! { @selector_seed_codec_deseed {} {} $ty {
                #[inline]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.$deserialize_fn(self)
                }
            }}

            impl_selector_seed_serde! { @selector_seed_select {} {} $ty }
        };
    }

    impl_ipld_num! (
        "A fixed-length number type represented as a int8";
        i8 : Int8 Int int8 {
            deserialize_i8
            visit_i8
            @conv {
                i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a int16" ;
        i16 : Int16 Int int16 {
            deserialize_i16
            visit_i16
            @conv {
                i8:visit_i8 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a int32" ;
        i32 : Int32 Int int32 {
            deserialize_i32
            visit_i32
            @conv {
                i8:visit_i8 i16:visit_i16 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a int64" ;
        i64 : Int64 Int int64 {
            deserialize_i64
            visit_i64
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a int128" ;
        i128 : Int128 Int int128 {
            deserialize_i128
            visit_i128
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a uint8" ;
        u8 : Uint8 Int uint8 {
            deserialize_u8
            visit_u8
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u16:visit_u16 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a uint16" ;
        u16 : Uint16 Int uint16 {
            deserialize_u16
            visit_u16
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u32:visit_u32 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a uint32" ;
        u32 : Uint32 Int uint32 {
            deserialize_u32
            visit_u32
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u64:visit_u64 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a uint64" ;
        u64 : Uint64 Int uint64 {
            deserialize_u64
            visit_u64
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u128:visit_u128
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a uint128" ;
        u128 : Uint128 Int uint128 {
            deserialize_u128
            visit_u128
            @conv {
                i8:visit_i8 i16:visit_i16 i32:visit_i32 i64:visit_i64 i128:visit_i128
                u8:visit_u8 u16:visit_u16 u32:visit_u32 u64:visit_u64
            }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a float32" ;
        f32 : Float32 Float float32 {
            deserialize_f32
            visit_f32
            @conv { f64:visit_f64 }
        }
    );
    impl_ipld_num! (
        "A fixed-length number type represented as a float64" ;
        f64 : Float64 Float float64 {
            deserialize_f64
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
        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl Representation for IpldString {
        const NAME: &'static str = "String";
        const SCHEMA: &'static str = "type String string";
        const DATA_MODEL_KIND: Kind = Kind::String;

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

    // TODO:
    impl_selector_seed_serde! { @codec_seed_visitor {} {} IpldString {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "An IPLD string")
        }

        #[inline]
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if Self::is_select() {
                self.match_primitive(IpldString::from(s))
            } else {
                unimplemented!()
            }
        }

        // TODO:
        #[inline]
        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if Self::is_select() {
                self.match_primitive(IpldString::from(s))
            } else {
                unimplemented!()
            }
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} IpldString {} }

    impl_selector_seed_serde! { @selector_seed_codec_deseed {} {} IpldString {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_string(self)
        }
    }}

    impl_selector_seed_serde! { @selector_seed_select {} {} IpldString }

    impl<'a> From<&'a str> for IpldString {
        #[inline]
        fn from(s: &'a str) -> Self {
            Self(s.nfc().collect::<String>())
        }
    }
    impl<'a> From<&'a mut str> for IpldString {
        fn from(s: &'a mut str) -> Self {
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
            write!(f, "{}", self.as_str())
        }
    }
}

mod bytes {
    use super::*;
    use crate::dev::bytes::Bytes as InnerBytes;

    /// A `bytes` type, which thinly wraps [`bytes::Bytes`].
    ///
    /// [`Bytes`]: bytes::Bytes
    #[derive(
        AsRef,
        AsMut,
        Clone,
        Debug,
        Default,
        Deref,
        Eq,
        From,
        Hash,
        Index,
        IndexMut,
        IntoIterator,
        Ord,
        PartialOrd,
        PartialEq,
    )]
    #[as_ref(forward)]
    #[as_mut(forward)]
    #[deref(forward)]
    #[from(forward)]
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
        pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
            Self(self.0.slice(range))
        }

        ///
        pub fn clear(&mut self) {
            self.0.clear()
        }
    }

    impl Representation for Bytes {
        const NAME: &'static str = "Bytes";
        const SCHEMA: &'static str = "type Bytes bytes";
        const DATA_MODEL_KIND: Kind = Kind::Bytes;

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "dag-json")] {
                    if C == DagJson::CODE {
                        return DagJson::serialize_bytes(self.as_ref(), serializer);
                    }
                }
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
                    Ok(Self::Value::from(bytes.into_boxed_slice()))
                }
            }

            cfg_if::cfg_if! {
                if #[cfg(feature = "dag-json")] {
                    if C == DagJson::CODE {
                        return DagJson::deserialize_bytes(deserializer, BytesVisitor);
                    }
                }
            }

            Ok(Self::from(Box::<[u8]>::deserialize(deserializer)?))
        }
    }

    impl_selector_seed_serde! { @codec_seed_visitor {} {} Bytes {
        #[inline]
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "A slice of bytes")
        }

        #[inline]
        fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if Self::is_select() {
                self.select_bytes(bytes)
            } else {
                // Ok(Self::Value::from(bytes))
                unimplemented!()
            }
        }

        #[inline]
        fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if Self::is_select() {
                self.select_bytes(&bytes)
            } else {
                // Ok(Self::Value::from(bytes))
                unimplemented!()
            }
        }
    }}

    impl_selector_seed_serde! { @codec_seed_visitor_ext {} {} Bytes {} }

    impl_selector_seed_serde! { @selector_seed_codec_deseed {} {} Bytes {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "dag-json")] {
                    if _C == DagJson::CODE {
                        return DagJson::deserialize_bytes(deserializer, self);
                    }
                }
            }

            deserializer.deserialize_bytes(self)
        }
    }}

    impl_selector_seed_serde! { @selector_seed_select {} {} Bytes }

    impl<'a, const C: u64, const D: bool, Ctx> CodedSelectorSeed<'a, C, D, Ctx, Bytes>
    where
        Ctx: Context,
    {
        #[inline]
        fn select_bytes<E>(mut self, bytes: &[u8]) -> Result<(), E>
        where
            E: de::Error,
        {
            match self.0.selector {
                Selector::Matcher(matcher) => {
                    let bytes = matcher
                        .subset
                        .as_ref()
                        .map(|Slice { from, to }| &bytes[*from as usize..*to as usize])
                        .unwrap_or(bytes);

                    match self.0.mode() {
                        SelectionMode::SelectNode => {
                            self.0
                                .select_matched_node(bytes.into(), matcher.label.as_deref())
                                .map_err(E::custom)?;
                        }
                        SelectionMode::SelectDag => {
                            self.0
                                .select_matched_dag(
                                    Bytes::copy_from_slice(bytes),
                                    matcher.label.as_deref(),
                                )
                                .map_err(E::custom)?;
                        }
                        _ => unimplemented!(),
                    }

                    // self.nodes.push_back(NodeRow::Match {
                    //     repr: Adl::Bytes,
                    //     path: self.path.clone(),
                    //     value: Value::Blob(bytes.into()),
                    // });

                    todo!();

                    Ok(())
                }
                Selector::ExploreInterpretAs(inner) => {
                    todo!("what reprs and ADLs are interpreted from byte nodes?")
                }
                selector => Err(Error::unsupported_selector::<Bytes>(&selector)).map_err(E::custom),
            }
        }
    }

    // impl Serialize for Bytes {
    //     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //     where
    //         S: Serializer,
    //     {
    //         serializer.serialize_bytes(self.as_ref())
    //     }
    // }

    // struct BytesVisitor;
    // // impl<'de> IpldVisitorExt<'de> for BytesVisitor {}
    // impl<'de> Visitor<'de> for BytesVisitor {
    //     type Value = Bytes;
    //     #[inline]
    //     fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         write!(formatter, "A slice of bytes")
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
    //         Ok(Self::Value::from(bytes.into_boxed_slice()))
    //     }
    // }

    // impl<'de> Deserialize<'de> for Bytes {
    //     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: Deserializer<'de>,
    //     {
    //         deserializer.deserialize_bytes(BytesVisitor)
    //     }
    // }
}

impl<'a, const C: u64, const D: bool, Ctx, T> CodedSelectorSeed<'a, C, D, Ctx, T>
where
    Ctx: Context,
    T: Representation + 'static,
{
    #[inline]
    fn match_primitive<'de, E>(mut self, dag: T) -> Result<(), E>
    where
        T: Into<SelectedNode>,
        E: de::Error,
    {
        let matcher = self
            .0
            .selector
            .as_matcher()
            .expect("should know that this is a matcher");

        match self.0.mode() {
            SelectionMode::SelectNode => {
                self.0
                    .select_matched_node(dag.into(), matcher.label.as_deref())
                    .map_err(E::custom)?;
            }
            SelectionMode::SelectDag => {
                self.0
                    .select_matched_dag(dag, matcher.label.as_deref())
                    .map_err(E::custom)?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
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
//
//     // const are_eq: bool = type_eq::<T, S>();
//     // static_assertions::const_assert!(type_eq::<T, S>());
//
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
//
//     match selector {
//         Selector::Matcher(Matcher { ref label, .. }) => {
//             let inner = <T>::deserialize(deserializer)
//                 .map_err(|err| Error::Decoder(anyhow::anyhow!(err.to_string())))?;
//
//             on_matched(&seed, inner, label)
//         }
//         _ => Err(Error::UnsupportedSelector {
//             type_name: T::NAME,
//             selector_name: selector.name(),
//         }),
//     }
// }

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
