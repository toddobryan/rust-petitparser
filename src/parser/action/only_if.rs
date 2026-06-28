use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{Failure, ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

type FactoryFn<T> = fn(&Context, Success<T>) -> ParseResult<T>;

#[derive(Clone, Debug)]
pub struct OnlyIfParser<T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub predicate: fn(&T) -> bool,
    pub message: Option<String>,
    pub factory: Option<FactoryFn<T>>,
    pub delegate_type: PhantomData<T>,
}

impl<T> OnlyIfParser<T>
where
    T: Debug + 'static,
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

impl<T> Parser<T> for OnlyIfParser<T>
where
    T: Clone + Debug + 'static,
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
