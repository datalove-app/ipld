use crate::dev::*;

mod option {
    use super::*;

    impl<T> Representation for Option<T>
    where
        T: Representation,
    {
        const NAME: &'static str = concat!("Optional", stringify!(T::NAME));
        // TODO
        const SCHEMA: &'static str = unimplemented!();
        // const SCHEMA: &'static str = concat!("type ", stringify!(T::NAME), " nullable");
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
            // DeserializeWrapper::<C, _>::default().deserialize(deserializer)
            unimplemented!()
        }
    }
}

mod wrapper {
    use super::*;
    use std::{rc::Rc, sync::Arc};

    macro_rules! impl_wrapper {
        ($wrapper:ident) => {
            impl<T> Representation for $wrapper<T>
            where
                T: Representation,
            {
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
