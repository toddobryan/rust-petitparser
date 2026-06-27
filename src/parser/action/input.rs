use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct InputParser<P, T: Debug> {
    pub delegate: P,
    pub message: Option<String>,
    pub delegate_type: PhantomData<T>,
}

impl<P, T: Debug> Parser<String> for InputParser<P, T>
where
    P: Parser<T>,
{
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        match &self.message {
            Some(m) => {
                let position = self.delegate.fast_parse_on(context);
                match position {
                    None => context.failure(m.clone()),
                    Some(pos) => context.success_with_position(
                        context.buffer()[context.position()..pos].iter().collect(),
                        pos,
                    ),
                }
            }
            None => {
                let result = self.delegate.parse_on(context);
                match result {
                    Err(f) => Err(f),
                    Ok(s) => {
                        let substring = context.buffer()[context.position()..s.context.position]
                            .iter()
                            .collect::<String>();
                        Ok(Success {
                            context: s.context.clone(),
                            value: substring,
                        })
                    }
                }
            }
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
