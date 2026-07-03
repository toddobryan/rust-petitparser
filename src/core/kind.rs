use crate::parser::{character::CharKind, repeater::separated::Trailing};

pub type PtrKey = *const ();

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserKind {
    And,
    Char {
        kind: CharKind,
        message: Option<String>,
    },
    CharacterRepeating {
        test: PtrKey,
        message: Option<String>,
        min: usize,
        max: Option<usize>,
    },
    Choice {
        joiner: PtrKey,
    },
    Constant(NeverEq),
    Continuation(NeverEq),
    ElementsAt {
        indexes: Vec<i32>,
    },
    EndOfInput {
        message: String,
    },
    Epsilon(NeverEq),
    Failure {
        message: Option<String>,
    },
    FlatMap(NeverEq),
    GreedyRepeating {
        min: usize,
        max: Option<usize>,
    },
    Input {
        message: Option<String>,
    },
    Labeled {
        label: String,
    },
    LazyRepeating {
        min: usize,
        max: Option<usize>,
    },
    Map(NeverEq),
    Newline,
    Not {
        message: String,
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
        message: Option<String>,
    },
    Predicate {
        predicate: PtrKey,
        length: usize,
        message: String,
    },
    Regex {
        pattern: String,
        message: String,
    },
    SeparatedListRepeating {
        min: usize,
        max: Option<usize>,
        trailing: Trailing,
    },
    Seq,
    Settable,
    Skip,
    Success(NeverEq),
    Token,
    Undefined {
        message: String,
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
