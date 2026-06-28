use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone)]
pub struct MapParser<T, F> {
    pub delegate: Rc<dyn Parser<T>>,
    pub f: F,
    pub from_type: PhantomData<T>,
}

impl<T: 'static, F> Debug for MapParser<T, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapParser")
            .field("delegate", &self.delegate)
            .field("f", &"<mapping function>")
            .field("from_type", &self.from_type)
            .finish()
    }
}

impl<T, U, F> Parser<U> for MapParser<T, F>
where
    T: 'static,
    U: 'static,
    F: Fn(T) -> U + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<U> {
        self.delegate.parse_on(context).map(|s| Success {
            context: s.context,
            value: (self.f)(s.value),
        })
    }

    // `fast_parse_on` only needs the resulting position, never the mapped value, so the
    // mapping function `f` never needs to run on this path.
    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
