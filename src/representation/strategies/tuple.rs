use crate::dev::*;
use macros::derive_more::From;
use maybestd::{fmt, marker::PhantomData};

#[doc(hidden)]
#[macro_export]
macro_rules! tuple {
    ($name:ident $ref_name:ident => $($ty:ident)*) => {
        /// Type that implements selection against tuple-represented structs.
        #[doc(hidden)]
        #[derive(Clone, Debug, From)]
        pub struct $ref_name<'a, T = Any, $($ty = Any,)*>($(pub &'a $ty,)* PhantomData<T>);

        impl<'a, T, $($ty,)*> $ref_name<'a, T, $($ty,)*>
        where
            T: Representation,
            $($ty: Representation,)*
        {
            pub(crate) const LEN: usize = tuple!(@len $($ty)*);
        }

        impl<'a, T, $($ty,)*> From<&'a ($($ty,)*)>
            for $ref_name<'a, T, $($ty,)*>
        {
            #[inline]
            fn from(($($ty,)*): &'a ($($ty,)*)) -> Self {
                Self($($ty,)* PhantomData)
            }
        }

        impl<'a, T, $($ty,)*> From<($(&'a $ty,)*)>
            for $ref_name<'a, T, $($ty,)*>
        {
            #[inline]
            fn from(($($ty,)*): ($(&'a $ty,)*)) -> Self {
                Self($($ty,)* PhantomData)
            }
        }

        impl<'a, T, $($ty,)*> From<&'a $name<T, $($ty,)*>>
            for $ref_name<'a, T, $($ty,)*>
        {
            #[inline]
            fn from(tuple: &'a $name<T, $($ty,)*>) -> Self {
                let $name($(ref $ty,)* _) = tuple;
                Self($($ty,)* PhantomData)
            }
        }

        ///
        #[doc(hidden)]
        #[derive(Clone, Debug, From)]
        pub struct $name<T = Any, $($ty = Any,)*>($(pub $ty,)* PhantomData<T>);

        impl<T, $($ty,)*> $name<T, $($ty,)*>
        where
            T: Representation,
            $($ty: Representation,)*
        {
            pub(crate) const LEN: usize = tuple!(@len $($ty)*);
        }

        impl<T, $($ty,)*> From<($($ty,)*)> for $name<T, $($ty,)*> {
            fn from(($($ty,)*): ($($ty,)*)) -> Self {
                Self($($ty,)* PhantomData)
            }
        }

        const _: () = {
            tuple!(@tuple $name $ref_name => $($ty)*);
        };
        const _: () = {
            tuple!(@struct_tuple $name $ref_name => $($ty)*);
        };
        const _: () = {
            // tuple!(listpairs! $name $ref_name => $($ty)*);
        };
        const _: () = {
            // tuple!(stringjoin! $name $ref_name => $($ty)*);
        };
        const _: () = {
            // tuple!(stringpairs! $name $ref_name => $($ty)*);
        };
        const _: () = {
            // tuple!(stringprefix! $name $ref_name => $($ty)*);
        };
    };
    (@repr_ext $name:ident $repr:ident => $($ty:ident)*) => {
        fn has_links(&self) -> bool {
            // let ($($ty,)*) = self;
            // false $(| $ty.has_links())*
            unimplemented!()
        }
    };
    (@len) => (0);
    (@len $first:ident $($ty:ident)*) => (1usize + $crate::tuple!(@len $($ty)*));
    // ($macro_name:ident! $name:ident $ref_name:ident => $($ty:ident)*) => {
    //     $macro_name!($name $ref_name => $($ty)*);
    // };
    (@tuple $name:ident $ref_name:ident => $($ty:ident)*) => {
        const LEN: usize = $crate::tuple!(@len $($ty)*);

        impl<$($ty,)*> Representation for ($($ty,)*)
        where
            $($ty: Representation,)*
        {
            type DataModelKind = type_kinds::List;
            type SchemaKind = type_kinds::Struct;
            type ReprKind = type_kinds::List;

            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str = "";
            const DATA_MODEL_KIND: Kind = Kind::List;
            const SCHEMA_KIND: Kind = Kind::Struct;
            const REPR_KIND: Kind = Kind::List;
            const REPR_STRATEGY: Strategy = Strategy::Tuple;
            const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

            $crate::tuple!(@repr_ext $name List => $($ty)*);

            fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: Serializer,
            {
                <$ref_name<'_, Null, $($ty,)*>>::from(self)
                    .serialize::<MC, _>(serializer)
            }

            #[inline]
            #[doc(hidden)]
            fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
            where
                De: Deserializer<'de>,
            {
                struct TupleVisitor<const MC: u64, $($ty,)*>(PhantomData<($($ty,)*)>);
                impl<'de, const MC: u64, $($ty,)*> Visitor<'de> for TupleVisitor<MC, $($ty,)*>
                where
                    $($ty: Representation,)*
                {
                    type Value = ($($ty,)*);
                    #[inline]
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "A tuple of {} elements", LEN)
                    }
                    #[inline]
                    fn visit_seq<Ac>(self, mut seq: Ac) -> Result<Self::Value, Ac::Error>
                    where
                        Ac: SeqAccess<'de>,
                    {
                        Ok((
                            $(seq
                                .next_element_seed(DeserializeWrapper::<MC, $ty>::new())?
                                .ok_or_else(|| Ac::Error::missing_field(""))?
                            ,)*
                        ))
                    }
                }

                deserializer.deserialize_seq(TupleVisitor::<MC, $($ty,)*>(PhantomData))
            }
        }
    };
    (@struct_tuple $name:ident $ref_name:ident => $($ty:ident)*) => {
        impl<'a, T, $($ty,)*> Representation for $ref_name<'a, T, $($ty,)*>
        where
            T: Representation,
            $($ty: Representation,)*
        {
            type DataModelKind = type_kinds::Map;
            type SchemaKind = type_kinds::Struct;
            type ReprKind = type_kinds::List;

            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str = "";
            const DATA_MODEL_KIND: Kind = Kind::Map;
            const SCHEMA_KIND: Kind = Kind::Struct;
            const REPR_KIND: Kind = Kind::List;
            const REPR_STRATEGY: Strategy = Strategy::Tuple;
            const FIELDS: &'static [&'static str] = T::FIELDS;
            const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

            $crate::tuple!(@repr_ext $ref_name List => $($ty)*);

            fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: Serializer,
            {
                use ser::SerializeSeq;

                let Self($($ty,)* _) = self;
                let mut seq = serializer.serialize_seq(Some(Self::LEN))?;
                $(
                    seq.serialize_element(&SerializeWrapper::<'_, MC, $ty>($ty))?;
                )*
                seq.end()
            }

            #[inline]
            #[doc(hidden)]
            fn deserialize<'de, const MC: u64, De>(_: De) -> Result<Self, De::Error>
            where
                De: Deserializer<'de>,
            {
                unreachable!(
                    "cannot deserialize {}; use {}::as_ref instead",
                    stringify!($ref_name),
                    stringify!($name),
                )
            }
        }

        impl<T, $($ty,)*> Representation for $name<T, $($ty,)*>
        where
            T: Representation,
            $($ty: Representation,)*
        {
            type DataModelKind = type_kinds::Map;
            type SchemaKind = type_kinds::Struct;
            type ReprKind = type_kinds::List;

            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str = "";
            const DATA_MODEL_KIND: Kind = Kind::Map;
            const SCHEMA_KIND: Kind = Kind::Struct;
            const REPR_KIND: Kind = Kind::List;
            const REPR_STRATEGY: Strategy = Strategy::Tuple;
            const FIELDS: &'static [&'static str] = T::FIELDS;
            const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

            $crate::tuple!(@repr_ext $name List => $($ty)*);

            fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: Serializer,
            {
                $ref_name::from(self).serialize::<MC, Se>(serializer)
            }

            #[inline]
            #[doc(hidden)]
            fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
            where
                De: Deserializer<'de>,
            {
                Ok(Self::from(<($($ty,)*)>::deserialize::<MC, _>(deserializer)?))
            }
        }

        // // Blanket impls for structs.
        // repr_serde! { @visitors for T => $name<T, $($ty,)*>
        //     { @dk type_kinds::Map @sk type_kinds::Struct @rk type_kinds::List }
        //     { T, $($ty)* }
        //     { T: Default + TryFrom<($($ty,)*)> + 'static,
        //       <T as TryFrom<($($ty,)*)>>::Error: fmt::Display,
        //       $($ty: Select<Ctx> + 'static,)*, } @serde {
        //     }
        // };
    };
}

