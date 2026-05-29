use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};

// TODO rewrite all of these as RepeatParser

#[derive(Clone, Debug)]
pub struct StarParser<P> {
    pub parser: P,
}

impl <T, P> Parser<Vec<T>> for StarParser<P> where P: Parser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<T>> {
        let mut matches = Vec::new();
        let mut ctx = context.clone();
        loop {
            match self.parser.parse_on(&ctx) {
                Err(_) => break,
                Ok(s) => {
                    if s.context.position == ctx.position {
                        break;
                    }
                    ctx = s.context;
                    matches.push(s.value);
                }
            }
        }
        Ok(Success { context: ctx, value: matches })
    }
}

#[derive(Debug, Clone)]
pub struct PlusParser<P> {
    pub parser: P,
}

impl <T, P> Parser<Vec<T>> for PlusParser<P> where P: Parser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<T>> {
        let first = self.parser.parse_on(context)?;
        let mut ctx = first.context;
        let mut matches: Vec<T> = vec![first.value];
        loop {
            match self.parser.parse_on(&ctx) {
                Err(_) => break,
                Ok(s) => {
                    if s.context.position == ctx.position {
                        break;
                    }
                    ctx = s.context;
                    matches.push(s.value);
                }
            }
        }
        Ok(Success { context: ctx, value: matches })
    }
}

#[derive(Debug, Clone)]
pub struct OptParser<P> {
    pub parser: P,
}

impl <T, P> Parser<Option<T>> for OptParser<P> where P: Parser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<Option<T>> {
        match self.parser.parse_on(context) {
            Err(_) => Ok(Success { context: context.clone(), value: None }),
            Ok(s) => Ok(Success { context: s.context, value: Some(s.value) }),
        }
    }
}
