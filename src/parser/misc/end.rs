use crate::core::context::{Context, HasContext};
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::rc::Rc;

pub fn eof() -> EndOfInputParser {
    EndOfInputParser {
        message: "Expected end of input".to_string(),
    }
}

pub fn eof_with_message(message: String) -> EndOfInputParser {
    EndOfInputParser { message }
}

#[derive(Clone, Debug)]
pub struct EndOfInputParser {
    pub message: String,
}

impl HasChildren for EndOfInputParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::EndOfInput {
            message: self.message.clone(),
        }
    }
}

impl Parser<()> for EndOfInputParser {
    fn parse_on(&self, context: &Context) -> ParseResult<()> {
        if context.position() < context.buffer().len() {
            context.failure(self.message.clone())
        } else {
            context.success(())
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        if context.position < context.buffer.len() {
            None
        } else {
            Some(context.position)
        }
    }
}
