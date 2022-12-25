//!
//! todo:
//!     duration, systemtime, paths, atomics, nonzero
//!     (ip/socket)addrs, (c/os)str

use crate::dev::*;
use macros::{
    derive_more::{From, Into},
    repr_serde,
};
use maybestd::{fmt, marker::PhantomData, num::Wrapping, rc::Rc, sync::Arc};

mod ignored {
    use super::*;

    impl Representation for IgnoredAny {
        const NAME: &'static str = "IgnoredAny";
        const SCHEMA: &'static str = "type IgnoredAny = Any";
        const DATA_MODEL_KIND: Kind = Kind::Null;
        const SCHEMA_KIND: Kind = Kind::Union;
        const REPR_KIND: Kind = Kind::Any;
        const REPR_STRATEGY: Strategy = Strategy::Ignored;

        fn to_selected_node(&self) -> SelectedNode {
            unreachable!()
        }

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            unreachable!()
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer)
        }
    }

    repr_serde! { @select for IgnoredAny }
    repr_serde! { @visitors for IgnoredAny {
        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", IgnoredAny::NAME)
        }
        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(AstResult::Ok)
        }
    }}

    impl<T: Representation> Representation for PhantomData<T> {
        const NAME: &'static str = "Phantom";
        const SCHEMA: &'static str = "type Phantom = Any";
        const DATA_MODEL_KIND: Kind = Kind::Null;
        const SCHEMA_KIND: Kind = Kind::Union;
        const REPR_KIND: Kind = Kind::Any;
        const REPR_STRATEGY: Strategy = Strategy::Ignored;

        fn to_selected_node(&self) -> SelectedNode {
            unreachable!()
        }

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // FIXME: call noop somehow
            unimplemented!()
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(_: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(PhantomData)
        }
    }

    repr_serde! { @select for PhantomData<T> { T } { T: Representation }}
    repr_serde! { @visitors for PhantomData<T> { T } { T: Representation + '__a } @serde {
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", T::NAME)
            }
        }
    }
}

mod implicit {
    use super::*;

    /// A type whose absence denotes the presence of the inner type's
    /// [`Default`] value.
    #[derive(Copy, Clone, Debug, Default, From)]
    pub struct Implicit<T: Default>(T);

    impl<T> Representation for Implicit<T>
    where
        T: Default + Representation,
    {
        const NAME: &'static str = "Implicit";
        const SCHEMA: &'static str = concat!("type ", stringify!(T::NAME), " implicit");
        const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
        const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
        const REPR_KIND: Kind = T::REPR_KIND;
        const REPR_STRATEGY: Strategy = T::REPR_STRATEGY;
        const HAS_LINKS: bool = T::HAS_LINKS;

        fn as_field(&self) -> Option<Field<'_>> {
            self.0.as_field()
        }

        fn to_selected_node(&self) -> SelectedNode {
            self.0.to_selected_node()
        }

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Representation::serialize::<C, S>(&self.0, serializer)
        }

        // TODO
        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // struct OptionVisitor<const C: u64, T>(PhantomData<T>);
            // impl<'de, const C: u64, T: Representation> Visitor<'de> for OptionVisitor<C, T> {
            //     type Value = Option<T>;
            //     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            //         write!(f, "A nullable `{}`", T::NAME)
            //     }
            //     #[inline]
            //     fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            //         Ok(None)
            //     }
            //     #[inline]
            //     fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            //         Ok(None)
            //     }
            //     #[inline]
            //     fn visit_some<D: Deserializer<'de>>(
            //         self,
            //         deserializer: D,
            //     ) -> Result<Self::Value, D::Error> {
            //         T::deserialize::<C, _>(deserializer).map(Some)
            //     }
            //     fn __private_visit_untagged_option<D>(
            //         self,
            //         deserializer: D,
            //     ) -> Result<Self::Value, ()>
            //     where
            //         D: Deserializer<'de>,
            //     {
            //         Ok(T::deserialize::<C, _>(deserializer).ok())
            //     }
            // }

            // deserializer.deserialize_option(OptionVisitor::<C, T>(PhantomData))

            unimplemented!()
        }
    }
}

// TODO: optional vs nullable?
mod option {
    use super::*;

    impl<T: Representation> Representation for Option<T> {
        const NAME: &'static str = "Nullable";
        const SCHEMA: &'static str = concat!("type ", stringify!(T::NAME), " nullable");
        const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND.union(Kind::Null);
        const SCHEMA_KIND: Kind = T::SCHEMA_KIND.union(Kind::Null);
        const REPR_KIND: Kind = T::REPR_KIND.union(Kind::Null);
        const REPR_STRATEGY: Strategy = T::REPR_STRATEGY;
        const HAS_LINKS: bool = T::HAS_LINKS;

        fn name(&self) -> &'static str {
            match self {
                Self::None => Null::NAME,
                Self::Some(t) => t.name(),
            }
        }

        fn has_links(&self) -> bool {
            match self {
                Self::None => false,
                Self::Some(t) => t.has_links(),
            }
        }

