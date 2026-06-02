use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct FlatMapParser<P, P2, T, T2> {
    pub delegate: P,
    pub f: fn(&T) -> P2,
    pub delegate_type: PhantomData<T>,
    pub result_type: PhantomData<T2>,
}

impl<P, P2, T, T2> Parser<T2> for FlatMapParser<P, P2, T, T2>
where
    P: Parser<T>,
    P2: Parser<T2>,
    T: Clone + Debug,
    T2: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T2> {
        let result = self.delegate.parse_on(context)?;
        let mapped_result = (self.f)(&result.value).parse_on(&result.context)?;
        Ok(mapped_result)
    }
}
