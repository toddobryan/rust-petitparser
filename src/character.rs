use std::rc::Rc;

use crate::context::{Context, ParseResult};
use crate::core::Parser;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CharKind {
    Any,
    Exact(char),
    Letter,
    Digit(u32),
    OneOf(Vec<char>),
    NoneOf(Vec<char>),
    Lowercase,
    Uppercase,
    Whitespace,
    Word,
}

impl CharKind {
    fn matches(&self, c: char) -> bool {
        match self {
            CharKind::Any => true,
            CharKind::Exact(expected) => c == *expected,
            CharKind::Letter => c.is_alphabetic(),
            CharKind::Digit(radix) => c.is_digit(*radix),
            CharKind::OneOf(chars) => chars.contains(&c),
            CharKind::NoneOf(chars) => !chars.contains(&c),
            CharKind::Lowercase => c.is_lowercase(),
            CharKind::Uppercase => c.is_uppercase(),
            CharKind::Whitespace => c.is_whitespace(),
            CharKind::Word => c.is_alphanumeric() || c == '_',
        }
    }

    fn default_description(&self) -> String {
        match self {
            CharKind::Any => "any character".to_string(),
            CharKind::Exact(c) => format!("'{}'", c),
            CharKind::Letter => "letter".to_string(),
            CharKind::Digit(radix) => {
                if *radix == 10 {
                    "digit".to_string()
                } else {
                    format!("digit (radix {})", radix)
                }
            }
            CharKind::OneOf(chars) => format!("any of {:?}", chars),
            CharKind::NoneOf(chars) => format!("none of {:?}", chars),
            CharKind::Lowercase => "lowercase letter".to_string(),
            CharKind::Uppercase => "uppercase letter".to_string(),
            CharKind::Whitespace => "whitespace".to_string(),
            CharKind::Word => "word character".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharParser {
    pub kind: CharKind,
    pub message: Option<String>,
}

impl CharParser {
    fn message_for(&self, found: Option<char>) -> String {
        if let Some(m) = &self.message {
            return m.clone();
        }
        let expected = self.kind.default_description();
        match found {
            Some(c) => format!("expected {}, but found '{}'", expected, c),
            None => format!("expected {}, but reached end of input", expected),
        }
    }
}

impl Parser<char> for CharParser {
    fn parse_on(&self, context: &Context) -> ParseResult<char> {
        let pos = context.position;
        if pos >= context.buffer.len() {
            return context.failure(self.message_for(None), pos);
        }
        let c = context.buffer[pos];
        if self.kind.matches(c) {
            context.success(c, pos + 1)
        } else {
            context.failure(self.message_for(Some(c)), pos)
        }
    }
}

pub struct PredicateCharParser {
    pub test: Rc<dyn Fn(char) -> bool>,
    pub description: String,
    pub message: Option<String>,
}

impl PredicateCharParser {
    fn message_for(&self, found: Option<char>) -> String {
        if let Some(m) = &self.message {
            return m.clone();
        }
        match found {
            Some(c) => format!("expected {}, but found '{}'", self.description, c),
            None => format!("expected {}, but reached end of input", self.description),
        }
    }
}

impl Parser<char> for PredicateCharParser {
    fn parse_on(&self, context: &Context) -> ParseResult<char> {
        let pos = context.position;
        if pos >= context.buffer.len() {
            return context.failure(self.message_for(None), pos);
        }
        let c = context.buffer[pos];
        if (self.test)(c) {
            context.success(c, pos + 1)
        } else {
            context.failure(self.message_for(Some(c)), pos)
        }
    }
}

pub fn any(message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Any, message: message.map(String::from) }
}

pub fn char(expected: char, message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Exact(expected), message: message.map(String::from) }
}

pub fn letter(message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Letter, message: message.map(String::from) }
}

pub fn digit(radix: Option<u32>, message: Option<&str>) -> CharParser {
    CharParser {
        kind: CharKind::Digit(radix.unwrap_or(10)),
        message: message.map(String::from),
    }
}

pub fn one_of(chars: &str, message: Option<&str>) -> CharParser {
    CharParser {
        kind: CharKind::OneOf(chars.chars().collect()),
        message: message.map(String::from),
    }
}

pub fn none_of(chars: &str, message: Option<&str>) -> CharParser {
    CharParser {
        kind: CharKind::NoneOf(chars.chars().collect()),
        message: message.map(String::from),
    }
}

pub fn lowercase(message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Lowercase, message: message.map(String::from) }
}

pub fn uppercase(message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Uppercase, message: message.map(String::from) }
}

pub fn whitespace(message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Whitespace, message: message.map(String::from) }
}

pub fn word(message: Option<&str>) -> CharParser {
    CharParser { kind: CharKind::Word, message: message.map(String::from) }
}

pub fn predicate<F>(
    test: F,
    description: impl Into<String>,
    message: Option<&str>,
) -> PredicateCharParser
where
    F: Fn(char) -> bool + 'static,
{
    PredicateCharParser {
        test: Rc::new(test),
        description: description.into(),
        message: message.map(String::from),
    }
}
