use crate::core::context::Context;
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::{Failure, ParseResult, Success};
use crate::prelude::HasContext;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct AndParser<T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub delegate_type: PhantomData<T>,
}

impl<T: Debug + 'static> HasChildren for AndParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::And
    }
}

impl<T> Parser<T> for AndParser<T>
where
    T: Debug + 'static,
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
pub struct NotParser<T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub delegate_type: PhantomData<T>,
    pub message: String,
}

impl<T: Debug + 'static> HasChildren for NotParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::Not {
            message: self.message.clone(),
        }
    }
}

impl<T> Parser<Failure> for NotParser<T>
where
    T: Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Failure> {
        let result = self.delegate.parse_on(context);
        match result {
            Ok(_) => context.failure(self.message.clone()),
            Err(failure) => context.success(failure),
        }
    }
}
