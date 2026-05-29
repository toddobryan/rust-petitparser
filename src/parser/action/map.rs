use std::marker::PhantomData;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};

pub struct MapParser<T, P, F> {
    pub parser: P,
    pub f: F,
    pub orig_type: PhantomData<T>,
}

impl <T, U, P, F> Parser<U> for MapParser<T, P, F>
where
    P: Parser<T>,
    F: Fn(T) -> U,
{
    fn parse_on(&self, context: &Context) -> ParseResult<U> {
        self.parser.parse_on(context).map(|s| Success {
            context: s.context,
            value: (self.f)(s.value),
        })
    }
}
