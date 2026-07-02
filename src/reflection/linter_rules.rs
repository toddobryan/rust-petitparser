use std::rc::Rc;

use crate::{
    core::parser::HasChildren,
    reflection::{
        analyzer::Analyzer,
        formatting::format_iterable,
        linter::{LinterIssue, LinterRule, LinterType},
    },
};

pub const ALL_LINTER_RULES: &[&dyn LinterRule] =
    &[&LeftRecursion, &NullableRepeater, &UnresolvedSettable];

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
