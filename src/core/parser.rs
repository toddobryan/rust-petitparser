use crate::core::context::Context;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

pub trait Parser<T>: Debug {
    fn parse_on(&self, context: &Context) -> ParseResult<T>;

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.parse_on(&Context { buffer, position })
            .ok()
            .map(|s| s.context.position)
    }

    fn parse(&self, input: &str) -> ParseResult<T> {
        let buffer: Rc<[char]> = input.chars().collect::<Vec<_>>().into();
        self.parse_on(&Context {
            buffer,
            position: 0,
        })
    }
}

impl<T, P: Parser<T> + ?Sized> Parser<T> for Rc<P> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        (**self).parse_on(context)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        (**self).fast_parse_on(buffer, position)
    }
}
