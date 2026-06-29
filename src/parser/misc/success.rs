use crate::core::context::{Context, HasContext};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SuccessParser<T: Clone + Debug> {
    result: T,
}

impl<T: Clone + Debug> HasChildren for SuccessParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }
}

impl<T> Parser<T> for SuccessParser<T>
where
    T: Clone + Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        context.success(self.result.clone())
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        Some(context.position)
    }
}

pub fn success<T: Clone + Debug>(result: T) -> SuccessParser<T> {
    SuccessParser { result }
}
