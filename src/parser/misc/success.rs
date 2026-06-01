use std::fmt::Debug;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Debug, Clone)]
pub struct SuccessParser<T: Clone + Debug> {
    result: T,
}

impl <T> Parser<T> for SuccessParser<T> where T: Clone + Debug {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        context.success(self.result.clone(), context.position)
    }

    fn fast_parse_on(&self, _buffer: Rc<[char]>, position: usize) -> Option<usize> {
        Some(position)
    }
}

pub fn success<T: Clone + Debug>(result: T) -> SuccessParser<T> {
    SuccessParser { result }
}
