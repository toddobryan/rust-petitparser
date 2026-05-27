use crate::core::{Shared, position_string};
use imstr::ImString;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct Context {
    pub buffer: Rc<Vec<char>>,
    pub position: usize,
}

impl Context {
    pub fn success<T>(&self, result: T, position: usize) -> ParseResult<T> {
        Success::new(self.buffer.clone(), result, position)
    }

    pub fn failure<T>(&self, message: ImString, position: usize) -> ParseResult<T> {
        Failure::new(self.buffer.clone(), position, message)
    }

    pub fn to_position_string(&self) -> String {
        position_string(&*self.buffer, self.position)
    }
}

impl<'a> Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context[{}]", self.to_position_string())
    }
}

#[derive(Debug)]
pub enum ParseResult<T> {
    Failure(Failure),
    Success(Success<T>),
}

#[derive(Debug)]
pub struct Failure {
    pub context: Rc<Context>,
    pub message: ImString,
}

impl Failure {
    pub fn new<T>(buffer: Rc<Vec<char>>, position: usize, message: ImString) -> ParseResult<T> {
        let result = ParseResult::Failure(Failure {
            context: Rc::new(Context { buffer, position }),
            message,
        });
        result
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failure[{}]: {}",
            self.context.to_position_string(),
            self.message
        )
    }
}

#[derive(Debug)]
pub struct Success<T> {
    pub context: Rc<Context>,
    pub value: Rc<T>,
}

impl<T> Success<T> {
    pub(crate) fn new(buffer: Rc<Vec<char>>, result: T, position: usize) -> ParseResult<T> {
        let pr = ParseResult::Success(Success {
            context: Rc::new(Context { buffer, position }),
            value: Rc::new(result),
        });
        pr
    }
}

impl<T> ParseResult<T> {
    fn is_success(&self) -> bool {
        match self {
            ParseResult::Success { .. } => true,
            _ => false,
        }
    }

    fn is_failure(&self) -> bool {
        !self.is_success()
    }

    fn map<U>(&self, callback: impl FnOnce(&T) -> U) -> ParseResult<U> {
        match self {
            ParseResult::Failure(failure) => failure
                .context
                .failure(failure.message.clone(), failure.context.position),
            ParseResult::Success(success) => success
                .context
                .success(callback(&*success.value), success.context.position),
        }
    }
}
