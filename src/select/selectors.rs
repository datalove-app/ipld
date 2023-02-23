use crate::dev::*;
use macros::derive_more::{AsMut, AsRef, From, TryInto};
use maybestd::{
    boxed::Box,
    fmt,
    ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive},
    rc::Rc,
    str::FromStr,
};

///
#[cfg_attr(feature = "dev", doc(hidden))]
pub static DEFAULT_SELECTOR: Selector = Selector::DEFAULT;

schema! {
    /// SelectorEnvelope is the recommended top-level value for serialized
    /// messages that don't have established existing context with marks the
    /// start of a selector: it's a single-member union used to kick us towards
    /// "nominative typing".
    ///
    /// See https://ipld.io/docs/schemas/using/migrations/ for a background on
    /// the theory behind this gentle-nominative concept.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From, TryInto)]
    pub type SelectorEnvelope union {
        ///
        | Selector "selector"
    } representation keyed
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From, TryInto)]
    // #[from(forward)]
    #[try_into(owned, ref, ref_mut)]
    pub type Selector union {
        ///
        | Matcher "."
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreAll "a"
        ///
        | ExploreFields "f"
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreIndex "i"
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreRange "r"
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreRecursive "R"
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreUnion "|"
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreConditional "&"
        ///
        | ExploreRecursiveEdge "@"
        ///
        #[ipld_attr(wrapper = "Box")]
        | ExploreInterpretAs "~"
    } representation keyed
}

schema! {
    /// ExploreAll is similar to a `*` -- it traverses all elements of an array,
    /// or all entries in a map, and applies a next selector to the reached
    /// nodes.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default, From)]
    pub type ExploreAll struct {
        next Selector (rename ">")
    }
}

schema! {
    /// ExploreFields traverses named fields in a map (or equivalently, struct,
    /// if traversing on typed/schema nodes) and applies a next selector to the
    /// reached nodes.
    ///
    /// Note that a concept of exploring a whole path (e.g. "foo/bar/baz") can
    /// be represented as a set of three nexted ExploreFields selectors, each
    /// specifying one field.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default, From)]
    pub type ExploreFields struct {
        // fields {String:Selector} (rename "f>"),
    }
}

schema! {
    /// ExploreIndex traverses a specific index in a list, and applies a next
    /// selector to the reached node.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default)]
    pub type ExploreIndex struct {
        index Int (rename "i")
        next Selector (rename ">")
    }
}

schema! {
    /// ExploreRange traverses a list, and for each element in the range
    /// specified, will apply a next selector to those reached nodes.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default)]
    pub type ExploreRange struct {
        start Int (rename "^")
        end Int (rename "$")
        next Selector (rename ">")
    }
}

schema! {
    /// ExploreRecursive traverses some structure recursively. To guide this
    /// exploration, it uses a "sequence", which is another Selector tree; some
    /// leaf node in this sequence should contain an ExploreRecursiveEdge
    /// selector, which denotes the place recursion should occur.
    ///
    /// In implementation, whenever evaluation reaches an ExploreRecursiveEdge
    /// marker in the recursion sequence's Selector tree, the implementation
    /// logically produces another new Selector which is a copy of the original
    /// ExploreRecursive selector, but with a decremented depth parameter for
    /// limit (if limit is of type depth), and continues evaluation thusly.
    ///
    /// It is not valid for an ExploreRecursive selector's sequence to contain
    /// no instances of ExploreRecursiveEdge; it *is* valid for it to contain
    /// more than one ExploreRecursiveEdge.
    ///
    /// ExploreRecursive can contain a nested ExploreRecursive! This is
    /// comparable to a nested for-loop. In these cases, any
    /// ExploreRecursiveEdge instance always refers to the nearest parent
    /// ExploreRecursive (in other words, ExploreRecursiveEdge can be thought of
    /// like the 'continue' statement, or end of a for-loop body; it is *not* a
    /// 'goto' statement).
    ///
    /// Be careful when using ExploreRecursive with a large depth limit
    /// parameter; it can easily cause very large traversals (especially if used
    /// in combination with selectors like ExploreAll inside the sequence).
    ///
    /// limit is a union type -- it can have an integer depth value (key
    /// "depth") or no value (key "none"). If limit has no value it is up to the
    /// implementation library using selectors to identify an appropriate max
    /// depth as necessary so that recursion is not infinite.
    ///
    /// stopAt specifies a Condition that stops the traversal when it is
    /// fulfilled. If throughout the traversal the selector encounters a node
    /// that matches Condition it will finish exploring the current node and it
    /// won't recurse more, stopping the traversal immediately. If Condition is
    /// never matched, the selector performs the traversal seamlessly until the
    /// end. This feature is of particular interest for applications that need
    /// to recurse a large linked structure up to a specific point. stopAt can
    /// be used to let the selector know where to stop recursing preventing from
    /// having to traverse the full structure.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default)]
    pub type ExploreRecursive struct {
        ///
        pub sequence Selector (rename ":>")
        // ///
        // pub limit RecursionLimit (rename "l")
        /// if a node matches, we won't match it nor explore it's children
        pub stopAt optional Condition (rename "!")
    }
}

