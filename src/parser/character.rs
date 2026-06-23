use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::range::RangeInclusive;
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
    Pattern(Vec<RangeInclusive<char>>),
    PatternCi(Vec<RangeInclusive<char>>),
    NegatedPattern(Vec<RangeInclusive<char>>),
    NegatedPatternCi(Vec<RangeInclusive<char>>),
    Lowercase,
    Uppercase,
    Whitespace,
    Word,
}

impl CharKind {
    fn matches(&self, c: char) -> bool {
        use CharKind::*;
        match self {
            Any => true,
            Exact(expected) => c == *expected,
            ExactCi(expected) => c.eq_ignore_ascii_case(expected),
            Letter => c.is_alphabetic(),
            Digit(radix) => c.is_digit(*radix),
            OneOf(chars) => chars.contains(&c),
            OneOfCi(chars) => chars
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .contains(&c.to_ascii_lowercase()),
            NoneOf(chars) => !chars.contains(&c),
            NoneOfCi(chars) => !chars
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .contains(&c.to_ascii_lowercase()),
            Pattern(ranges) => ranges.iter().any(|r| r.contains(&c)),
            PatternCi(ranges) => ranges.iter().any(|r| {
                r.contains(&c.to_ascii_lowercase()) || r.contains(&c.to_ascii_uppercase())
            }),
            NegatedPattern(ranges) => !ranges.iter().any(|r| r.contains(&c)),
            NegatedPatternCi(ranges) => !ranges.iter().any(|r| {
                r.contains(&c.to_ascii_lowercase()) || r.contains(&c.to_ascii_uppercase())
            }),
            Lowercase => c.is_lowercase(),
            Uppercase => c.is_uppercase(),
            Whitespace => c.is_whitespace(),
            Word => c.is_alphanumeric() || c == '_',
        }
    }

    fn default_description(&self) -> String {
        use CharKind::*;
        match self {
            Any => "any character".to_string(),
            Exact(c) => format!("'{}'", c),
            ExactCi(c) => format!("'{}' (case-insensitive)", c),
            Letter => "letter".to_string(),
            Digit(radix) => {
                if *radix == 10 {
                    "digit".to_string()
                } else {
                    format!("digit (radix {})", radix)
                }
            }
            OneOf(chars) => format!("any of {:?}", chars),
            OneOfCi(chars) => format!("any of {:?} (case-insensitive)", chars),
            NoneOf(chars) => format!("none of {:?}", chars),
            NoneOfCi(chars) => format!("none of {:?} (case-insensitive)", chars),
            Lowercase => "lowercase letter".to_string(),
            Uppercase => "uppercase letter".to_string(),
            Pattern(ranges) => ranges_to_string(ranges, false, false),
            PatternCi(ranges) => ranges_to_string(ranges, false, true),
            NegatedPattern(ranges) => ranges_to_string(ranges, true, false),
            NegatedPatternCi(ranges) => ranges_to_string(ranges, true, true),
            Whitespace => "whitespace".to_string(),
            Word => "word character (letter, digit, or '_')".to_string(),
        }
    }
}

fn ranges_to_string(
    ranges: &[RangeInclusive<char>],
    is_negated: bool,
    is_case_insensitive: bool,
) -> String {
    fn to_string(r: &RangeInclusive<char>) -> String {
        if r.start == r.last {
            format!("{}", r.start)
        } else {
            format!("{}-{}", r.start, r.last)
        }
    }
    let caret = if is_negated { "^" } else { "" };
    let case_insenstive = if is_case_insensitive {
        " (case-insensitive)"
    } else {
        ""
    };
    if ranges.len() == 1 && ranges[0].start == ranges[0].last {
        format!("{}{:?}{}", caret, ranges[0].start, case_insenstive)
    } else {
        let concat_ranges = ranges
            .iter()
            .map(to_string)
            .collect::<Vec<String>>()
            .join("");
        format!("[{}{}]{}", caret, concat_ranges, case_insenstive)
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

pub fn pattern(p: &str) -> CharParser {
    let (is_negated, ranges) = parse_pattern(p);
    CharParser {
        kind: {
            if is_negated {
                CharKind::NegatedPattern(ranges)
            } else {
                CharKind::Pattern(ranges)
            }
        },
        message: None,
    }
}

pub fn pattern_ci(p: &str) -> CharParser {
    let (is_negated, ranges) = parse_pattern(p);
    CharParser {
        kind: {
            if is_negated {
                CharKind::NegatedPatternCi(ranges)
            } else {
                CharKind::PatternCi(ranges)
            }
        },
        message: None,
    }
}

fn parse_pattern(pattern: &str) -> (bool, Vec<RangeInclusive<char>>) {
    let p: Vec<char> = pattern.chars().collect();
    let p: &[char] = &p;
    let mut ranges: Vec<RangeInclusive<char>> = Vec::new();
    let is_negated: bool = !p.is_empty() && p[0] == '^';
    let mut current: usize = if is_negated { 1 } else { 0 };
    while current < p.len() {
        if current + 2 < p.len() && p[current + 1] == '-' {
            ranges.push((p[current]..=p[current + 2]).into());
            current += 3;
        } else {
            ranges.push((p[current]..=p[current]).into());
            current += 1;
        }
    }
    (is_negated, ranges)
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
