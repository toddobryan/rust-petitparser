use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{Failure, ParseResult, Success};
use crate::prelude::HasContext;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct AndParser<T, P> {
    pub delegate: P,
    pub delegate_type: PhantomData<T>,
}

impl<T, P> Parser<T> for AndParser<T, P>
where
    P: Parser<T>,
    T: Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        let result = self.delegate.parse_on(context);
        match result {
            Ok(success) => Ok(Success {
                context: context.clone(),
                value: success.value,
            }),
            Err(failure) => Err(failure),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotParser<T, P> {
    pub delegate: P,
    pub delegate_type: PhantomData<T>,
    pub message: String,
}

impl<T, P> Parser<Failure> for NotParser<T, P>
where
    P: Parser<T>,
    T: Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Failure> {
        let result = self.delegate.parse_on(context);
        match result {
            Ok(_) => context.failure(self.message.clone()),
            Err(failure) => context.success(failure),
        }
    }
}
