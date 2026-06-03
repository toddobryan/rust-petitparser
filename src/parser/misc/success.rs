use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SuccessParser<T: Clone + Debug> {
    result: T,
}

impl<T> Parser<T> for SuccessParser<T>
where
    T: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        context.success(self.result.clone())
    }

    fn fast_parse_on(&self, _buffer: Rc<[char]>, position: usize) -> Option<usize> {
        Some(position)
    }
}

pub fn success<T: Clone + Debug>(result: T) -> SuccessParser<T> {
    SuccessParser { result }
}
