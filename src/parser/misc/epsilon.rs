use std::fmt::Debug;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Clone, Debug)]
pub struct EpsilonParser<T: Clone + Debug> {
    pub result: T,
}

impl <T> Parser<T> for EpsilonParser<T> where T: Clone + Debug {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        context.success(self.result.clone(), context.position)
    }

    fn fast_parse_on(&self, _buffer: Rc<[char]>, position: usize) -> Option<usize> {
        Some(position)
    }
}

pub fn epsilon() -> EpsilonParser<()> {
    EpsilonParser { result: () }
}

pub fn epsilon_with<T: Clone + Debug>(result: T) -> EpsilonParser<T> {
    EpsilonParser { result }
}
