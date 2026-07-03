use crate::core::context::{Context, HasContext};
use crate::core::kind::{ParserKind, PtrKey};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::{Failure, ParseResult};
use std::fmt::Debug;
use std::rc::Rc;

pub fn string(literal: &str) -> PredicateParser {
    let literal = literal.to_string();
    PredicateParser {
        length: literal.chars().count(),
        message: format!("Expected string: \"{}\"", literal),
        predicate: Rc::new(move |s| s == literal),
    }
}

pub fn string_ignore_case(literal: &str) -> PredicateParser {
    let literal = literal.to_string().to_lowercase();
    PredicateParser {
        length: literal.chars().count(),
        message: format!("Expected string (case-insensitive): \"{}\"", literal),
        predicate: Rc::new(move |s| s.to_lowercase() == literal),
    }
}

pub struct PredicateParser {
    pub length: usize,
    pub predicate: Rc<dyn Fn(&str) -> bool>,
    pub message: String,
}

impl Clone for PredicateParser {
    fn clone(&self) -> Self {
        Self {
            length: self.length,
            predicate: self.predicate.clone(),
            message: self.message.clone(),
        }
    }
}

impl Debug for PredicateParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PredicateParser")
            .field("length", &self.length)
            .field("predicate", &"<string predicate>")
            .field("message", &self.message)
            .finish()
    }
}

impl PredicateParser {
    pub fn with_message(&self, message: String) -> Self {
        Self {
            length: self.length,
            predicate: self.predicate.clone(),
            message,
        }
    }
}

impl HasChildren for PredicateParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Predicate {
            predicate: Rc::as_ptr(&self.predicate) as PtrKey,
            length: self.length,
            message: &self.message,
        }
    }

    fn is_string_predicate(&self) -> bool {
        true
    }
}

impl Parser<String> for PredicateParser {
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        let start = context.position;
        let end = start + self.length;
        if end <= context.buffer.len() {
            let substring: String = context.buffer[start..end].iter().collect();
            if (self.predicate)(&substring) {
                context.success_with_position(substring, context.position + self.length)
            } else {
                Err(Failure {
                    context: context.clone(),
                    message: self.message.clone(),
                })
            }
        } else {
            Err(Failure {
                context: context.clone(),
                message: "Input buffer is too short for predicate".to_string(),
            })
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let end = context.position + self.length;
        if end <= context.buffer.len() {
            let substring: String = context.buffer[context.position..end].iter().collect();
            if (self.predicate)(&substring) {
                return Some(end);
            }
        }
        None
    }
}
