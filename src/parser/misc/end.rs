use crate::core::context::Context;
use crate::core::parser::Parser;
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

impl Parser<()> for EndOfInputParser {
    fn parse_on(&self, context: &Context) -> ParseResult<()> {
        if context.position < context.buffer.len() {
            context.failure(self.message.clone(), context.position)
        } else {
            context.success((), context.position)
        }
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        if position < buffer.len() {
            None
        } else {
            Some(position)
        }
    }
}
