/// Provides skeletons for conveniently implementing serde-compatibility for
/// IPLD types.
#[macro_export]
macro_rules! impl_ipld_serde {
    (@context_seed_visitor
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ty
        { $($visit_fns:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::Visitor<'de> for $crate::dev::ContextSeed<'a, C, $ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = ();

            $($visit_fns)*
        }
    };
    (@context_seed_visitor_ext
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ty
        { $($visit_fns:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::IpldVisitorExt<'de> for $crate::dev::ContextSeed<'a, C, $ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            $($visit_fns)*
        }
    };
    (@context_seed_deseed @default
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ty
    ) => {
        $crate::dev::impl_ipld_serde!(@context_seed_deseed
            { $($generics:tt)* } { $($bounds:tt)* }
            $ty:ty
        {
            #[inline]
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(self)
            }
        })
    };
    (@context_seed_deseed
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ty
        { $($deseed_fn:tt)* }
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::DeserializeSeed<'de> for $crate::dev::ContextSeed<'a, C, $ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = ();

            $($deseed_fn)*
        }
    };
    (@context_seed_select
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ty
    ) => {
        impl<C, $($generics)*> $crate::dev::Select<C> for $ty
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            #[inline]
            fn select(
                params: $crate::dev::SelectionParams<'_, C, Self>,
                ctx: &mut C,
            ) -> Result<(), $crate::dev::Error> {
                $crate::dev::ContextSeed::<'_, C, Self>::select(params, ctx)
            }
        }
    };

    // newtype impls

    (@context_seed_deseed_newtype
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ident as $inner_ty:ident
    ) => {
        impl<'a, 'de, C, $($generics)*> $crate::dev::DeserializeSeed<'de> for $crate::dev::ContextSeed<'a, C, $ty>
        where
            C: $crate::dev::Context,
            $($bounds)*
        {
            type Value = ();

            #[inline]
            fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: $crate::dev::Deserializer<'de>,
            {
                use $crate::dev::SelectionCallback::*;

                let (selector, state, callback, ctx) = self.into_parts();
                let callback = match callback {
                    SelectNode { cb, only_matched } => SelectNode { cb, only_matched },
                    SelectDag { mut cb } => SelectDag {
                        cb: Box::new(move |selection, ctx| {
                            let dag = selection.dag.downcast::<$inner_ty>()?;
                            let selection = $crate::dev::DagSelection {
                                dag: $ty(dag).into(),
                                ..selection
                            };
                            cb(selection, ctx)
                        }),
                    },
                    _ => unreachable!(),
                };

                $crate::dev::ContextSeed::<'_, C, $inner_ty>::from(selector, state, callback, ctx)
                    .deserialize(deserializer)
            }
        }
    };
    // TODO: deprecate
    (@select_newtype
        { $($generics:tt)* } { $($bounds:tt)* }
        $ty:ident as $inner_ty:ident
    ) => {
        /// Delegates directly to the inner type's [`Select`] implmentation,
        /// wrapping the provided callbacks to ensure the expected types are
        /// selected.
        ///
        /// [`Select`]: crate::dev::Select
        impl<C, $($generics)*> $crate::dev::Select<C> for $ty
        where
            C: $crate::dev::Context,
            $inner_ty: $crate::dev::Select<C>,
            $($bounds)*
        {
            fn select(
                params: $crate::dev::SelectionParams<'_, C, Self>,
                ctx: &mut C,
            ) -> Result<(), $crate::dev::Error> {
                use $crate::dev::SelectionCallback::*;

                let params = $crate::dev::SelectionParams::<'_, C, $inner_ty> {
                    cid: params.cid,
                    selector: params.selector,
                    max_path_depth: params.max_path_depth,
                    max_link_depth: params.max_link_depth,
                    callback: match params.callback {
                        SelectNode { cb, only_matched } => SelectNode { cb, only_matched },
                        SelectDag { mut cb } => SelectDag {
                            cb: Box::new(move |selection, ctx| {
                                let dag = selection.dag.downcast::<$inner_ty>()?;
                                let selection = $crate::dev::DagSelection {
                                    dag: Self(dag).into(),
                                    ..selection
                                };
                                cb(selection, ctx)
                            }),
                        },
                        _ => unreachable!(),
                    },
                };

                <$inner_ty>::select(params, ctx)
            }
        }
    };

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
}
