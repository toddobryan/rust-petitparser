use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct PositionParser;

impl Parser<usize> for PositionParser {
    fn parse_on(&self, context: &Context) -> ParseResult<usize> {
        context.success(context.position)
    }

    fn fast_parse_on(&self, _buffer: Rc<[char]>, position: usize) -> Option<usize> {
        Some(position)
    }
}

pub fn position() -> PositionParser {
    PositionParser
}
