use crate::core::context::{Context, HasContext};
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct FailureParser {
    message: Option<String>,
}

impl Parser<()> for FailureParser {
    fn parse_on(&self, context: &Context) -> ParseResult<()> {
        context.failure(
            self.message
                .clone()
                .unwrap_or("unable to parse".to_string()),
        )
    }
}

impl HasChildren for FailureParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Failure {
            message: self.message.as_deref(),
        }
    }
}

pub fn failure() -> FailureParser {
    FailureParser { message: None }
}

pub fn failure_with_message(message: String) -> FailureParser {
    FailureParser {
        message: Some(message),
    }
}