// schema! {
//     ///
//     #[ipld_attr(internal)]
//     #[derive(Clone, Debug, From)]
//     pub type RecursionLimit union {
//         ///
//         | RecursionLimit_None "none"
//         ///
//         | RecursionLimit_Depth "depth"
//     } representation keyed
// }
// schema! {
//     ///
//     #[ipld_attr(internal)]
//     #[derive(Clone, Debug, Default)]
//     pub type RecursionLimit_None struct {}
// }
// schema! {
//     ///
//     #[ipld_attr(internal)]
//     #[derive(Clone, Debug, Default, From)]
//     #[from(forward)]
//     pub type RecursionLimit_Depth int
// }

schema! {
    /// ExploreRecursiveEdge is a special sentinel value which is used to mark
    /// the end of a sequence started by an ExploreRecursive selector: the
    /// recursion goes back to the initial state of the earlier ExploreRecursive
    /// selector, and proceeds again (with a decremented maxDepth value).
    ///
    /// An ExploreRecursive selector that doesn't contain an
    /// ExploreRecursiveEdge is nonsensical. Containing more than one
    /// ExploreRecursiveEdge is valid. An ExploreRecursiveEdge without an
    /// enclosing ExploreRecursive is an error.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default)]
    pub type ExploreRecursiveEdge struct {}
}

schema! {
    /// ExploreUnion allows selection to continue with two or more distinct
    /// selectors while exploring the same tree of data.
    ///
    /// ExploreUnion can be used to apply a Matcher on one node (causing it to
    /// be considered part of a (possibly labelled) result set), while
    /// simultaneously continuing to explore deeper parts of the tree with
    /// another selector, for example.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default,
        // From
    )]
    pub type ExploreUnion null
    // pub type ExploreUnion [Selector]
}

schema! {
    /// Note that ExploreConditional versus a Matcher with a Condition are
    /// distinct: ExploreConditional progresses deeper into a tree; whereas a
    /// Matcher with a Condition may look deeper to make its decision, but
    /// returns a match for the node it's on rather any of the deeper values.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type ExploreConditional struct {
        condition Condition (rename "&")
        next Selector (rename ">")
    }
}

schema! {
    /// ExploreInterpretAs is a transformation that attempts to 'reify' the
    /// current node using an ADL specified by 'as'. ADLs are recognized by
    /// agreed-upon strings, similar to libp2p protocols. The ExploreInterpretAs
    /// reification process may introduce a data-dependant amount of budget on
    /// evaluation based on the specific traversal and ADL implementation.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type ExploreInterpretAs struct {
        r#as String
        next Selector (rename ">")
    }
}

schema! {
    /// Slice is a predicate that selects only a subset of node.
    /// This is applicable primarily in the context of reified nodes based on the
    /// InterpetAs clause above, where the primitive (bytes or string) node is actually
    /// composed from multiple underlying substrate nodes.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default)]
    pub type Slice struct {
        from Int (rename "[")
        to Int (rename "]")
    }
}

schema! {
    /// Matcher marks a node to be included in the "result" set. (All nodes
    /// traversed by a selector are in the "covered" set (which is a.k.a. "the
    /// merkle proof"); the "result" set is a subset of the "covered" set.)
    ///
    /// In libraries using selectors, the "result" set is typically provided to
    /// some user-specified callback.
    ///
    /// A selector tree with only "explore*"-type selectors and no Matcher
    /// selectors is valid; it will just generate a "covered" set of nodes and
    /// no "result" set.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, Default, From)]
    pub type Matcher struct {
        /// match is true based on position alone if this is not set.
        pub onlyIf optional Condition
        /// labels can be used to match multiple different structures in one selection.
        pub label optional String
        /// if set, only the subset of the node specified by the slice is matched.
        pub subset optional Slice

    }
}

