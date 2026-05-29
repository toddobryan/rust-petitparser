use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::core::result::{Failure, ParseResult, Success};
use crate::core::token::position_string;

#[derive(Clone, Debug)]
pub struct Context {
    pub buffer: Rc<[char]>,
    pub position: usize,
}

impl Context {
    pub fn new(buffer: Rc<[char]>, position: usize) -> Self {
        Self { buffer, position }
    }

    pub fn success<T>(&self, value: T, position: usize) -> ParseResult<T> {
        Ok(Success {
            context: Context { buffer: self.buffer.clone(), position },
            value,
        })
    }

    pub fn failure<T>(&self, message: impl Into<String>, position: usize) -> ParseResult<T> {
        Err(Failure {
            context: Context { buffer: self.buffer.clone(), position },
            message: message.into(),
        })
    }

    pub fn to_position_string(&self) -> String {
        position_string(self.buffer.clone(), self.position)
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context[{}]", self.to_position_string())
    }
}
