use crate::dev::*;

/// Provides skeletons for implementing IPLD traits for custom types.
#[macro_export]
macro_rules! impl_ipld {
    (@visitor
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty => $select_ty:ty
        { $($visitor_fns:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::Visitor<'de> for $crate::dev::ContextSeed<'a, C, $type, $select_ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = Option<$select_ty>;

            $($visitor_fns)*
        }
    };
    (@visitor_ext
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty => $select_ty:ty
        { $($visit_fns:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::IpldVisitorExt<'de> for $crate::dev::ContextSeed<'a, C, $type, $select_ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            $($visit_fns)*
        }
    };
    (@deseed
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty => $select_ty:ty
        { $($deseed_fn:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::DeserializeSeed<'de> for $crate::dev::ContextSeed<'a, C, $type, $select_ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = Option<$select_ty>;

            $($deseed_fn)*
        }
    };
    (@select_self
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty
    ) => {
        $crate::dev::impl_ipld! {
            @match_self { $($generics)* } { $($bounds)* } $type {
                /// Produces a stream of [`Selection`]s.
                #[inline]
                fn select<S: $crate::dev::Select<C>>(
                    selector: &$crate::dev::Selector,
                    state: &mut $crate::dev::SelectorState,
                    ctx: &mut C
                ) -> Result<Option<S>, Error> {
                    S::r#match(selector, state, ctx)
                }
            }
        }
    };
    (@match_self
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty
        { $($select_fn:tt)* }
    ) => {
        impl<C, $($generics)*> $crate::dev::Select<C> for $type
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            #[inline]
            fn r#match(
                selector: &$crate::dev::Selector,
                state: &mut $crate::dev::SelectorState,
                ctx: &mut C
            ) -> Result<Option<Self>, $crate::dev::Error> {
                $crate::dev::ContextSeed::<'_, C, Self>::from(selector, state, ctx).decode()
            }

            $($select_fn)*
        }
    };

    // (@select_default
    //     { $($generics:tt)* } { $($bounds:tt)* }
    //     $type:ty
    // ) => {
    //     $crate::dev::impl_ipld! {
    //         @select { $($generics)* } { $($bounds)* } $type {
    //             /// Produces a stream of [`Selection`]s.
    //             #[inline]
    //             fn select<S: $crate::dev::Select<C>>(
    //                 selector: &$crate::dev::Selector,
    //                 state: $crate::dev::SelectorState,
    //                 ctx: &mut C
    //             ) -> Result<Option<S>, Error> {
    //                 let deserializer = ctx.path_decoder(state.path())?;
    //                 $crate::dev::ContextSeed::<'_, C, Self, S>::from(selector, state, ctx)
    //                     .deserialize(deserializer)
    //                     .map_err($crate::dev::Error::decoder)
    //             }
    //         }
    //     }
    // };
    // (visit_self
    //     { $($generics:tt)* } { $($bounds:tt)* }
    //     $type:ty
    //     { $($visit_fn:tt)* }
    //     { $($flush_fn:tt)* }
    // ) => {
    //     impl<C, $($generics)*> $crate::dev::Visit<C> for $type
    //     where
    //         C: $crate::dev::Context,
    //         $($bounds)*
    //     {
    //         // #[inline]
    //         // fn r#match(
    //         //     selector: &$crate::dev::Selector,
    //         //     state: $crate::dev::SelectorState,
    //         //     ctx: &mut C
    //         // ) -> Result<Option<Self>, $crate::dev::Error> {
    //         //     let deserializer = ctx.path_decoder(state.path())?;
    //         //     $crate::dev::ContextSeed::<'_, C, Self, Self>::from(selector, state, ctx)
    //         //         .deserialize(deserializer)
    //         //         .map_err($crate::dev::Error::decoder)
    //         // }

    //         fn visit<F, T: $crate::dev::Representation>(
    //             &mut self,
    //             selector: &$crate::dev::Selector,
    //             state: $crate::dev::SelectorState,
    //             ctx: &mut C,
    //             op: F,
    //         ) -> Result<(), $crate::dev::Error>
    //         where
    //             F: Fn(T, &mut C) -> Result<Option<T>, $crate::dev::Error>,
    //         {
    //             unimplemented!()
    //         }

    //         fn flush(
    //             &mut self,
    //             selector: &$crate::dev::Selector,
    //             state: $crate::dev::SelectorState,
    //             ctx: &mut C,
    //         ) -> Result<(), $crate::dev::Error> {
    //             unimplemented!()
    //         }
    //     }
    // };
    // (visit_primitive
    //     { $($generics:tt)* } { $($bounds:tt)* }
    //     $type:ty
    // ) => {
    //     $crate::dev::impl_ipld! {
    //         @match_self { $($generics)* } { $($bounds)* } $type {
    //             /// Produces a stream of [`Selection`]s.
    //             #[inline]
    //             fn select<S: $crate::dev::Select<C>>(
    //                 selector: &$crate::dev::Selector,
    //                 state: $crate::dev::SelectorState,
    //                 ctx: &mut C
    //             ) -> Result<Option<S>, Error>
    //             {
    //                 S::r#match(selector, ctx)
    //             }
    //         }
    //     }
    // };
}

///
/// `key` is map/struct field name, `value` is its type
pub fn serialize_map_field<T>(key: &str, value: &T) -> Result<(), ()>
where
    T: Serialize + Representation,
{
    // if !(<T as Representation>::KIND == Kind::Union) {
    //     return Ok(());
    // }

    Err(())

    // if is_keyed::<T>() {
    //     // write union variant name T::name(value), then value
    //     Ok(())
    // } else if is_envelope::<T>() {
    //     // write (tag, tag_value), key: value
    //     Ok(())
    // } else if is_inline::<T>() {
    //     // write (tag, tag_value), then inline all value's keys (must be map or struct or union of maps)
    //     Ok(())
    // } else if is_kinded::<T>() {
    //     // write key, then value
    //     Ok(())
    // } else if is_byteprefix::<T>() {
    //     // is_byteprefix
    //     Ok(())
    // } else {
    //     Err(())
    // }
}

// pub const fn is_primitive<T: Representation>() -> bool {
//     match <T as Representation>::KIND {
//         Kind::Null | Kind::Boolean | Kind::Integer | Kind::Float | Kind::String | Kind::Bytes
//             if has_no_fields::<T>() =>
//         {
//             true
//         }
//         _ => false,
//     }
// }

macro_rules! match_kind {
    ($expr:expr, $kind:ident) => {
        match $expr {
            Kind::$kind => true,
            _ => return false,
        }
    };
}

// pub const fn is_enum<T: Representation>() -> bool {
//     match_kind!(<T as Representation>::KIND, Enum);

//     const fn validate_fields<T: Representation>(kind: &Kind, fields: &[Field<()>]) -> bool {
//         match fields {
//             &[] => true,
//             &[ref field, ref rest @ ..] => {
//                 field.value == *kind && validate_fields::<T>(kind, &rest)
//             }
//         }
//     }

//     let Fields::Enum { ref kind, fields } = <T as Representation>::FIELDS;
//     validate_fields::<T>(kind, fields)
// }

// pub const fn is_union<T: Representation>() -> bool {
//     if <T as Representation>::KIND != Kind::Union {
//         return false;
//     }

// }

// #[macro_export(local_inner_macros)]
// macro_rules! gen_has_field_kind {
//     ($fn_name:ident, $kind:pat) => {
//         const fn $fn_name<T: Representation>() -> bool {
//             match <T as Representation>::FIELDS {
//                 $kind => true,
//                 _ => false,
//             }
//         }
//     };
// }

// gen_has_field_kind!(has_no_fields, Fields::None);
// gen_has_field_kind!(has_list_fields, Fields::List(_));
// gen_has_field_kind!(has_map_fields, Fields::Map { .. });
// gen_has_field_kind!(has_struct_fields, Fields::Struct(_));
// gen_has_field_kind!(has_enum_fields, Fields::Enum { .. });
// gen_has_field_kind!(has_keyed_fields, Fields::Keyed(_));
// gen_has_field_kind!(has_envelope_fields, Fields::Envelope { .. });
// gen_has_field_kind!(has_inline_fields, Fields::Inline { .. });
// gen_has_field_kind!(has_kinded_fields, Fields::Kinded(_));
// gen_has_field_kind!(has_byteprefix_fields, Fields::Byteprefix(_));
