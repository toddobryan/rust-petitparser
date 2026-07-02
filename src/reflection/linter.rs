use std::rc::Rc;

use crate::{core::parser::HasChildren, reflection::analyzer::Analyzer};

#[derive(Debug, Clone, PartialEq)]
pub enum LinterType {
    Info,
    Warning,
    Error,
}

pub struct LinterIssue {
    pub title: String,
    pub severity: LinterType,
    pub description: String,
    // the offending parser — Rc<dyn HasChildren> so it's type-erased
    pub parser: Rc<dyn HasChildren>,
}

pub trait LinterRule {
    fn severity(&self) -> LinterType;
    fn title(&self) -> &str;
    fn run(&self, analyzer: &Analyzer, parser: &Rc<dyn HasChildren>, issues: &mut Vec<LinterIssue>);
}

pub fn linter(root: Rc<dyn HasChildren>, rules: &[&dyn LinterRule]) -> Vec<LinterIssue> {
    let analyzer = Analyzer::new(root);
    let mut issues = Vec::new();
    for parser in &analyzer.parsers {
        for rule in rules {
            rule.run(&analyzer, parser, &mut issues);
        }
    }
    issues
}
