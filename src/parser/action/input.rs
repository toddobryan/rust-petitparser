use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{Failure, ParseResult, Success};

#[derive(Clone, Debug)]
pub struct InputParser<P, T: Debug> {
    pub delegate: P,
    pub message: Option<String>,
    pub delegate_type: PhantomData<T>,
}

impl <P, T: Debug> Parser<String> for InputParser<P, T> where P: Parser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        match &self.message {
            Some(m) => {
                let position = 
                    self.delegate.fast_parse_on(context.buffer.clone(), context.position);
                match position {
                    None => Err(Failure { message: m.clone(), context: context.clone() }),
                    Some(pos) => Ok(Success { context: Context { buffer: context.buffer.clone(), position: pos }, value: context.buffer[context.position..pos].iter().collect() }),
                }
            }
            None => {
                let result = self.delegate.parse_on(context);
                match result {
                    Err(f) => Err(f),
                    Ok(s) => {
                        let substring = context.buffer[context.position.. s.context.position].iter().collect::<String>();
                        Ok(Success { context: s.context.clone(), value: substring })
                    }
                }
            }
        }
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.delegate.fast_parse_on(buffer, position)
    }
}