use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{Failure, ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct OnlyIfParser<P, T> {
    pub delegate: P,
    pub predicate: fn(&T) -> bool,
    pub message: Option<String>,
    pub factory: Option<fn(&Context, Success<T>) -> ParseResult<T>>,
    pub delegate_type: PhantomData<T>,
}

impl<P, T> OnlyIfParser<P, T> where P: Parser<T>, T: Debug
{
    fn make_factory(&self, context: &Context, result: Success<T>) -> ParseResult<T> {
        match self.factory {
            Some(factory) => factory(context, result),
            None => Err(Failure {
                context: context.clone(),
                message: self
                    .message
                    .clone()
                    .unwrap_or(format!("unexpected \"{:?}\"", result.value)),
            }),
        }
    }
}

impl<P, T> Parser<T> for OnlyIfParser<P, T>
where
    P: Parser<T>,
    T: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        let result = self.delegate.parse_on(context)?;
        if !(self.predicate)(&result.value) {
            self.make_factory(context, result)
        } else {
            Ok(result)
        }
    }
}
