use std::rc::Rc;

use crate::{core::parser::HasChildren, reflection::analyzer::ptr};

#[derive(Clone, Debug)]
pub struct ParserPath {
    pub parsers: Vec<Rc<dyn HasChildren>>,
    pub indexes: Vec<usize>,
}

impl ParserPath {
    pub fn new(parsers: Vec<Rc<dyn HasChildren>>, indexes: Vec<usize>) -> Self {
        assert!(!parsers.is_empty(), "parsers cannot be empty");
        assert!(
            indexes.len() == parsers.len() - 1,
            "indexes must be one element shorter than parsers"
        );
        for (i, index) in indexes.iter().enumerate() {
            if ptr(&parsers[i].children()[*index]) != ptr(&parsers[i + 1]) {
                panic!("indexes invalid");
            }
        }
        ParserPath { parsers, indexes }
    }

    pub fn source(&self) -> Rc<dyn HasChildren> {
        self.parsers[0].clone()
    }

    pub fn target(&self) -> Rc<dyn HasChildren> {
        self.parsers.last().unwrap().clone()
    }

    pub fn len(&self) -> usize {
        self.parsers.len()
    }

    pub fn contains(&self, parser: &Rc<dyn HasChildren>) -> bool {
        self.parsers.iter().any(|p| ptr(p) == ptr(parser))
    }

    pub fn push(&mut self, parser: Rc<dyn HasChildren>, index: usize) {
        self.parsers.push(parser.clone());
        self.indexes.push(index);
    }

    pub fn pop(&mut self) {
        self.parsers.pop();
        self.indexes.pop();
    }
}

pub fn depth_first_search(
    path: &mut ParserPath,
    predicate: &impl Fn(&ParserPath) -> bool,
) -> Vec<ParserPath> {
    if predicate(path) {
        return vec![path.clone()];
    }
    let mut results = Vec::new();
    let children = path.target().children(); // hoist: owns the Vec, so push/pop below is fine
    for (i, child) in children.iter().enumerate() {
        if !path.contains(child) {
            path.push(child.clone(), i);
            results.extend(depth_first_search(path, predicate));
            path.pop();
        }
    }
    results
}
