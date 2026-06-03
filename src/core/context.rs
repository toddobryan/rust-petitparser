use crate::core::result::{Failure, ParseResult, Success};
use crate::core::token::position_string;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct Context {
    pub buffer: Rc<[char]>,
    pub position: usize,
}

pub trait HasContext {
    fn buffer(&self) -> Rc<[char]>;
    fn position(&self) -> usize;

    fn success<T>(&self, value: T) -> ParseResult<T> {
        self.success_with_position(value, self.position())
    }

    fn success_with_position<T>(&self, value: T, position: usize) -> ParseResult<T> {
        Ok(Success {
            context: Context {
                buffer: self.buffer().clone(),
                position,
            },
            value,
        })
    }

    fn failure<T>(&self, message: impl Into<String>) -> ParseResult<T> {
        self.failure_with_position(message, self.position())
    }

    fn failure_with_position<T>(
        &self,
        message: impl Into<String>,
        position: usize,
    ) -> ParseResult<T> {
        Err(Failure {
            context: Context {
                buffer: self.buffer().clone(),
                position,
            },
            message: message.into(),
        })
    }

    fn to_position_string(&self) -> String {
        position_string(self.buffer().clone(), self.position())
    }
}

impl HasContext for Context {
    fn buffer(&self) -> Rc<[char]> {
        self.buffer.clone()
    }

    fn position(&self) -> usize {
        self.position
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Context[{}={}]",
            self.position,
            self.to_position_string()
        )
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
