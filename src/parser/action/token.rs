use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};
use crate::core::token::Token;

#[derive(Clone, Debug)]
pub struct TokenParser<P> {
    pub parser: P,
}

impl <T, P> Parser<Token<T>> for TokenParser<P> where P: Parser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<Token<T>> {
        match self.parser.parse_on(context) {
            Err(e) => Err(e),
            Ok(s) => Ok(Success {
                context: s.context.clone(),
                value: Token::new(
                    s.value,
                    context.buffer.clone(),
                    context.position,
                    s.context.position
                )
            }),
        }
    }
}
