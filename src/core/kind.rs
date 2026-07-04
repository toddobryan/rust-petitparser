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
    /// (e.g. `TabularDefinition`). Opaque to structural equality — see the note about opaque
    /// variants comparing equal-via-derive that still needs the `NeverEq` marker.
    Other(NeverEq),
}

#[derive(Clone, Debug)]
pub struct NeverEq;

impl PartialEq for NeverEq {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
