use crate::core::context::{Context, HasContext};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct NewlineParser {
    pub message: Option<String>,
}

impl HasChildren for NewlineParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }
}

impl Parser<String> for NewlineParser {
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        let position = context.position;
        let buffer: &[char] = &context.buffer;
        if position < buffer.len() {
            if buffer[position] == '\n' {
                return context.success_with_position("\n".to_string(), position + 1);
            } else if buffer[position] == '\r' {
                return if position + 1 < buffer.len() && buffer[position + 1] == '\n' {
                    context.success_with_position("\r\n".to_string(), position + 2)
                } else {
                    context.success_with_position("\r".to_string(), position + 1)
                };
            }
        }
        context.failure(
            self.message
                .clone()
                .unwrap_or("newline expected".to_string()),
        )
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let position = context.position;
        let buffer = context.buffer.clone();
        if position < buffer.len() {
            if buffer[position] == '\n' {
                return Some(position + 1);
            } else if buffer[position] == '\r' {
                return if position + 1 < buffer.len() && buffer[position + 1] == '\n' {
                    Some(position + 2)
                } else {
                    Some(position + 1)
                };
            }
        }
        None
    }
}

pub fn newline() -> NewlineParser {
    NewlineParser { message: None }
}
