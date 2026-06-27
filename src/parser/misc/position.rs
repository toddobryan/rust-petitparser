use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Clone, Debug)]
pub struct PositionParser;

impl Parser<usize> for PositionParser {
    fn parse_on(&self, context: &Context) -> ParseResult<usize> {
        context.success(context.position)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        Some(context.position)
    }
}

pub fn position() -> PositionParser {
    PositionParser
}
