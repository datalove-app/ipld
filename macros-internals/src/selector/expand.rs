use super::*;
use crate::dev::*;
use quote::{quote, ToTokens, TokenStreamExt};

impl ToTokens for RootSelectorDefinition {
    // TODO:
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let def = &self.def;
        let root_type = &self.root_type;

        let imports = if self.internal {
            quote!(
                use crate as _ipld;
            )
        } else {
            quote!(
                extern crate ipld as _ipld;
            )
        };

        tokens.append_all(quote! {{
            #imports
            #[allow(unused_imports)]
            use _ipld::dev::*;

            let selector = #def;
            assert!(<#root_type as Select<Selector>>::validate(&selector));
            selector
            // Selector::new::<#root_type>(selector)
        }});
    }
}

impl ToTokens for SelectorDefinition {
    /// Expands into a `Selector` instance.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            // Matcher
            Self::Matcher(label) => {
                let label = label
                    .as_ref()
                    .map_or(quote!(None), |l| quote!(Some(String::from(#l))));

                quote! {
                    Selector::from(Matcher::from(#label))
                }
            }
            // ExploreAll
            Self::ExploreAll(sel) => quote! {
                Selector::from(Box::new(ExploreAll::from(#sel)))
            },
            // ExploreFields TODO:
            Self::ExploreFields(sels) => unimplemented!(),
            // Self::ExploreFields(sels) => {
            //     let len = sels.len();
            //     let sels = sels.iter().map(|(k, v)| quote!(map.insert(#k, #v);));

            //     quote! {
            //         Selector::from(ExploreFields::from({
            //             let mut map = std::collections::HashMap::with_capacity(#len);
            //             #(#sels)*
            //             map
            //         })
            //     }
            // }
            // ExploreIndex
            Self::ExploreIndex { index, def } => unimplemented!(),
            // ExploreRange
            Self::ExploreRange { start, end, def } => unimplemented!(),
            // ExploreRecursive
            Self::ExploreRecursive { sequence, limit } => {
                let limit = limit.as_ref().map_or(
                    quote!(RecursionLimit_None),
                    |l| quote!(RecursionLimit_Depth::from(#l)),
                );

                quote! {
                    Selector::from(ExploreRecursive {
                        sequence: #sequence,
                        limit: #limit.into(),
                    })
                }
            }
            // ExploreUnion
            Self::ExploreUnion(sels) => unimplemented!(),
            // Self::ExploreUnion(sels) => {
            //     let len = sels.len();
            //     let sels = sels.iter();

            //     quote! {
            //         Selector::from(ExploreUnion::from(vec![#(#sels)*])
            //     }
            // }
            // TODO ExploreConditional
            Self::ExploreConditional { def } => unimplemented!(),
            // ExploreRecursiveEdge
            Self::ExploreRecursiveEdge => quote!(Selector::from(ExploreRecursiveEdge)),
            // ?? if interpolated is a SelectorDefinition, match variant and assert_impls!(T: Select<Ctx, variant>)
            _ => quote! {},
        });
    }
}

// impl ToTokens for InterpolatedSelector {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         match self {
//             Self::Expected(t) => t.to_tokens(tokens),
//             Self::Ident(ident) => tokens.append_all(quote! {
//                 SelectorArgs::Selector()
//             }),
//         }
//     }
// }

impl<T> ToTokens for Interpolated<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Expected(t) => t.to_tokens(tokens),
            Self::Ident(ident) => ident.to_tokens(tokens),
        }
    }
}
