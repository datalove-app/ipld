use crate::dev::*;
use macros::derive_more::{AsRef, From, TryInto};
use std::rc::Rc;

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
        | Selector "selector"
    } representation keyed;
}

schema! {
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From, TryInto)]
    // #[from(forward)]
    #[try_into(owned, ref, ref_mut)]
    pub type Selector union {
        | Matcher "."

        #[ipld_attr(wrapper = "Rc")]
        | ExploreAll "a"

        | ExploreFields "f"

        #[ipld_attr(wrapper = "Rc")]
        | ExploreIndex "i"

        #[ipld_attr(wrapper = "Rc")]
        | ExploreRange "r"

        // #[ipld_attr(wrapper = "Rc")]
        // | ExploreRecursive "R"

        // | ExploreUnion "|"

        // #[ipld_attr(wrapper = "Rc")]
        // | ExploreConditional "&"

        // /// sentinel value; only valid in some positions.
        // | ExploreRecursiveEdge "@"

        // #[ipld_attr(wrapper = "Rc")]
        // | ExploreInterpretAs "~"
    } representation keyed;
}

schema! {
    /// ExploreAll is similar to a `*` -- it traverses all elements of an array,
    /// or all entries in a map, and applies a next selector to the reached
    /// nodes.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type ExploreAll struct {
        pub next Selector (rename ">"),
    };
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
    #[derive(Clone, Debug, From)]
    pub type ExploreFields struct {
        // fields {String:Selector} (rename "f>"),
    };
}

schema! {
    /// ExploreIndex traverses a specific index in a list, and applies a next
    /// selector to the reached node.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type ExploreIndex struct {
        pub index Int (rename "i"),
        pub next Selector (rename ">"),
    };
}

schema! {
    /// ExploreRange traverses a list, and for each element in the range
    /// specified, will apply a next selector to those reached nodes.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type ExploreRange struct {
        pub start Int (rename "^"),
        pub end Int (rename "$"),
        pub next Selector (rename ">"),
    };
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
    #[derive(Clone, Debug, From)]
    pub type ExploreRecursive struct {
        pub sequence Selector (rename ":>"),
        pub limit RecursionLimit (rename "l"),
        // /// if a node matches, we won't match it nor explore it's children
        pub stopAt optional Condition (rename "!"),
    };
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type RecursionLimit union {
        | RecursionLimit_None "none"
        | RecursionLimit_Depth "depth"
    } representation keyed;
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type RecursionLimit_None struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    #[from(forward)]
    pub type RecursionLimit_Depth int;
}

schema! {
    /// ExploreRecursiveEdge is a special sentinel value which is used to mark
    /// the end of a sequence started by an ExploreRecursive selector: the
    /// recursion goes back to the initial state of the earlier ExploreRecursive
    /// selector, and proceeds again (with a decremented maxDepth value).
    ///
    /// An ExploreRecursive selector that doesn't contain an
    /// ExploreRecursiveEdge is nonsensical.  Containing more than one
    /// ExploreRecursiveEdge is valid. An ExploreRecursiveEdge without an
    /// enclosing ExploreRecursive is an error.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type ExploreRecursiveEdge struct {};
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
    #[derive(Clone, Debug)]
    pub type ExploreUnion null;
    // TODO: pub type ExploreUnion [Selector];
}

schema! {
    /// Note that ExploreConditional versus a Matcher with a Condition are
    /// distinct: ExploreConditional progresses deeper into a tree; whereas a
    /// Matcher with a Condition may look deeper to make its decision, but
    /// returns a match for the node it's on rather any of the deeper values.
    #[ipld_attr(internal)]
    #[derive(Clone, Debug, From)]
    pub type ExploreConditional struct {
        pub condition Condition (rename "&"),
        pub next Selector (rename ">"),
    };
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
        pub r#as String (rename "c"),
        pub next Selector (rename ">"),
    };
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
    #[derive(Clone, Debug, From)]
    pub type Matcher struct {
        pub onlyIf optional Condition,
        pub label optional String,
    };
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
        | Condition_HasField "hasField"
        | Condition_HasValue "="
        | Condition_HasKind "%"
        | Condition_IsLink "/"
        | Condition_GreaterThan "greaterThan"
        | Condition_LessThan "lessThan"
        | Condition_And "and"
        | Condition_Or "or"
    } representation keyed;
}

schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_HasField struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_HasValue struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_HasKind struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_IsLink struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_GreaterThan struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_LessThan struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_And struct {};
}
schema! {
    ///
    #[ipld_attr(internal)]
    #[derive(Clone, Debug)]
    pub type Condition_Or struct {};
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
