use crate::core::context::Context;
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::{ParseResult, Success};
use crate::core::token::Token;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct TokenParser<T> {
    pub parser: Rc<dyn Parser<T>>,
}

impl<T: Debug + 'static> HasChildren for TokenParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.parser.clone()]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Token
    }

    fn is_token_parser(&self) -> bool {
        true
    }
}

impl<T> Parser<Token<T>> for TokenParser<T>
where
    T: Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Token<T>> {
        match self.parser.parse_on(context) {
            Err(e) => Err(e),
            Ok(s) => Ok(Success {
                context: s.context.clone(),
                value: Token::new(s.value, context.clone(), s.context.position),
            }),
        }
    }

    // Building a Token is pure bookkeeping the fast path never needs — it only needs to know
    // where the delegate stopped.
    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.parser.fast_parse_on(context)
    }
}
