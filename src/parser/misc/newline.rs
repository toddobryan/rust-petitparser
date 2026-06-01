use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct NewlineParser {
    pub message: Option<String>,
}

impl Parser<String> for NewlineParser {
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        let position = context.position;
        let buffer: &[char] = &context.buffer;
        if position < buffer.len() {
            if buffer[position] == '\n' {
                return context.success("\n".to_string(), position + 1);
            } else if buffer[position] == '\r' && buffer[position + 1] == '\n' {
                return context.success("\r\n".to_string(), position + 2);
            } else if buffer[position] == '\r' {
                return context.success("\r".to_string(), position + 1);
            }
        }
        context.failure(
            self.message
                .clone()
                .unwrap_or("newline expected".to_string()),
            position,
        )
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        if position < buffer.len() {
            if buffer[position] == '\n' {
                return Some(position + 1);
            } else if buffer[position] == '\r' {
                return if position + 1 < buffer.len() && buffer[position + 1] == '\n' {
                    Some(position + 2)
                } else {
                    Some(position + 1)
                }
            }
        }
        None
    }
}

pub fn newline() -> NewlineParser {
    NewlineParser { message: None }
}
