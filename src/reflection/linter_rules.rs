use std::rc::Rc;

use crate::{
    core::{kind::ParserKind, parser::HasChildren},
    reflection::{
        analyzer::{Analyzer, ptr},
        equality::{is_parser_iterable_equal, parsers_equal},
        formatting::format_iterable,
        linter::{LinterIssue, LinterRule, LinterType},
        path::ParserPath,
    },
};

pub const ALL_LINTER_RULES: &[&dyn LinterRule] = &[
    &CharacterRepeater,
    &DuplicateParser,
    &LeftRecursion,
    &NestedChoice,
    &NullableRepeater,
    &OverlappingChoice,
    &RepeatedChoice,
    &UnnecessaryInput,
    &UnoptimizedInput,
    &UnreachableChoice,
    &UnresolvedSettable,
    &UnusedResult,
];

pub struct CharacterRepeater;

impl LinterRule for CharacterRepeater {
    fn severity(&self) -> LinterType {
        LinterType::Warning
    }

    fn title(&self) -> &str {
        "Character repeater"
    }

    fn run(
        &self,
        _analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Input { .. }) {
            let repeating = parser.children()[0].clone();
            if matches!(repeating.kind(), ParserKind::PossessiveRepeating { .. }) {
                let character = repeating.children()[0].clone();
                if matches!(
                    character.kind(),
                    ParserKind::Char { .. } | ParserKind::PredicateChar { .. }
                ) {
                    issues.push(LinterIssue {
                        title: self.title().to_string(),
                        severity: self.severity(),
                        description: format!(
                            "A flattened repeater {:?} that delegates to a character \
                                parser {:?} can be much more efficiently implemented \
                                using `star_string`, `plus_string`, `times_string`, or \
                                `repeat_string` that directly returns the underlying String \
                                instead of an intermediate Vector.",
                            repeating, character,
                        ),
                        parser: parser.clone(),
                    });
                }
            }
        }
    }
}

pub struct DuplicateParser;

impl LinterRule for DuplicateParser {
    fn severity(&self) -> LinterType {
        LinterType::Info
    }

    fn title(&self) -> &str {
        "Duplicate parser"
    }

    fn run(
        &self,
        analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        let duplicates: Vec<&Rc<dyn HasChildren>> = analyzer
            .parsers
            .iter()
            .filter(|p2| parsers_equal(parser, p2))
            .collect();
        if duplicates.len() > 1 && ptr(duplicates[0]) == ptr(parser) {
            issues.push(LinterIssue {
                title: self.title().to_string(),
                severity: self.severity(),
                description: format!(
                    "{} instances of the parser {:?} exist in this \
                        grammar. If possible, reuse the same parser instances to reduce \
                        memory footprint and increase performance.",
                    duplicates.len(),
                    parser.clone()
                ),
                parser: parser.clone(),
            });
        }
    }
}

pub struct LeftRecursion;

impl LinterRule for LeftRecursion {
    fn severity(&self) -> LinterType {
        LinterType::Error
    }

    fn title(&self) -> &str {
        "Left recursion"
    }

    fn run(
        &self,
        analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if !analyzer.cycle_set(parser).is_empty() {
            issues.push(LinterIssue {
                title: self.title().to_string(),
                severity: self.severity(),
                parser: parser.clone(),
                description: format!(
                    "The parser directly or indirectly refers to itself without \
                              consuming input:\n{}\nThis causes an infinite loop when parsing.",
                    format_iterable(analyzer.cycle_set(parser), Some(1))
                ),
            });
        }
    }
}

pub struct NestedChoice;

impl LinterRule for NestedChoice {
    fn severity(&self) -> LinterType {
        LinterType::Info
    }

    fn title(&self) -> &str {
        "Nested choice"
    }

    fn run(
        &self,
        _analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Choice { .. }) {
            let length = parser.children().len();
            for (i, child) in parser.children().iter().enumerate() {
                if i < length - 1 && matches!(child.kind(), ParserKind::Choice { .. }) {
                    issues.push(LinterIssue {
                        title: self.title().to_string(),
                        severity: self.severity(),
                        description: format!(
                            "The choice at index {} is another choice {:?} that adds \
                                unnecessary overhead that can be avoided by flattening it into \
                                the parent",
                            i, child,
                        ),
                        parser: parser.clone(),
                    });
                }
            }
        }
    }
}

