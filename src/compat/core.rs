use crate::dev::*;
use macros::repr_serde;
use maybestd::{fmt, marker::PhantomData};

mod ignored {
    use super::*;

    impl Representation for IgnoredAny {
        type DataModelKind = type_kinds::Null;
        type SchemaKind = type_kinds::Union;
        type ReprKind = type_kinds::Any;

        const NAME: &'static str = "IgnoredAny";
        const SCHEMA: &'static str = "type IgnoredAny = Any";
        const DATA_MODEL_KIND: Kind = Kind::Null;
        const SCHEMA_KIND: Kind = Kind::Union;
        const REPR_KIND: Kind = Kind::Any;
        const __IGNORED: bool = true;

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

    repr_serde! { @select_for IgnoredAny }
    repr_serde! { @visitors for IgnoredAny => IgnoredAny
        { @dk (type_kinds::Null) @sk (type_kinds::Union) @rk (type_kinds::Any) }
        {} {} @serde {
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", IgnoredAny::NAME)
            }
            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(())
            }
        }
    }

    ///
    pub type Ignored<T> = PhantomData<T>;

    impl<T: Representation> Representation for Ignored<T> {
        type DataModelKind = type_kinds::Null;
        type SchemaKind = type_kinds::Union;
        type ReprKind = type_kinds::Any;

        const NAME: &'static str = "Ignored";
        const SCHEMA: &'static str = "type Ignored = IgnoredAny";
        const DATA_MODEL_KIND: Kind = Kind::Null;
        const SCHEMA_KIND: Kind = Kind::Union;
        const REPR_KIND: Kind = Kind::Any;
        const __IGNORED: bool = true;

        #[doc(hidden)]
        fn serialize<const C: u64, S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("unimplemented"))
        }

        #[doc(hidden)]
        fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Err(D::Error::custom("unimplemented"))
        }
    }

    repr_serde! { @select_for Ignored<T> => Ignored<T>
        { @dk (type_kinds::Null) @sk (type_kinds::Union) @rk (type_kinds::Any) }
        { T } { T: Representation }
    }
    repr_serde! { @visitors for Ignored<T> => Ignored<T>
        { @dk (type_kinds::Null) @sk (type_kinds::Union) @rk (type_kinds::Any) }
        { T } { T: Representation + '_a } @serde {
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", T::NAME)
            }
            // TODO: delegate
        }
    }
}

// TODO:
mod option {
    use super::*;

    // ? define Optional<T> and Nullable<T>?

    impl<T> Representation for Option<T>
    where
        T: Representation,
    {
        type DataModelKind = T::DataModelKind;
        type SchemaKind = T::SchemaKind;
        type ReprKind = T::ReprKind;
        // type ReprKind = type_kinds::Optional<<T as Representation>::ReprKind>;
        // type ReprKind = typenum::op!(type_kinds::Null | T::ReprKind);
        // type ReprKind = typenum::Or<type_kinds::Null, T::ReprKind>;

        const NAME: &'static str = concat!("Optional", stringify!(T::NAME));
        const SCHEMA: &'static str = concat!("type ", stringify!(T::NAME), " nullable");
        const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
        const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
        const REPR_KIND: Kind = T::REPR_KIND;
        const IS_LINK: bool = T::IS_LINK;
        const HAS_LINKS: bool = T::HAS_LINKS;

        fn name(&self) -> &'static str {
            match self {
                Self::None => Null::NAME,
                Self::Some(t) => t.name(),
            }
        }

        // fn kind(&self) -> Kind {
        //     match self {
        //         Self::None => Null::KIND,
        //         Self::Some(t) => t.kind(),
        //     }
        // }

        fn has_links(&self) -> bool {
            match self {
                Self::None => false,
                Self::Some(t) => t.has_links(),
            }
        }

        fn as_field(&self) -> Option<Field<'_>> {
            self.as_ref().and_then(Representation::as_field)
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
                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("Optional")
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

mod wrapper {
    use super::*;
    use maybestd::{rc::Rc, sync::Arc};

    macro_rules! impl_wrapper {
        ($wrapper:ident) => {
            impl<T> Representation for $wrapper<T>
            where
                T: Representation,
            {
                type DataModelKind = T::DataModelKind;
                type SchemaKind = T::SchemaKind;
                type ReprKind = T::ReprKind;

                const NAME: &'static str = T::NAME;
                const SCHEMA: &'static str = T::SCHEMA;
                const DATA_MODEL_KIND: Kind = T::DATA_MODEL_KIND;
                const SCHEMA_KIND: Kind = T::SCHEMA_KIND;
                const REPR_KIND: Kind = T::REPR_KIND;
                const IS_LINK: bool = T::IS_LINK;
                const HAS_LINKS: bool = T::HAS_LINKS;

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

                fn as_field(&self) -> Option<Field<'_>> {
                    self.as_ref().as_field()
                }

                #[doc(hidden)]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    self.as_ref().serialize::<C, _>(serializer)
                }

                #[doc(hidden)]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    Ok(Self::new(T::deserialize::<'de, C, _>(deserializer)?))
                }
            }

            impl<Ctx, T> Select<Ctx> for $wrapper<T>
            where
                Ctx: Context,
                T: Select<Ctx> + 'static,
            {
                // #[doc(hidden)]
                // #[inline]
                // fn __select<'a>(seed: SelectorSeed<'a, Ctx, Self>) -> Result<(), Error> {
                //     T::__select(seed.wrap::<T, _>($wrapper::from))
                // }

                #[doc(hidden)]
                #[inline]
                fn __select_de<'a, 'de, const C: u64, D>(
                    seed: SelectorSeed<'a, Ctx, Self>,
                    deserializer: D,
                ) -> Result<(), D::Error>
                where
                    D: Deserializer<'de>,
                {
                    T::__select_de::<C, D>(seed.wrap::<T, _>($wrapper::from), deserializer)
                }

                #[doc(hidden)]
                #[inline]
                fn __select_seq<'a, 'de, const C: u64, A>(
                    seed: SelectorSeed<'a, Ctx, Self>,
                    seq: A,
                ) -> Result<Option<()>, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    T::__select_seq::<C, A>(seed.wrap::<T, _>($wrapper::from), seq)
                }

                #[doc(hidden)]
                #[inline]
                fn __select_map<'a, 'de, const C: u64, A>(
                    seed: SelectorSeed<'a, Ctx, Self>,
                    map: A,
                    is_key: bool,
                ) -> Result<Option<()>, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    T::__select_map::<C, A>(seed.wrap::<T, _>($wrapper::from), map, is_key)
                }
            }
        }; /*
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

    impl_wrapper!(Box);
    impl_wrapper!(Rc);
    impl_wrapper!(Arc);
}
