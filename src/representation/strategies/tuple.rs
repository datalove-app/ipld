use crate::dev::*;
use macros::derive_more::From;
use maybestd::{cell::RefCell, fmt, marker::PhantomData, mem::MaybeUninit};

#[doc(hidden)]
#[macro_export]
macro_rules! tuple {
    ($name:ident $ref_name:ident $suffix:ident => $($ty:ident)*) => {
        /// Type that implements selection against tuple-represented structs.
        #[doc(hidden)]
        #[derive(Clone, Debug, From)]
        pub struct $ref_name<'a, T = Any, $($ty = Any,)*>($(pub &'a $ty,)* PhantomData<T>);

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

        impl<T, $($ty,)*> From<($($ty,)*)> for $name<T, $($ty,)*> {
            fn from(($($ty,)*): ($($ty,)*)) -> Self {
                Self($($ty,)* PhantomData)
            }
        }

        const _: () = {
            const LEN: usize = tuple!(@len $($ty)*);
            tuple!(@tuple $name $ref_name => $($ty)*);
        };
        const _: () = {
            const LEN: usize = tuple!(@len $($ty)*);
            tuple!(@struct_tuple $name $ref_name $suffix => $($ty)*);
        };
    };
    (@repr_methods $name:ident $repr:ident => $($ty:ident)*) => {
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
        impl<$($ty,)*> Representation for ($($ty,)*)
        where
            $($ty: Representation,)*
        {
            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str = concat!(
                "type ", stringify!($name), " struct { ",
                    $(stringify!($ty), stringify!($ty),)*
                " } representation tuple"
            );
            const DATA_MODEL_KIND: Kind = Kind::List;
            const SCHEMA_KIND: Kind = Kind::Struct;
            const REPR_KIND: Kind = Kind::List;
            const REPR_STRATEGY: Strategy = Strategy::Tuple;
            const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

            $crate::tuple!(@repr_methods $name List => $($ty)*);

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
                                .next_element_seed(DeserializeRepr::<MC, $ty>::new())?
                                .ok_or_else(|| Ac::Error::missing_field(""))?
                            ,)*
                        ))
                    }
                }

                deserializer.deserialize_seq(TupleVisitor::<MC, $($ty,)*>(PhantomData))
            }
        }
    };
    // (@alias $name:ident => $($ty:ident)*) => ()
    (@struct_tuple $name:ident $ref_name:ident $suffix:ident => $($ty:ident)*) => {
        impl<'a, T, $($ty,)*> Representation for $ref_name<'a, T, $($ty,)*>
        where
            T: Representation,
            $($ty: Representation,)*
        {
            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str = concat!(
                "type ", stringify!($name), " struct { ",
                    $(stringify!($ty), stringify!($ty),)*
                " } representation tuple"
            );
            const DATA_MODEL_KIND: Kind = Kind::Map;
            const SCHEMA_KIND: Kind = Kind::Struct;
            const REPR_KIND: Kind = Kind::List;
            const REPR_STRATEGY: Strategy = Strategy::Tuple;
            const FIELDS: &'static [&'static str] = T::FIELDS;
            const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

            $crate::tuple!(@repr_methods $ref_name List => $($ty)*);

            #[inline]
            #[doc(hidden)]
            fn serialize<const MC: u64, Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: Serializer,
            {
                use ser::SerializeSeq;

                let Self($($ty,)* _) = self;
                let mut seq = serializer.serialize_seq(Some(LEN))?;
                $(
                    seq.serialize_element(&SerializeRepr::<'_, MC, $ty>($ty))?;
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
            const NAME: &'static str = stringify!($name);
            const SCHEMA: &'static str = concat!(
                "type ", stringify!($name), " struct { ",
                    $(stringify!($ty), stringify!($ty),)*
                " } representation tuple"
            );
            const DATA_MODEL_KIND: Kind = Kind::Map;
            const SCHEMA_KIND: Kind = Kind::Struct;
            const REPR_KIND: Kind = Kind::List;
            const REPR_STRATEGY: Strategy = Strategy::Tuple;
            const FIELDS: &'static [&'static str] = T::FIELDS;
            const HAS_LINKS: bool = false $(| $ty::HAS_LINKS )*;

            $crate::tuple!(@repr_methods $name List => $($ty)*);

            #[inline]
            #[doc(hidden)]
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
        $crate::tuple!(@select $name $suffix => $($ty)* { $($ty)* });
        $crate::repr_serde! { @visitors for $name<T, $($ty,)*>
            { T, $($ty,)* } { T: Select<Ctx> + 'static,
                              $($ty: Select<Ctx> + 'static,)* } @serde {
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A len {}-tuple struct of type {}", LEN, T::NAME)
            }
            #[inline]
            fn visit_seq<Ac>(self, seq: Ac) -> Result<Self::Value, Ac::Error>
            where
                Ac: SeqAccess<'de>,
            { paste::paste! {
                // self.into_inner()
                //     .[<select $suffix>]::<MC, false, $($ty,)* _>()
                unimplemented!()
            }}
        }};
    };
    (@select $name:ident $suffix:ident => $($ty:ident)* { $($ty2:ident)* }) => {
        // todo impl MapIterator<String, $ty>
        // todo     for [<Iter $suffix>]< $name<T, $($ty,)*> >
        // todo     for [<SerdeIter $suffix>]< $name<T, $($ty,)*> >

        impl<'a, Ctx, T> SelectorSeed<'a, Ctx, T>
        where
            Ctx: Context,
            T: Select<Ctx> + 'static,
        { paste::paste! {
            ///
            /// todo: only implements matcher
            pub fn [<select $suffix>]<const MC: u64, $($ty,)* It, Cfn>(
                mut self,
                mut iter: It,
                constructor: Cfn,
            ) -> Result<(), Error>
            where
                $($ty: Select<Ctx> + 'static,)*
                $(It: MapIterator<String, $ty>,)*
                Cfn: FnOnce($($ty,)*) -> Result<T, Error>,
            {
                // select the matched node, and set up the dag
                self.handle_node(SelectedNode::Map)?;
                // FIXME: should maybe just use an option, since idx might get inited
                $(let [<v_ $ty:lower>] = RefCell::new(MaybeUninit::uninit());)*
                // create cleanup cb
                let drop_inited = |idx: usize, err: Error| -> Result<(), Error> {
                    let mut i = 0usize;
                    $({ if i < idx {
                        unsafe {
                            [<v_ $ty:lower>].borrow_mut().assume_init_drop();
                        }
                        i += 1;
                    }})*
                    Err(err)
                };

                //
                let mut idx = 0usize;
                $(  if let Err(err) = <_ as MapIterator<String, $ty>>::next_key::<MC>(&mut iter, Some(T::FIELDS[idx]))
                        .and_then(|_: Option<String>| self.handle_field::<MC, String, $ty>(
                            &mut iter,
                            self.is_dag_select().then_some(Box::new(|child, _| {
                                [<v_ $ty:lower>].borrow_mut().write(child);
                                Ok(())
                            })),
                        ))
                    {
                        return drop_inited(idx, err);
                    }

                    idx += 1;
                    // if idx < LEN {
                    //     iter.into_next()
                    // } else {
                    //     iter
                    // }
                )*

                // match dag
                if self.is_dag_select() {
                    $(let [<v_ $ty:lower>] = unsafe {
                        [<v_ $ty:lower>].into_inner().assume_init()
                    };)*
                    self.handle_dag(constructor($([<v_ $ty:lower>],)*)?)?;
                }

                Ok(())
            }

            pub fn [<patch $suffix>]<const MC: u64, $($ty,)* It>(
                mut self,
                mut iter: It,
            ) -> Result<(), Error>
            where
                $($ty: Select<Ctx> + 'static,)*
                $(It: MapIterator<String, $ty>,)*
            {
                unimplemented!()
            }
        }}
    };
}

struct TupleSerdeIter<const MAX: usize, T, U, Ac> {
    pub index: usize,
    inner: Ac,
    _t: PhantomData<(T, U)>,
}
impl<const MAX: usize, T, U, Ac> TupleSerdeIter<MAX, T, U, Ac> {
    // pub fn into_next<V>(self) -> TupleSerdeIter<MAX, T, V, Ac> {
    //     TupleSerdeIter {
    //         index: self.index + 1,
    //         inner: self.inner,
    //         _t: PhantomData,
    //     }
    // }
}

tuple!(Tuple2 TupleRef2 _2 => A B);
tuple!(Tuple3 TupleRef3 _3 => A B C);
tuple!(Tuple4 TupleRef4 _4 => A B C D);
tuple!(Tuple5 TupleRef5 _5 => A B C D E);
tuple!(Tuple6 TupleRef6 _6 => A B C D E F);
tuple!(Tuple7 TupleRef7 _7 => A B C D E F G);
tuple!(Tuple8 TupleRef8 _8 => A B C D E F G H);
tuple!(Tuple9 TupleRef9 _9 => A B C D E F G H I);
tuple!(Tuple10 TupleRef10 _10 => A B C D E F G H I J);
tuple!(Tuple11 TupleRef11 _11 => A B C D E F G H I J K);
tuple!(Tuple12 TupleRef12 _12 => A B C D E F G H I J K L);
tuple!(Tuple13 TupleRef13 _13 => A B C D E F G H I J K L M);
tuple!(Tuple14 TupleRef14 _14 => A B C D E F G H I J K L M N);
tuple!(Tuple15 TupleRef15 _15 => A B C D E F G H I J K L M N O);
tuple!(Tuple16 TupleRef16 _16 => A B C D E F G H I J K L M N O P);
tuple!(Tuple17 TupleRef17 _17 => A B C D E F G H I J K L M N O P Q);
tuple!(Tuple18 TupleRef18 _18 => A B C D E F G H I J K L M N O P Q R);
tuple!(Tuple19 TupleRef19 _19 => A B C D E F G H I J K L M N O P Q R S);

// impl<const MAX: usize, T, U, Ac> MapIterator<String, U> for TupleSerdeIter<MAX, T, U, Ac> {
//     fn size_hint(&self) -> Option<usize> {
//         Some(MAX)
//     }

//     fn field(&self) -> Field<'_> {
//         unimplemented!()
//     }

//     fn next_ignored(&mut self) -> Result<bool, Error> {
//         Ok(false)
//     }
// }

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
