use crate::core::context::Context;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

pub trait Parser<T: 'static>: Debug + 'static {
    fn parse_on(&self, context: &Context) -> ParseResult<T>;

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.parse_on(context).ok().map(|s| s.context.position)
    }

    fn parse(&self, input: &str) -> ParseResult<T> {
        let buffer: Rc<[char]> = input.chars().collect::<Vec<_>>().into();
        self.parse_on(&Context {
            text: Rc::new(String::from(input)),
            buffer,
            position: 0,
        })
    }
}

impl<T: 'static, P: Parser<T> + ?Sized> Parser<T> for Rc<P> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        (**self).parse_on(context)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        (**self).fast_parse_on(context)
    }
}
