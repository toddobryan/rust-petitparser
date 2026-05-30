use crate::core::result::Failure;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub struct ParserError {
    pub failure: Failure,
}

impl ParserError {
    pub fn message(&self) -> &str {
        &self.failure.message
    }

    pub fn offset(&self) -> usize {
        self.failure.context.position
    }

    pub fn source(&self) -> Rc<[char]> {
        self.failure.context.buffer.clone()
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}",
            self.failure.message,
            self.failure.context.to_position_string()
        )
    }
}

pub struct UnsupportedError {
    pub message: String,
}
