use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CharKind {
    Any,
    Exact(char),
    ExactCi(char),
    Letter,
    Digit(u32),
    OneOf(Vec<char>),
    OneOfCi(Vec<char>),
    NoneOf(Vec<char>),
    NoneOfCi(Vec<char>),
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
            CharKind::ExactCi(expected) => c.eq_ignore_ascii_case(expected),
            CharKind::Letter => c.is_alphabetic(),
            CharKind::Digit(radix) => c.is_digit(*radix),
            CharKind::OneOf(chars) => chars.contains(&c),
            CharKind::OneOfCi(chars) => chars
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .contains(&c.to_ascii_lowercase()),
            CharKind::NoneOf(chars) => !chars.contains(&c),
            CharKind::NoneOfCi(chars) => !chars
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .contains(&c.to_ascii_lowercase()),
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
            CharKind::ExactCi(c) => format!("'{}' (case-insensitive)", c),
            CharKind::Letter => "letter".to_string(),
            CharKind::Digit(radix) => {
                if *radix == 10 {
                    "digit".to_string()
                } else {
                    format!("digit (radix {})", radix)
                }
            }
            CharKind::OneOf(chars) => format!("any of {:?}", chars),
            CharKind::OneOfCi(chars) => format!("any of {:?} (case-insensitive)", chars),
            CharKind::NoneOf(chars) => format!("none of {:?}", chars),
            CharKind::NoneOfCi(chars) => format!("none of {:?} (case-insensitive)", chars),
            CharKind::Lowercase => "lowercase letter".to_string(),
            CharKind::Uppercase => "uppercase letter".to_string(),
            CharKind::Whitespace => "whitespace".to_string(),
            CharKind::Word => "word character (letter, digit, or '_')".to_string(),
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

    pub fn with_message(self, message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            ..self
        }
    }
}

impl Parser<char> for CharParser {
    fn parse_on(&self, context: &Context) -> ParseResult<char> {
        let pos = context.position();
        if pos >= context.buffer().len() {
            return context.failure(self.message_for(None));
        }
        let c = context.buffer()[pos];
        if self.kind.matches(c) {
            context.success_with_position(c, pos + 1)
        } else {
            context.failure_with_position(self.message_for(Some(c)), pos)
        }
    }
}

#[derive(Clone)]
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

impl Debug for PredicateCharParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PredicateCharParser")
            .field("test", &"<predicate function>")
            .field("description", &self.description)
            .field("message", &self.message)
            .finish()
    }
}

impl Parser<char> for PredicateCharParser {
    fn parse_on(&self, context: &Context) -> ParseResult<char> {
        if context.position() >= context.buffer().len() {
            return context.failure(self.message_for(None));
        }
        let c = context.buffer()[context.position()];
        if (self.test)(c) {
            context.success_with_position(c, context.position() + 1)
        } else {
            context.failure(self.message_for(Some(c)))
        }
    }
}

pub fn any() -> CharParser {
    CharParser {
        kind: CharKind::Any,
        message: None,
    }
}

pub fn char(c: char) -> CharParser {
    CharParser {
        kind: CharKind::Exact(c),
        message: None,
    }
}

pub fn char_ci(c: char) -> CharParser {
    CharParser {
        kind: CharKind::ExactCi(c),
        message: None,
    }
}

pub fn letter() -> CharParser {
    CharParser {
        kind: CharKind::Letter,
        message: None,
    }
}

pub fn digit() -> CharParser {
    digit_with_radix(10)
}

pub fn digit_with_radix(radix: u32) -> CharParser {
    CharParser {
        kind: CharKind::Digit(radix),
        message: None,
    }
}

pub fn one_of(chars: &str) -> CharParser {
    CharParser {
        kind: CharKind::OneOf(chars.chars().collect()),
        message: None,
    }
}

pub fn one_of_ci(chars: &str) -> CharParser {
    CharParser {
        kind: CharKind::OneOfCi(chars.chars().collect()),
        message: None,
    }
}

pub fn none_of(chars: &str) -> CharParser {
    CharParser {
        kind: CharKind::NoneOf(chars.chars().collect()),
        message: None,
    }
}

pub fn none_of_ci(chars: &str) -> CharParser {
    CharParser {
        kind: CharKind::NoneOfCi(chars.chars().collect()),
        message: None,
    }
}

pub fn lowercase() -> CharParser {
    CharParser {
        kind: CharKind::Lowercase,
        message: None,
    }
}

pub fn uppercase() -> CharParser {
    CharParser {
        kind: CharKind::Uppercase,
        message: None,
    }
}

pub fn whitespace() -> CharParser {
    CharParser {
        kind: CharKind::Whitespace,
        message: None,
    }
}

pub fn word() -> CharParser {
    CharParser {
        kind: CharKind::Word,
        message: None,
    }
}

pub fn char_if<F>(test: F, description: impl Into<String>) -> PredicateCharParser
where
    F: Fn(char) -> bool + 'static,
{
    PredicateCharParser {
        test: Rc::new(test),
        description: description.into(),
        message: None,
    }
}