        fn as_field(&self) -> Option<Field<'_>> {
            self.as_ref().and_then(Representation::as_field)
        }

        fn to_selected_node(&self) -> SelectedNode {
            match self {
                Self::None => Null.to_selected_node(),
                Self::Some(t) => t.to_selected_node(),
            }
        }

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                None => Representation::serialize::<C, _>(&Null, serializer),
                Some(inner) => inner.serialize::<C, _>(serializer),
            }
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct OptionVisitor<const C: u64, T>(PhantomData<T>);
            impl<'de, const C: u64, T: Representation> Visitor<'de> for OptionVisitor<C, T> {
                type Value = Option<T>;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "A nullable `{}`", T::NAME)
                }
                #[inline]
                fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                    Ok(None)
                }
                #[inline]
                fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                    Ok(None)
                }
                #[inline]
                fn visit_some<D: Deserializer<'de>>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, D::Error> {
                    T::deserialize::<C, _>(deserializer).map(Some)
                }
                fn __private_visit_untagged_option<D>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, ()>
                where
                    D: Deserializer<'de>,
                {
                    Ok(T::deserialize::<C, _>(deserializer).ok())
                }
            }

            deserializer.deserialize_option(OptionVisitor::<C, T>(PhantomData))
        }
    }
}

macro_rules! derive {
    ($self:ident @newtype_ref) => ($self.0);
    (@newtype $wrapper:ident) => { derive!($wrapper $wrapper, @newtype_ref); };
    ($self:ident @from_ref) => ($self.as_ref());
    (@from $wrapper:ident) => { derive!($wrapper $wrapper::from, @from_ref); };
    ($wrapper:ident $constructor:expr, @$as_ref:ident) => {
        impl<T: Representation> Representation for $wrapper<T> {
            const NAME: &'static str = T::NAME;
            const SCHEMA: &'static str = T::SCHEMA;
            const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
            const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
            const REPR_KIND: Kind = T::REPR_KIND;
            const REPR_STRATEGY: Strategy = T::REPR_STRATEGY;
            const HAS_LINKS: bool = T::HAS_LINKS;
            fn name(&self) -> &'static str {
                derive!(self @$as_ref).name()
            }
            fn has_links(&self) -> bool {
                derive!(self @$as_ref).has_links()
            }
            fn as_field(&self) -> Option<Field<'_>> {
                derive!(self @$as_ref).as_field()
            }
            fn to_selected_node(&self) -> SelectedNode {
                derive!(self @$as_ref).to_selected_node()
            }
            #[doc(hidden)]
            fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                derive!(self @$as_ref).serialize::<C, _>(serializer)
            }
            #[doc(hidden)]
            fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok($constructor(T::deserialize::<'de, C, _>(deserializer)?))
            }
        }

        impl<'a: 'de, 'de, const MC: u64, Ctx, T> Visitor<'de> for AstWalk<'a, MC, Ctx, $wrapper<T>>
        where
            Ctx: Context,
            T: Select<Ctx>,
        {
            type Value = AstResult<$wrapper<T>>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                unimplemented!()
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                unimplemented!()
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                unimplemented!()
            }
            fn visit_newtype_struct<D>(
                self,
                deserializer: D
            ) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                unimplemented!()
            }
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                unimplemented!()
            }
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                unimplemented!()
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                unimplemented!()
            }
        }
        impl<'a: 'de, 'de, const MC: u64, Ctx, T> LinkVisitor<'de, MC> for AstWalk<'a, MC, Ctx, $wrapper<T>>
        where
            Ctx: Context,
            T: Select<Ctx>,
        {
        }

        impl<Ctx, T> Select<Ctx> for $wrapper<T>
        where
            Ctx: Context,
            T: Select<Ctx> + 'static,
        {
            type Walker<'a, const MC: u64> = AstWalk<'a, MC, Ctx, Self> where Ctx: 'a;

            // #[doc(hidden)]
            // #[inline]
            // fn __select_de<'a, 'de, const C: u64, D>(
            //     seed: SelectorSeed<'a, Ctx, Self>,
            //     deserializer: D,
            // ) -> Result<(), D::Error>
            // where
            //     D: Deserializer<'de>,
            // {
            //     let seed = seed.wrap::<T, _>($constructor);
            //     T::__select_de::<C, D>(seed, deserializer)
            // }
        }
    };
    /*
        (@dyn $wrapper:ident) => {
            impl Representation for $wrapper<dyn ErasedRepresentation> {
                const NAME: &'static str = T::NAME;
                const SCHEMA: &'static str = T::SCHEMA;
                const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
                const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
                const REPR_KIND: Kind = T::REPR_KIND;
                const IS_LINK: bool = T::IS_LINK;
                const HAS_LINKS: bool = T::HAS_LINKS;

                #[inline]
                fn name(&self) -> &'static str {
                    self.as_ref().name()
                }

                fn data_model_kind(&self) -> Kind {
                    self.as_ref().data_model_kind()
                }

                fn schema_kind(&self) -> Kind {
                    self.as_ref().schema_kind()
                }

                fn repr_kind(&self) -> Kind {
                    self.as_ref().repr_kind()
                }

                fn has_links(&self) -> bool {
                    self.as_ref().has_links()
                }

                #[doc(hidden)]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    // self.as_ref().serialize::<C, _>(serializer)
                    unimplemented!()
                }

                #[doc(hidden)]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    // Ok(Self::new(T::deserialize::<'de, C, _>(deserializer)?))
                    unimplemented!()
                }
            }
        };
        */
}

derive!(@newtype Wrapping);
derive!(@from Box);
derive!(@from Rc);
derive!(@from Arc);
