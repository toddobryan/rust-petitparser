use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Clone, Debug)]
pub struct PositionParser;

impl Parser<usize> for PositionParser {
    fn parse_on(&self, context: &Context) -> ParseResult<usize> {
        context.success(context.position, context.position)
    }

    fn fast_parse_on(&self, _buffer: Rc<[char]>, position: usize) -> Option<usize> {
        Some(position)
    }
}

pub fn position() -> PositionParser {
    PositionParser
}