tuple!(StructTuple2 StructTupleRef2 => A B);
tuple!(StructTuple3 StructTupleRef3 => A B C);
tuple!(StructTuple4 StructTupleRef4 => A B C D);
tuple!(StructTuple5 StructTupleRef5 => A B C D E);
tuple!(StructTuple6 StructTupleRef6 => A B C D E F);
tuple!(StructTuple7 StructTupleRef7 => A B C D E F G);
tuple!(StructTuple8 StructTupleRef8 => A B C D E F G H);
tuple!(StructTuple9 StructTupleRef9 => A B C D E F G H I);
tuple!(StructTuple10 StructTupleRef10 => A B C D E F G H I J);
tuple!(StructTuple11 StructTupleRef11 => A B C D E F G H I J K);
tuple!(StructTuple12 StructTupleRef12 => A B C D E F G H I J K L);
tuple!(StructTuple13 StructTupleRef13 => A B C D E F G H I J K L M);
tuple!(StructTuple14 StructTupleRef14 => A B C D E F G H I J K L M N);
tuple!(StructTuple15 StructTupleRef15 => A B C D E F G H I J K L M N O);
tuple!(StructTuple16 StructTupleRef16 => A B C D E F G H I J K L M N O P);
tuple!(StructTuple17 StructTupleRef17 => A B C D E F G H I J K L M N O P Q);
tuple!(StructTuple18 StructTupleRef18 => A B C D E F G H I J K L M N O P Q R);
tuple!(StructTuple19 StructTupleRef19 => A B C D E F G H I J K L M N O P Q R S);

