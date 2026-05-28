use crate::core::{Shared, position_string};
use imstr::ImString;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct Context<'src> {
    pub buffer: &'src Vec<char>,
    pub position: usize,
}

impl <'src> Context<'src> {
    pub fn success<T>(&self, result: T, position: usize) -> ParseResult<T> {
        Success::new(self.buffer, result, position)
    }

    pub fn failure<T>(&self, message: &'src str, position: usize) -> ParseResult<T> {
        Failure::new(self.buffer, position, message)
    }

    pub fn to_position_string(&self) -> String {
        position_string(self.buffer, self.position)
    }
}

impl<'src> Display for Context<'src> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context[{}]", self.to_position_string())
    }
}

#[derive(Debug)]
pub enum ParseResult<'src, T> {
    Failure(Failure<'src>),
    Success(Success<'src, T>),
}

#[derive(Debug)]
pub struct Failure<'src> {
    pub context: Box<Context<'src>>,
    pub message: &'src str,
}

impl <'src> Failure<'src> {
    pub fn new<T>(buffer: &'src Vec<char>, position: usize, message: &'src str) -> ParseResult<'src, T> {
        let result = ParseResult::Failure(Failure {
            context: Box::new(Context { buffer, position }),
            message,
        });
        result
    }
}

impl <'src> Display for Failure<'src> {
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
pub struct Success<'src, T> {
    pub context: Box<Context<'src>>,
    pub value: T,
}

impl<'src, T> Success<'src, T> {
    pub(crate) fn new(buffer: &'src Vec<char>, result: T, position: usize) -> ParseResult<'src, T> {
        let pr = ParseResult::Success(Success {
            context: Box::new(Context { buffer, position }),
            value: result,
        });
        pr
    }
}

impl<'src, T> ParseResult<'src, T> {
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
                .failure(failure.message, failure.context.position),
            ParseResult::Success(success) => {
                let mapped = callback(&success.value);
                success
                    .context
                    .success(mapped, success.context.position)
            },
        }
    }
}
