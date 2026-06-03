use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct ToParser<T, P, V> {
    pub delegate: P,
    pub value: V,
    pub delegate_type: PhantomData<T>,
}

impl<T, P, V> Parser<V> for ToParser<T, P, V>
where
    P: Parser<T>,
    T: Debug,
    V: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<V> {
        self.delegate.parse_on(context).map(|s| Success {
            context: s.context,
            value: self.value.clone(),
        })
    }
}
