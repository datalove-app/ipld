/// Provides skeletons for conveniently implementing serde-compatibility for
/// IPLD types.
#[macro_export]
macro_rules! impl_ipld_serde {
    (@context_visitor
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty
        { $($visitor_fns:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::Visitor<'de> for $crate::dev::ContextSeed<'a, C, $type>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = ();

            $($visitor_fns)*
        }
    };
    (@context_visitor_ext
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty
        { $($visit_fns:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::IpldVisitorExt<'de> for $crate::dev::ContextSeed<'a, C, $type>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            $($visit_fns)*
        }
    };
    (@context_deseed
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty
        { $($deseed_fn:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::DeserializeSeed<'de> for $crate::dev::ContextSeed<'a, C, $type>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = ();

            $($deseed_fn)*
        }
    };
    (@context_select
        { $($generics:tt)* } { $($bounds:tt)* }
        $type:ty
    ) => {
        impl<C, $($generics)*> $crate::dev::Select<C> for $type
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            #[inline]
            fn select(
                params: $crate::dev::SelectionParams<'_, C, Self>,
                ctx: &mut C,
            ) -> Result<(), $crate::dev::Error> {
                $crate::dev::select_from_seed::<C, Self>(params, ctx)
            }
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
    //
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
    //
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
