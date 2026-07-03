use crate::core::context::{Context, HasContext};
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct PositionParser;

impl HasChildren for PositionParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::Position
    }

    fn is_directly_nullable(&self) -> bool {
        true
    }
}

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
