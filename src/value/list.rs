use crate::dev::*;

///
pub type List<T = Value> = Vec<T>;

// TODO: write the 4 Select impls, then the latter 3 for Vec<Link<T>>

impl<T: Representation> Representation for List<T> {
    const NAME: &'static str = "List";
    // const SCHEMA: &'static str = format!("type {} = &{}", Self::NAME, T::NAME);
    const KIND: Kind = Kind::List;
}

// // TODO: add the rest of the selectors
// impl_root_select!(Matcher, ExploreAll, ExploreIndex, ExploreRange, {
//     default impl<Ctx, T> Select<Selector, Ctx> for Vec<T>
//     where
//         Ctx: Context,
//         T: Representation + 'static
// });
