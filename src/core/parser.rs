use std::fmt::Debug;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::result::ParseResult;

pub trait Parser<T>: Debug {
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