schema! {
    /// Condition is expresses a predicate with a boolean result.
    ///
    /// Condition clauses are used several places:
    ///   - in Matcher, to determine if a node is selected.
    ///   - in ExploreRecursive, to halt exploration.
    ///   - in ExploreConditional,
    ///
    /// TODO -- Condition is very skeletal and incomplete.
    /// The place where Condition appears in other structs is correct; the rest
    /// of the details inside it are not final nor even completely drafted.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type Condition union {
        ///
        | Condition_HasField "hasField"
        /// # will need to contain a kinded union, lol.  these conditions are gonna get deep.)
        | Condition_HasValue "="
        /// will ideally want to refer to the DataModel ReprKind enum...!  will
        /// we replicate that here?  don't want to block on cross-schema
        /// references, but it's interesting that we've finally found a good
        /// example wanting it.
        | Condition_HasKind "%"
        /// will need this so we can use it in recursions to say "stop at CID QmFoo".
        | Condition_IsLink "/"
        ///
        | Condition_GreaterThan "greaterThan"
        ///
        | Condition_LessThan "lessThan"
        ///
        | Condition_And "and"
        ///
        | Condition_Or "or"
    } representation keyed
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_HasField struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_HasValue struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_HasKind struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_IsLink struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_GreaterThan struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_LessThan struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_And struct {}
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_Or struct {}
}

/// Sealed marker trait for types that can be used as `Selector`s.
#[doc(hidden)]
pub trait ISelector: Representation + private::Sealed {
    ///
    fn inner_selector(&self) -> Option<&Selector> {
        None
    }

    /// TODO:
    fn inner_selector_for<F: AsRef<str>>(&self, field: F) -> Option<&Selector> {
        self.inner_selector()
    }

    /// TODO:
    fn inner_selector_at<I: Into<usize>>(&self, index: I) -> Option<&Selector> {
        self.inner_selector()
    }
}

mod private {
    use crate::dev::*;

    /// Sealed marker trait for `Selector` types.
    #[doc(hidden)]
    pub trait Sealed {}

    // impl Sealed for Selector {}
    impl Sealed for Matcher {}
    impl Sealed for ExploreAll {}
    impl Sealed for ExploreFields {}
    impl Sealed for ExploreIndex {}
    impl Sealed for ExploreRange {}
    impl Sealed for ExploreRecursive {}
    impl Sealed for ExploreUnion {}
    impl Sealed for ExploreConditional {}
    impl Sealed for ExploreRecursiveEdge {}
}

macro_rules! impl_variant {
    ($variant:ident $selector_ty:ty |
        { $is:ident, $as:ident, $try_as:ident, $try_into:ident }) => {
        impl_variant!(@is $is -> $variant $selector_ty);
        impl_variant!(@as $as -> $variant $selector_ty);
        impl_variant!(@try_as $try_as -> $variant $selector_ty);
        // impl_variant!(@try_into $try_into -> $variant $selector_ty);
    };
    (@wrapped $variant:ident $selector_ty:ty |
        { $is:ident, $as:ident, $try_as:ident, $try_into:ident }) => {
        impl_variant!(@is $is -> $variant $selector_ty);
        impl_variant!(@as $as -> $variant $selector_ty);
        // impl_variant!(@try_into $try_into -> $variant Rc<$selector_ty>);
    };

    (@is $fn:ident -> $variant:ident $selector_ty:ty) => {
        #[inline]
        pub const fn $fn(&self) -> bool {
            match self {
                Self::$variant(..) => true,
                _ => false,
            }
        }
    };
    (@as $fn:ident -> $variant:ident $selector_ty:ty) => {
        #[inline]
        pub fn $fn(&self) -> Option<&$selector_ty> {
            match self {
                Self::$variant(inner) => Some(inner),
                _ => None,
            }
        }
    };
    (@try_as $fn:ident -> $variant:ident $selector_ty:ty) => {
        #[inline]
        pub fn $fn(&self) -> Result<&$selector_ty, Error> {
            match self {
                Self::$variant(inner) => Ok(inner),
                _ => Err(Error::SelectorAssertionFailure),
            }
        }
    };
    // (@try_into $fn:ident -> $variant:ident $selector_ty:ty) => {
    //     #[inline]
    //     pub fn $fn(&self) -> Result<&$selector_ty, Error> {
    //         match self {
    //             Self::$variant(inner) => Ok(inner),
    //             _ => Err(Error::SelectorAssertionFailure),
    //         }
    //     }
    // };
}

