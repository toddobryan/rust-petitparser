use std::rc::Rc;

use crate::{
    core::parser::HasChildren,
    reflection::{
        analyzer::Analyzer,
        formatting::format_iterable,
        linter::{LinterIssue, LinterRule, LinterType},
    },
};

pub const ALL_LINTER_RULES: &[&dyn LinterRule] = &[
    &CharacterRepeater,
    &LeftRecursion,
    &NestedChoice,
    &NullableRepeater,
    &UnnecessaryInput,
    &UnoptimizedInput,
    &UnreachableChoice,
    &UnresolvedSettable,
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
        if parser.is_input() {
            let repeating = parser.children()[0].clone();
            if repeating.is_possessive_repeating() {
                let character = repeating.children()[0].clone();
                if character.is_char() {
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
        if parser.is_choice() {
            let length = parser.children().len();
            for (i, child) in parser.children().iter().enumerate() {
                if i < length - 1 && child.is_choice() {
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
        if parser.is_repeating()
            && parser
                .children()
                .first()
                .is_some_and(|p| analyzer.is_nullable(p))
        {
            if parser.is_separated_repeating()
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
        if parser.is_input()
            && parser.input_message().is_none()
            && let Some(delegate) = parser.children().first()
            && (delegate.is_char()
                || delegate.is_input()
                || delegate.is_newline()
                || delegate.is_string_predicate()
                || delegate.is_char_repeating())
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
        if parser.is_input() && parser.input_message().is_none() {
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
        if parser.is_choice() {
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
        if parser.is_settable() && parser.is_undefined() {
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
