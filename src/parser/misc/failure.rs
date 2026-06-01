use std::fmt::Debug;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Clone, Debug)]
pub struct FailureParser {
    message: Option<String>,
}

impl Parser<()> for FailureParser {
    fn parse_on(&self, context: &Context) -> ParseResult<()> {
        context.failure(self.message.clone().unwrap_or("unable to parse".to_string()), context.position)
    }
}

pub fn failure() -> FailureParser {
    FailureParser { message: None }
}

pub fn failure_with_message(message: String) -> FailureParser {
    FailureParser { message: Some(message) }
}