impl Selector {
    // TODO: handle ExploreFields and ExploreUnion iteratively, directly delegate the rest
    // fn new<Ctx, T, S>(args: T::SelectorArgs) -> Self
    // where
    //     Ctx: Context, // TODO FromContext?
    //     // S: ISelector,
    //     T: Select<Ctx, S>,
    //     Self: From<S>,
    // {
    //     Self::from(T::new_selector(args))
    // }
    //
    // fn select<Ctx, T, S>(&self, T) -> Result<S::Output, ()>
    // where
    //     Ctx: Context,
    //     S: ISelector,
    //     T: Select<Ctx, S>,
    // {
    //     match self {
    //         // Self::Matcher
    //         // Self::ExploreAll
    //         // Self::ExploreFields
    //         // Self::ExploreIndex
    //         // Self::ExploreRange
    //         // Self::ExploreRecursive
    //         // Self::ExploreUnion
    //         // Self::ExploreConditional
    //         // Self::ExploreRecursiveEdge
    //     }
    // }

    ///
    pub const DEFAULT: Self = Self::Matcher(Matcher {
        onlyIf: None,
        label: None,
        subset: None,
    });

    ///
    pub const fn code(&self) -> char {
        match self {
            Self::Matcher(_) => Matcher::CODE,
            Self::ExploreAll(_) => ExploreAll::CODE,
            Self::ExploreFields(_) => ExploreFields::CODE,
            Self::ExploreIndex(_) => ExploreIndex::CODE,
            Self::ExploreRange(_) => ExploreRange::CODE,
            Self::ExploreRecursive(_) => ExploreRecursive::CODE,
            Self::ExploreUnion(_) => ExploreUnion::CODE,
            Self::ExploreConditional(_) => ExploreConditional::CODE,
            Self::ExploreRecursiveEdge(_) => ExploreRecursiveEdge::CODE,
            Self::ExploreInterpretAs(_) => ExploreInterpretAs::CODE,
        }
    }

    /// Attempts to produce the next selector to apply, given an optional field
    /// (key or index).
    /// TODO: matcher is infinite; need to distinguish link boundaries
    /// TODO: should return Option<&Selector>
    pub fn next<'a>(&self, field: Option<&Field<'_>>) -> Option<&Selector> {
        match (self, field) {
            (Self::Matcher(_), _) => Some(self),
            (Self::ExploreAll(inner), _) => Some(&inner.next),
            // TODO assert that provided field/index matches what the selector defines, otherwise return None
            (Self::ExploreFields { .. }, Some(f)) => todo!(),
            (Self::ExploreIndex(inner), Some(f)) if f.is_idx(inner.index as usize) => {
                Some(&inner.next)
            }
            (Self::ExploreRange(inner), Some(f))
                if f.as_usize()
                    .filter(|idx| inner.contains(&(*idx as Int)))
                    .is_some() =>
            {
                Some(&inner.next)
            }
            (Self::ExploreRecursive(inner), _) => Some(&inner.sequence),
            (Self::ExploreRecursiveEdge(_), _) => todo!(),
            (Self::ExploreUnion { .. }, _) => todo!(),
            (Self::ExploreInterpretAs(inner), _) => Some(&inner.next),
            _ => None,
        }
    }

    impl_variant!(Matcher Matcher |
        {is_matcher, as_matcher, try_as_matcher, try_into_matcher});
    impl_variant!(@wrapped ExploreAll ExploreAll |
        {is_explore_all, as_explore_all, try_as_explore_all, try_into_explore_all});
    impl_variant!(ExploreFields ExploreFields |
        {is_explore_fields, as_explore_fields, try_as_explore_fields, try_into_explore_fields});
    impl_variant!(@wrapped ExploreIndex ExploreIndex |
        {is_explore_index, as_explore_index, try_as_explore_index, try_into_explore_index});
    impl_variant!(@wrapped ExploreRange ExploreRange |
        {is_explore_range, as_explore_range, try_as_explore_range, try_into_explore_range});
    // impl_variant!(ExploreRecursive ExploreRecursive |
    //     {is_explore_recursive, as_explore_recursive, try_into_explore_recursive});
    impl_variant!(ExploreUnion ExploreUnion |
        {is_explore_union, as_explore_union, try_as_explore_union, try_into_explore_union});
    // impl_variant!(ExploreConditional ExploreConditional |
    //     {is_explore_conditional, as_explore_conditional, try_into_explore_conditional);
    // impl_variant!(ExploreInterpretAs ExploreInterpretAs |
    //     {is_explore_interpret_as, as_explore_interpret_as, try_into_explore_interpret_as});
    // impl_variant!(ExploreRecursiveEdge ExploreRecursiveEdge |
    //     {is_explore_recursive_edge, as_explore_recursive_edge, try_into_explore_recursive_edge});
}