pub struct NullableRepeater;

impl LinterRule for NullableRepeater {
    fn severity(&self) -> LinterType {
        LinterType::Error
    }

    fn title(&self) -> &str {
        "Nullable repeater"
    }

    fn run(
        &self,
        analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(
            parser.kind(),
            ParserKind::PossessiveRepeating { .. }
                | ParserKind::GreedyRepeating { .. }
                | ParserKind::LazyRepeating { .. }
                | ParserKind::SeparatedListRepeating { .. }
        ) && parser
            .children()
            .first()
            .is_some_and(|p| analyzer.is_nullable(p))
        {
            if matches!(parser.kind(), ParserKind::SeparatedListRepeating { .. })
                && let Some(sep) = parser.children().get(1)
                && !analyzer.is_nullable(sep)
            {
                return;
            }

            issues.push(LinterIssue {
                title: self.title().to_string(),
                severity: self.severity(),
                parser: parser.clone(),
                description: "A repeater that delegates to a nullable parser causes an infinite \
                              loop when parsing."
                    .to_string(),
            });
        }
    }
}

pub struct OverlappingChoice;

impl LinterRule for OverlappingChoice {
    fn severity(&self) -> LinterType {
        LinterType::Info
    }

    fn title(&self) -> &str {
        "Overlapping choice"
    }

    fn run(
        &self,
        analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Choice { .. }) {
            let children = parser.children();
            for (i, child_i) in children.iter().enumerate() {
                let first_i = analyzer.first_set(child_i);
                for (j, child_j) in children.iter().enumerate().skip(i + 1) {
                    let first_j = analyzer.first_set(child_j);
                    if is_parser_iterable_equal(&first_i, &first_j) {
                        issues.push(LinterIssue {
                            title: self.title().to_string(),
                            severity: self.severity(),
                            description: format!(
                                "The choices at index {i} and {j} have overlapping first-sets, \
                                    which can be an indication of an inefficient grammar:\n
                                    {}\n
                                    If possible, try extracting common prefixes from choices.",
                                format_iterable(&first_i, None)
                            ),
                            parser: parser.clone(),
                        });
                    }
                }
            }
        }
    }
}

pub struct RepeatedChoice;

impl LinterRule for RepeatedChoice {
    fn severity(&self) -> LinterType {
        LinterType::Warning
    }

    fn title(&self) -> &str {
        "Repeated choice"
    }

    fn run(
        &self,
        _analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Choice { .. }) {
            let children = parser.children();
            for i in 0..children.len() {
                for j in (i + 1)..children.len() {
                    if parsers_equal(&children[i], &children[j]) {
                        issues.push(LinterIssue {
                            title: self.title().to_string(),
                            severity: self.severity(),
                            description: format!(
                                "The choices at indexes {} and {} are identical:\n \
                            {}: {:?}\n \
                            {}: {:?}\n\
                            The second choice can never succeed and can therefore be \
                            removed.",
                                i,
                                j,
                                i,
                                children[i].clone(),
                                j,
                                children[j].clone()
                            ),
                            parser: parser.clone(),
                        });
                    }
                }
            }
        }
    }
}

/// Our name for Dart's UnnecessaryFlatten
pub struct UnnecessaryInput;

impl LinterRule for UnnecessaryInput {
    fn title(&self) -> &str {
        "Unnecessary input"
    }

    fn severity(&self) -> LinterType {
        LinterType::Warning
    }

    fn run(
        &self,
        _analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Input { .. })
            && matches!(parser.kind(), ParserKind::Input { message: None })
            && let Some(delegate) = parser.children().first()
            && (matches!(
                delegate.kind(),
                ParserKind::Char { .. } | ParserKind::PredicateChar { .. }
            ) || matches!(delegate.kind(), ParserKind::Input { .. })
                || matches!(delegate.kind(), ParserKind::Newline)
                || matches!(delegate.kind(), ParserKind::Predicate { .. })
                || matches!(delegate.kind(), ParserKind::CharacterRepeating { .. }))
        {
            issues.push(LinterIssue {
                title: self.title().to_string(),
                severity: self.severity(),
                description: format!(
                    "A flatten parser delegating to a parser ({:?}) that is \
                        returning the accepted input string adds unnecessary overhead and \
                        can be removed.",
                    delegate.clone(),
                ),
                parser: parser.clone(),
            });
        }
    }
}

