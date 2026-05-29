use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};

#[derive(Clone)]
pub struct MapParser<T, P, F> {
    pub delegate: P,
    pub f: F,
    pub from_type: PhantomData<T>,
}

impl <T, U, P, F> Debug for MapParser<T, P, F> where P: Parser<T>, F: Fn(T) -> U {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapParser")
            .field("delegate", &self.delegate)
            .field("f", &"<mapping function>")
            .field("from_type", &self.from_type)
            .finish()
    }
}

impl <T, U, P, F> Parser<U> for MapParser<T, P, F>
where
    P: Parser<T>,
    F: Fn(T) -> U,
{
    fn parse_on(&self, context: &Context) -> ParseResult<U> {
        self.delegate.parse_on(context).map(|s| Success {
            context: s.context,
            value: (self.f)(s.value),
        })
    }
}