/* Selector */

impl Default for Selector {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// TODO for all
impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl FromStr for Selector {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl Matcher {
    ///
    pub const CODE: char = '.';
}

/* ExploreAll */

impl ExploreAll {
    ///
    pub const CODE: char = 'a';

    ///
    pub const fn to_range(&self) -> RangeFrom<usize> {
        0usize..
    }
}

impl RangeBounds<Int> for ExploreAll {
    fn start_bound(&self) -> Bound<&Int> {
        Bound::Included(&0)
    }
    fn end_bound(&self) -> Bound<&Int> {
        Bound::Unbounded
    }
}

/* ExploreFields */

impl ExploreFields {
    ///
    pub const CODE: char = 'f';

    ///
    pub fn contains_key(&self, key: &str) -> bool {
        unimplemented!()
    }
}

/* ExploreIndex */

impl ExploreIndex {
    ///
    pub const CODE: char = 'i';

    ///
    pub const fn index(&self) -> Int {
        self.index
    }

    ///
    pub const fn to_range(&self) -> RangeInclusive<usize> {
        self.index as usize..=self.index as usize
    }
}

impl RangeBounds<Int> for ExploreIndex {
    fn start_bound(&self) -> Bound<&Int> {
        Bound::Included(&self.index)
    }
    fn end_bound(&self) -> Bound<&Int> {
        Bound::Included(&self.index)
    }
}

/* ExploreRange */

impl ExploreRange {
    ///
    pub const CODE: char = 'r';

    ///
    pub const fn start(&self) -> Int {
        self.start
    }

    ///
    pub const fn end(&self) -> Int {
        self.end
    }

    ///
    pub const fn to_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl RangeBounds<Int> for ExploreRange {
    fn start_bound(&self) -> Bound<&Int> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> Bound<&Int> {
        Bound::Excluded(&self.end)
    }
}

/* ExploreRecursive */

impl ExploreRecursive {
    ///
    pub const CODE: char = 'R';
}

// impl Default for RecursionLimit {
//     fn default() -> Self {
//         Self::RecursionLimit_None(RecursionLimit_None {})
//     }
// }

/* ExploreUnion */

impl ExploreUnion {
    ///
    pub const CODE: char = '|';

    /// Validates that the first [`Selector`] is a matcher.
    pub fn matches_first(&self) -> bool {
        // self.0[0].is_matcher()
        unimplemented!()
    }

    /// Asserts that the first [`Selector`] is a matcher.
    pub fn assert_matches_first<T: Representation>(&self) -> Result<(), Error> {
        if self.matches_first() {
            Ok(())
        } else {
            // Err(Error::unsupported_selector::<T>(self.0[0]))
            unimplemented!()
        }
    }

    /// Validates that all selectors to perform .
    pub fn all_disjoint(&self) -> bool {
        // self.0.iter().all(|s| s.code() == code)
        unimplemented!()
    }

    // pub fn to_ranges(&self) -> impl Iterator<Item = Range<usize>> {
    //     unimplemented!()
    // }
}

/* ExploreConditional */

impl ExploreConditional {
    ///
    pub const CODE: char = '&';
}

/* ExploreRecursiveEdge */

impl ExploreRecursiveEdge {
    ///
    pub const CODE: char = '@';
}

/* ExploreInterpretAs */

impl ExploreInterpretAs {
    ///
    pub const CODE: char = '~';
}

/* Slice */

impl Slice {
    ///
    pub const fn to_range(&self) -> Range<usize> {
        self.from as usize..self.to as usize
    }
}

impl RangeBounds<Int> for Slice {
    fn start_bound(&self) -> Bound<&Int> {
        Bound::Included(&self.from)
    }
    fn end_bound(&self) -> Bound<&Int> {
        Bound::Excluded(&self.to)
    }
}

impl From<Range<Int>> for Slice {
    fn from(range: Range<Int>) -> Self {
        Self {
            from: range.start,
            to: range.end,
        }
    }
}