// impl_tuple!(@repr Tuple12 Tuple12RefTuple2Ref => A B C D E F G H I J K L {

// } {
//     fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
//     where
//         Se: Serializer,
//     {
//         use ser::SerializeSeq;

//         // let Self($(ref $ty,)*) = self;
//         // let mut seq = serializer.serialize_seq(Some(LEN))?;
//         // $(
//         //     seq.serialize_element(&SerializeWrapper::<'_, MC, _>($ty))?;
//         // )*
//         // seq.end()
//         // self.as_ref().serialize::<MC, _>(serializer)
//         unimplemented!()
//     }

//     #[inline]
//     #[doc(hidden)]
//     fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
//     where
//         De: Deserializer<'de>,
//     {
//         unimplemented!()

//         //     struct TupleVisitor<const MC: u64, const S: u8, $($ty,)*>(PhantomData<($($ty,)*)>);
//         //     impl<const MC: u64, const S: u8, $($ty,)*> Default for TupleVisitor<MC, S, $($ty,)*> {
//         //         fn default() -> Self {
//         //             Self(PhantomData)
//         //         }
//         //     }
//         //     impl<'de, const MC: u64, const S: u8, $($ty,)*> Visitor<'de> for TupleVisitor<MC, S, $($ty,)*> {
//         //         type Value = $name<S, $($ty,)*>;
//         //         #[inline]
//         //         fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         //             // write!(f, "A list of `{}`", T::NAME)
//         //             unimplemented!()
//         //         }

