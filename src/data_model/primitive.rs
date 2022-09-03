use crate::dev::*;
use macros::derive_more::{AsRef, Deref, From, IntoIterator};

pub use self::null::*;

mod null {
    use super::*;

    /// A nothing type.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize)]
    pub struct Null;

    impl Representation for Null {
        const NAME: &'static str = "Null";
        const SCHEMA: &'static str = "type Null null";
        const DATA_MODEL_KIND: Kind = Kind::Null;
    }

    impl_ipld_serde! { @context_visitor {} {} Null {
        #[inline]
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "Null")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_primitive(Null)
        }
    }}

    impl_ipld_serde! { @context_deseed {} {} Null {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_unit(self)
        }
    }}

    impl_ipld_serde! { @context_select {} {} Null }
}

mod string {
    use super::*;

    impl Representation for String {
        const NAME: &'static str = "String";
        const SCHEMA: &'static str = "type String string";
        const DATA_MODEL_KIND: Kind = Kind::String;
    }

    impl_ipld_serde! { @context_visitor {} {} String {
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
            self.visit_primitive(s)
        }
    }}

    impl_ipld_serde! { @context_deseed {} {} String {
        #[inline]
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_string(self)
        }
    }}

    impl_ipld_serde! { @context_select {} {} String }

    // impl<'a> Representation for &'a str {
    //     const NAME: &'static str = "String";
    //     const SCHEMA: &'static str = "type String string";
    //     const KIND: Kind = Kind::String;
    // }
}

schema! {
    /// A `bytes` type.
    #[ipld_attr(internal)]
    #[derive(
        AsRef,
        Clone,
        Debug,
        Default,
        Deref,
        Eq,
        From,
        Hash,
        IntoIterator,
        Ord,
        PartialEq,
        PartialOrd
    )]
    #[as_ref(forward)]
    #[deref(forward)]
    #[from(forward)]
    pub type Bytes bytes;
}

///
pub type Int = Int32;

///
pub type Float = Float64;

/// Implements IPLD traits for native primitive types.
macro_rules! impl_ipld_native {
    (   $doc_str:expr ;
        $native_ty:ty : $name:ident $kind:ident $ipld_type:ident {
            $deserialize_fn:ident
            $visit_fn:ident
        }
    ) => {
        #[doc = $doc_str]
        pub type $name = $native_ty;

        impl Representation for $native_ty {
            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str =
                concat!("type ", stringify!($name), " ", stringify!($ipld_type));
            const DATA_MODEL_KIND: Kind = Kind::$kind;
        }

        impl_ipld_serde! { @context_visitor {} {} $native_ty {
            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $doc_str)
            }

            #[inline]
            fn $visit_fn<E>(self, v : $native_ty) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_primitive(v)
            }
        }}

        impl_ipld_serde! { @context_deseed {} {} $native_ty {
            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.$deserialize_fn(self)
            }
        }}

        impl_ipld_serde! { @context_select {} {} $native_ty }
    };

    (@null
        $doc_str:expr ;
        $native_ty:ty : $name:ident $ipld_type:ident {
            $deserialize_fn:ident
            $visit_fn:ident
            $visit_arg:ident : $visit_ty:ty
        }
    ) => {};
}

impl_ipld_native! (
    "A boolean type" ;
    bool : Bool Bool bool {
        deserialize_bool
        visit_bool
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a int8";
    i8 : Int8 Int int {
        deserialize_i8
        visit_i8
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a int16" ;
    i16 : Int16 Int int {
        deserialize_i16
        visit_i16
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a int32" ;
    i32 : Int32 Int int {
        deserialize_i32
        visit_i32
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a int64" ;
    i64 : Int64 Int int {
        deserialize_i64
        visit_i64
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a int128" ;
    i128 : Int128 Int int {
        deserialize_i128
        visit_i128
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a uint8" ;
    u8 : Uint8 Int int {
        deserialize_u8
        visit_u8
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a uint16" ;
    u16 : Uint16 Int int {
        deserialize_u16
        visit_u16
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a uint32" ;
    u32 : Uint32 Int int {
        deserialize_u32
        visit_u32
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a uint64" ;
    u64 : Uint64 Int int {
        deserialize_u64
        visit_u64
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a uint128" ;
    u128 : Uint128 Int int {
        deserialize_u128
        visit_u128
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a float32" ;
    f32 : Float32 Float float {
        deserialize_f32
        visit_f32
    }
);
impl_ipld_native! (
    "A fixed-length number type represented as a float64" ;
    f64 : Float64 Float float {
        deserialize_f64
        visit_f64
    }
);

impl<'a, C, T> ContextSeed<'a, C, T>
where
    C: Context,
    T: Representation + 'static,
{
    #[inline]
    fn visit_primitive<'de, E>(mut self, dag: T) -> Result<(), E>
    where
        T: Into<SelectedNode>,
        E: serde::de::Error,
    {
        // must check selector
        // depending on mode, do something with deserialized data
        // must be defined per type

        let matcher = self
            .selector
            .as_matcher()
            .expect("should know that this is a matcher");

        match self.mode() {
            SelectionMode::SelectNode => {
                self.select_matched_node(dag.into(), matcher.label.as_deref())
                    .map_err(E::custom)?;
            }
            SelectionMode::SelectDag => {
                self.select_matched_dag(dag, matcher.label.as_deref())
                    .map_err(E::custom)?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        match self.selector {
            Selector::Matcher(matcher) => {
                let bytes = matcher
                    .subset
                    .as_ref()
                    .map(|Slice { from, to }| &bytes[*from as usize..*to as usize])
                    .unwrap_or(bytes);

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
            selector => Err(Error::unsupported_selector::<Bytes>(selector)).map_err(E::custom),
        }
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