pub struct UnoptimizedInput;

impl LinterRule for UnoptimizedInput {
    fn severity(&self) -> LinterType {
        LinterType::Info
    }

    fn title(&self) -> &str {
        "Unoptimized input"
    }

    fn run(
        &self,
        _analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Input { .. })
            && matches!(parser.kind(), ParserKind::Input { message: None })
        {
            issues.push(LinterIssue {
                title: self.title().to_string(),
                severity: self.severity(),
                description: "A flatten parser without an error message is unable to switch \
                            to the fast parsing mode. This can lead to inefficient parsers \
                            and can usually be easily fixed by providing an error message \
                            that should be used in case the delegate fails to parse."
                    .to_string(),
                parser: parser.clone(),
            });
        }
    }
}

pub struct UnreachableChoice;

impl LinterRule for UnreachableChoice {
    fn severity(&self) -> LinterType {
        LinterType::Warning
    }

    fn title(&self) -> &str {
        "Unreachable choice"
    }

    fn run(
        &self,
        analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Choice { .. }) {
            let length = parser.children().len();
            for (i, child) in parser.children().iter().enumerate() {
                if i < length - 1 && analyzer.is_nullable(child) {
                    issues.push(LinterIssue {
                        title: self.title().to_string(),
                        severity: self.severity(),
                        description: format!(
                            "The choice at index {} is nullable:\n \
                                {}: {:?}\n\
                                thus the choices after that can never be reached and can be \
                                removed:\n\
                                {}",
                            i,
                            i,
                            child,
                            format_iterable(&parser.children()[i + 1..], Some(i + 1))
                        ),
                        parser: parser.clone(),
                    });
                }
            }
        }
    }
}

pub struct UnresolvedSettable;

impl LinterRule for UnresolvedSettable {
    fn severity(&self) -> LinterType {
        LinterType::Error
    }

    fn title(&self) -> &str {
        "Unresolved settable"
    }

    fn run(
        &self,
        _analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        // An unresolved settable is the owning `SettableParser` (kind `Settable`, *not* the
        // `SettableRef` weak back-references) still delegating to the `undefined()` sentinel — i.e.
        // its sole child has kind `Undefined`. Checking the owner's kind (not `SettableRef`) is
        // what keeps this firing once per unresolved rule rather than once per reference to it.
        if matches!(parser.kind(), ParserKind::Settable)
            && parser
                .children()
                .first()
                .is_some_and(|child| matches!(child.kind(), ParserKind::Undefined { .. }))
        {
            issues.push(LinterIssue {
                title: self.title().to_string(),
                severity: self.severity(),
                parser: parser.clone(),
                description: "This settable parser was created with `undefined()` and never \
                              had `.set()` called on it. This is typically a bug in a \
                              recursive grammar definition."
                    .to_string(),
            });
        }
    }
}

pub struct UnusedResult;

impl LinterRule for UnusedResult {
    fn severity(&self) -> LinterType {
        LinterType::Info
    }

    fn title(&self) -> &str {
        "Unused result"
    }

    fn run(
        &self,
        analyzer: &Analyzer,
        parser: &Rc<dyn HasChildren>,
        issues: &mut Vec<LinterIssue>,
    ) {
        if matches!(parser.kind(), ParserKind::Input { .. }) {
            let deep_children = analyzer.all_children(parser);
            let ignored_results: Vec<&Rc<dyn HasChildren>> = deep_children
                .iter()
                .filter(|p| p.is_result_producing())
                .collect();
            if !ignored_results.is_empty() {
                let path = analyzer
                    .find_path(parser, &|path: &ParserPath| {
                        ignored_results
                            .iter()
                            .any(|ir| ptr(ir) == ptr(&path.target()))
                    })
                    .unwrap();
                issues.push(LinterIssue {
                    title: self.title().to_string(),
                    severity: self.severity(),
                    description: format!(
                        "The flatten parser discards the result of its children and \
                            instead returns the consumed input. Yet this flatten parser \
                            (indirectly) refers to one or more other parsers that explicitly \
                            produce a result which is then ignored when called from this \
                            context:\n\
                            {}\n\
                            This might point to an inefficient grammar or a possible bug.",
                        format_iterable(&path.parsers, Some(1))
                    ),
                    parser: parser.clone(),
                });
            }
        }
    }
}
