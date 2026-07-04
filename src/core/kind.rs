use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use crate::parser::{character::CharKind, repeater::separated::Trailing};

pub type PtrKey = *const ();

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserKind<'a> {
    And,
    Char {
        kind: &'a CharKind,
        message: Option<&'a str>,
    },
    CharacterRepeating {
        test: PtrKey,
        message: Option<&'a str>,
        min: usize,
        max: Option<usize>,
    },
    Choice {
        joiner: PtrKey,
    },
    Constant(NeverEq),
    Continuation(NeverEq),
    ElementsAt {
        indexes: &'a [i32],
    },
    EndOfInput {
        message: &'a str,
    },
    Epsilon(NeverEq),
    Failure {
        message: Option<&'a str>,
    },
    FlatMap(NeverEq),
    GreedyRepeating {
        min: usize,
        max: Option<usize>,
    },
    Input {
        message: Option<&'a str>,
    },
    Labeled {
        label: &'a str,
    },
    LazyRepeating {
        min: usize,
        max: Option<usize>,
    },
    Map(NeverEq),
    Newline,
    Not {
        message: &'a str,
    },
    OnlyIf {
        predicate: PtrKey,
        factory: Option<PtrKey>,
    },
    Pick {
        index: i32,
    },
    Position,
    PossessiveRepeating {
        min: usize,
        max: Option<usize>,
    },
    PredicateChar {
        test: PtrKey,
        message: Option<&'a str>,
    },
    Predicate {
        predicate: PtrKey,
        length: usize,
        message: &'a str,
    },
    Regex {
        pattern: &'a str,
        message: &'a str,
    },
    SeparatedListRepeating {
        min: usize,
        max: Option<usize>,
        trailing: Trailing,
    },
    Seq,
    /// The owning `SettableParser` (strong `Rc`). Distinct from `SettableRef` so
    /// `UnresolvedSettable` flags an unresolved rule once (at the owner) and so an owner and a
    /// `.borrow()` of it don't compare structurally equal (which would false-flag `DuplicateParser`).
    Settable,
    /// A `SettableParserRef` (weak back-reference). See `Settable`.
    SettableRef,
    Skip,
    Success(NeverEq),
    Token,
    Undefined {
        message: &'a str,
    },
    /// Catch-all for custom `HasChildren` implementors outside this crate's own parser set
    /// (e.g. `TabularDefinition`). The payload is the custom parser's own structural-equality
    /// hook: it lets a downstream parser opt into the linter's `DuplicateParser`/`RepeatedChoice`/
    /// etc. by defining how two of its kind compare. Use [`AlwaysDistinct`] to opt out (never
    /// equal). Only the parser's *own* identity (type + scalar props) needs comparing here — its
    /// children are compared separately by `structural_eq`'s recursion.
    Other(Rc<dyn CustomParserKind>),
}

/// The structural-equality hook a custom (out-of-crate) parser supplies via
/// [`ParserKind::Other`]. Implement it, then return `ParserKind::Other(Rc::new(YourKind))` from
/// your parser's `HasChildren::kind()`.
///
/// Compare against `other` by downcasting it (trait-upcast to `&dyn Any`, then `downcast_ref`) —
/// for a parser whose identity is entirely its children, that's just a type check:
/// `(other as &dyn std::any::Any).downcast_ref::<YourKind>().is_some()`.
pub trait CustomParserKind: Any + Debug {
    /// Whether `self` and `other` represent structurally-equal parsers (ignoring children, which
    /// are compared elsewhere). Should be symmetric — the implementor's responsibility.
    fn eq_custom(&self, other: &dyn CustomParserKind) -> bool;
}

// Makes `Rc<dyn CustomParserKind>: PartialEq`, so `ParserKind` keeps deriving `PartialEq`.
impl PartialEq for dyn CustomParserKind {
    fn eq(&self, other: &Self) -> bool {
        self.eq_custom(other)
    }
}

/// A ready-made [`CustomParserKind`] for custom parsers that don't want to participate in
/// structural equality: it never compares equal to anything (not even another `AlwaysDistinct`),
/// so the linter treats each such parser as unique.
#[derive(Debug)]
pub struct AlwaysDistinct;

impl CustomParserKind for AlwaysDistinct {
    fn eq_custom(&self, _: &dyn CustomParserKind) -> bool {
        false
    }
}

/// Marker used by opaque `ParserKind` variants (`Map`/`Constant`/`Epsilon`/…) whose payloads are
/// closures or type-erased values that can't be compared: it makes those variants compare unequal
/// under the derived `PartialEq` (conservative — never a false "equal").
#[derive(Clone, Debug)]
pub struct NeverEq;

impl PartialEq for NeverEq {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TypeCheckKind; // identity is just "same type" (like a children-only custom parser)

    impl CustomParserKind for TypeCheckKind {
        fn eq_custom(&self, other: &dyn CustomParserKind) -> bool {
            (other as &dyn Any)
                .downcast_ref::<TypeCheckKind>()
                .is_some()
        }
    }

    #[derive(Debug)]
    struct OtherTypeCheckKind;

    impl CustomParserKind for OtherTypeCheckKind {
        fn eq_custom(&self, other: &dyn CustomParserKind) -> bool {
            (other as &dyn Any)
                .downcast_ref::<OtherTypeCheckKind>()
                .is_some()
        }
    }

    fn other(k: impl CustomParserKind + 'static) -> ParserKind<'static> {
        ParserKind::Other(Rc::new(k))
    }

    #[test]
    fn type_check_kind_equals_same_type() {
        assert_eq!(other(TypeCheckKind), other(TypeCheckKind));
    }

    #[test]
    fn type_check_kind_differs_from_other_type() {
        assert_ne!(other(TypeCheckKind), other(OtherTypeCheckKind));
    }

    #[test]
    fn always_distinct_never_equal_even_to_itself() {
        assert_ne!(other(AlwaysDistinct), other(AlwaysDistinct));
    }

    #[test]
    fn other_never_equals_a_builtin_variant() {
        assert_ne!(other(TypeCheckKind), ParserKind::Token);
    }
}
