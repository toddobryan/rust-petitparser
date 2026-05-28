use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::context::{Context, ParseResult};

pub type Shared<T> = Rc<T>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<T> {
    pub value: T,
    pub buffer: Rc<[char]>,
    pub start: usize,
    pub end: usize,
}

impl<T> Token<T> {
    pub fn new(value: T, buffer: Rc<[char]>, start: usize, end: usize) -> Self {
        Self { value, buffer, start, end }
    }

    pub fn input(&self) -> &[char] {
        &self.buffer[self.start..self.end]
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
        write!(
            f,
            "Token[{}]: {}",
            position_string(&self.buffer, self.start),
            self.value
        )
    }
}

pub fn line_and_column_of(_buffer: &[char], _position: usize) -> (usize, usize) {
    // TODO: scan buffer up to position, counting newlines
    (1, 0)
}

pub fn position_string(buffer: &[char], position: usize) -> String {
    let (line, column) = line_and_column_of(buffer, position);
    format!("{}:{}", line, column)
}

pub trait Parser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T>;

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.parse_on(&Context::new(buffer, position))
            .ok()
            .map(|s| s.context.position)
    }

    fn parse(&self, input: &str) -> ParseResult<T> {
        let buffer: Rc<[char]> = input.chars().collect::<Vec<_>>().into();
        self.parse_on(&Context::new(buffer, 0))
    }
}