//         //         #[inline]
//         //         fn visit_seq<Ac>(self, mut seq: Ac) -> Result<Self::Value, Ac::Error>
//         //         where
//         //             Ac: SeqAccess<'de>,
//         //         {
//         //             // let mut list = List::with_capacity(seq.size_hint().unwrap_or(8));
//         //             // while let Some(elem) =
//         //             //     seq.next_element_seed(DeserializeWrapper::<MC, T>::default())?
//         //             // {
//         //             //     list.push(elem);
//         //             // }
//         //             // Ok(list)
//         //             unimplemented!()
//         //         }
//         //     }

//         //     deserializer.deserialize_seq(TupleVisitor::<MC, { $strategy }, $($ty,)*>::default())
//         // }
//     }
// });

// impl_tuple!(Tuple2 Tuple2Ref Strategy::BytesPrefix as u8 => A B {
//     #[inline]
//     #[doc(hidden)]
//     fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
//     where
//         Se: Serializer,
//     {
//         use ser::SerializeSeq;

//         // let Self($(ref $ty,)*) = self;
//         // let mut seq = serializer.serialize_seq(Some(LEN))?;
//         // $(
//         //     seq.serialize_element(&SerializeWrapper::<'_, MC, _>($ty))?;
//         // )*
//         // seq.end()
//         // self.as_ref().serialize::<MC, _>(serializer)
//         unimplemented!()
//     }

//     #[inline]
//     #[doc(hidden)]
//     fn deserialize<'de, const MC: u64, De>(deserializer: De) -> Result<Self, De::Error>
//     where
//         De: Deserializer<'de>,
//     {
//         unimplemented!()

//         //     struct TupleVisitor<const MC: u64, const S: u8, $($ty,)*>(PhantomData<($($ty,)*)>);
//         //     impl<const MC: u64, const S: u8, $($ty,)*> Default for TupleVisitor<MC, S, $($ty,)*> {
//         //         fn default() -> Self {
//         //             Self(PhantomData)
//         //         }
//         //     }
//         //     impl<'de, const MC: u64, const S: u8, $($ty,)*> Visitor<'de> for TupleVisitor<MC, S, $($ty,)*> {
//         //         type Value = $name<S, $($ty,)*>;
//         //         #[inline]
//         //         fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         //             // write!(f, "A list of `{}`", T::NAME)
//         //             unimplemented!()
//         //         }

//         //         #[inline]
//         //         fn visit_seq<Ac>(self, mut seq: Ac) -> Result<Self::Value, Ac::Error>
//         //         where
//         //             Ac: SeqAccess<'de>,
//         //         {
//         //             // let mut list = List::with_capacity(seq.size_hint().unwrap_or(8));
//         //             // while let Some(elem) =
//         //             //     seq.next_element_seed(DeserializeWrapper::<MC, T>::default())?
//         //             // {
//         //             //     list.push(elem);
//         //             // }
//         //             // Ok(list)
//         //             unimplemented!()
//         //         }
//         //     }

//         //     deserializer.deserialize_seq(TupleVisitor::<MC, { $strategy }, $($ty,)*>::default())
//         // }
//     }
// });
// // impl_tuple!(Tuple3 Tuple3Ref => A B C);
// // impl_tuple!(Tuple4 Tuple4Ref => A B C D);
// // impl_tuple!(Tuple5 Tuple5Ref => A B C D E);
// // impl_tuple!(Tuple6 Tuple6Ref => A B C D E F);
// // impl_tuple!(Tuple7 Tuple7Ref => A B C D E F G);
// // impl_tuple!(Tuple8 Tuple8Ref => A B C D E F G H);
// // impl_tuple!(Tuple9 Tuple9Ref => A B C D E F G H I);
// // impl_tuple!(Tuple10 Tuple10Ref => A B C D E F G H I J);
// // impl_tuple!(Tuple11 Tuple11Ref => A B C D E F G H I J K);
// // impl_tuple!(Tuple12 Tuple12Ref => A B C D E F G H I J K L);
