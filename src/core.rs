use std::any::Any;
use std::fmt::{write, Debug, Display};
use std::hash::Hash;
use std::rc::Rc;
use imstr::ImString;
use unicode_categories::UnicodeCategories;
use crate::context::{Context, ParseResult, Success, Failure};

pub type Shared<T> = Rc<T>;

#[derive(PartialEq, Eq)]
pub struct Token<T> {
    pub value: Rc<T>,
    pub buffer: Rc<Vec<char>>,
    pub start: usize,
    pub end: usize,
}

impl<T> Token<T> {
    pub fn new(value: T, buffer: Rc<Vec<char>>, start: usize, end: usize) -> Token<T> {
        Token { value: Rc::new(value), buffer, start, end }
    }

    pub fn input(&self) -> &[char] {
        &self.buffer[self.start as usize..self.end as usize]
    }

    pub fn length(&self) -> usize {
        self.end - self.start
    }

    pub fn line(&self) -> usize {
        line_and_column_of(&self.buffer, self.start).0
    }

    pub fn column(&self) -> usize {
        line_and_column_of(&self.buffer, self.start).1
    }
}

impl<T: Display> Display for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Token[{}]: {}", position_string(&self.buffer, self.start), self.value)
    }
}

pub fn line_and_column_of(buffer: &Vec<char>, position: usize) -> (usize, usize) {
    let mut line: usize = 1;
    let mut offset: usize = 0;

    // TODO

    (line, offset)
}

pub fn position_string(buffer: &Vec<char>, position: usize) -> String {
    let (line, column) = line_and_column_of(buffer, position);
    format!("{}:{}", line, column)
}

impl <'src> PartialEq for CharTest<'src> {
    fn eq(&self, other: &CharTest<'src>) -> bool {
        self.description == other.description && Rc::as_ptr(&self.test) == Rc::as_ptr(&other.test)
    }
}

impl <'src> Eq for CharTest<'src> {}

impl <'src> Debug for CharTest<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CharTest {{ description: {:?}, test: @{:?}", self.description, Rc::as_ptr(&self.test))
    }
}

impl <'src> Hash for CharTest<'src> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.description.hash(state);
        Rc::as_ptr(&self.test).hash(state);
    }
}

enum Parser<'src> {
    Char(CharParser<'src>)
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
enum CharParser<'src> {
    Any,
    Exact(char),
    Letter,
    Digit(Option<u32>),
    OneOf(Vec<char>),
    Lowercase,
    Uppercase,
    NoneOf(Vec<char>),
    Predicate(CharTest<'src>),
    Whitespace,
    Word,
}

static ANY: Arc<fn(char) -> bool> = Arc::new(|_: char| true);
fn exact(exp: char, c: char) -> bool { c == exp }
fn letter(c: char) -> bool { c.is_ascii_alphabetic() }
fn digit(c: char) -> bool { c.is_ascii_digit() }
fn one_of(chars: &Vec<char>, c: char) -> bool { chars.contains(&c) }
fn lowercase(c: char) -> bool { c.is_ascii_lowercase() }
fn uppercase(c: char) -> bool { c.is_ascii_uppercase() }
fn none_of(chars: &Vec<char>, c: char) -> bool { !chars.contains(&c) }
fn predicate(pred: fn(char) -> bool, c: char) -> bool { pred(c) }
fn whitespace(c: char) -> bool { c.is_whitespace() }
fn word(c: char) -> bool { c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' }

impl <'src> CharParser<'src> {
    fn test(&self) -> CharTest<'src> {
        match self {
            CharParser::Any => CharTest {
                description: Some("ANY character"),
                test: Rc::new(ANY),
            },
            CharParser::Exact(_) => {}
            CharParser::Letter => {}
            CharParser::Digit(_) => {}
            CharParser::OneOf(_) => {}
            CharParser::Lowercase => {}
            CharParser::NoneOf(_) => {}
            CharParser::Predicate(_) => {}
            CharParser::Whitespace => {}
            CharParser::Word => {}
        }
    }
}

#[derive(Clone)]
struct CharTest<'src> {
    description: Option<&'src str>,
    test: Rc<dyn FnOnce(char) -> bool>,
}


pub trait Parse<'src, T>: Any + PartialEq + Debug + Clone + Eq + Hash {
    fn parse_on(&self, context: &Context) -> ParseResult<T>;
    fn fast_parse_on(&self, buffer: &'src Vec<char>, position: usize) -> Option<usize> {
        let result: ParseResult<T> = self.parse_on(&Context { buffer, position });
        match result {
            ParseResult::Failure(_) => None,
            ParseResult::Success(Success { context: c, value: _ }) => Some(c.position),
        }
    }
    fn parse(&self, input: &str, position: usize) -> ParseResult<T> {
        self.parse_on(&Context { buffer: &input.chars().collect(), position })
    }
}

impl <'src> Parse<'src, char> for CharParser<'src> {
    fn parse_on(&self, context: &Context) -> ParseResult<char> {
        let buffer = context.buffer;
        let position = context.position;
        match self {
            CharParser::Any => Success::new(buffer, buffer[position], position + 1),
            CharParser::Exact(c) =>
                check_next_char(
                    context,
                    Box::new(|bp: char| bp == *c),
                    format!("expected '{}', but found '{}'", c, buffer[position]),
                ),
            CharParser::Letter =>
                check_next_char(
                    context,
                    Box::new(|bp: char| bp.is_letter()),
                    format!("expected letter, but found '{}'", buffer[position])
                ),
            CharParser::Digit(radix) =>
                check_next_char(
                    context,
                    Box::new(|bp: char| bp.is_digit(radix.unwrap_or(10))),
                    format!("expected digit for base {}, but found '{}'", radix.unwrap_or(10), buffer[position])
                ),
            CharParser::OneOf(chars) =>
                check_next_char(
                    context,
                    Box::new(|bp: char| chars.contains(&bp)),
                    format!("expected ANY of {:?}, but found '{}'", chars, buffer[position])
                ),
            CharParser::Lowercase =>
                check_next_char(
                    context,
                    Box::new(|bp: char| bp.is_ascii_lowercase()),
                    format!("expected lowercase letter, but found '{}'", buffer[position])
                ),
            CharParser::NoneOf(chars) =>
                check_next_char(
                    context,
                    Box::new(|bp: char| !chars.contains(&bp)),
                    format!("expected a char not in {:?}, but found '{}'", chars, buffer[position])
                ),
            CharParser::Predicate(test) =>
                check_next_char(
                    context,
                    Box::new(test.test),
                    format!("expected predicate, but found '{}'", buffer[position])
                ),
            CharParser::Whitespace =>
                check_next_char(
                    context,
                    Box::new(|bp: char| bp.is_whitespace()),
                    format!("expected whitespace character, but found '{}'", buffer[position])
                ),
            CharParser::Word =>
                check_next_char(
                    context,
                    Box::new(|bp: char| bp.is_ascii_alphabetic() || bp.is_ascii_digit() || bp == '_'),
                    format!("expected word character (alpha, digit or _), but found '{}'", buffer[position])
                )
        }
    }
}

fn check_next_char(context: &Context, test: Box<CharTest>, failure_message: String) -> ParseResult<char> {
    let buffer = context.buffer.clone();
    let position = context.position;
    if test(buffer[position]) {
        Success::new(buffer.clone(), buffer[position], position + 1)
    } else {
        Failure::new(buffer, position, ImString::from(failure_message))
    }
}