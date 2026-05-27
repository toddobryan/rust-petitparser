use std::fmt::{Display, Formatter};
use crate::context::Failure;
use crate::core::Shared;

pub struct ParserError<'a> {
    pub failure: &'a Failure<'a>,
}

impl <'a> ParserError<'a> {
    pub fn message(&self) -> &str {
        &self.failure.message
    }

    pub fn offset(&self) -> usize {
        self.failure.context.position
    }

    pub fn source(&self) -> Shared<Vec<char>> {
        self.failure.context.buffer.clone()
    }
}

impl <'a> Display for ParserError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.failure.message, self.failure.context.to_position_string())
    }
}

pub struct UnsupportedError {
    pub message: String,
}