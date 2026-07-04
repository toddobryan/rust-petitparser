use crate::core::context::{Context, HasContext};
use crate::core::kind::{NeverEq, ParserKind};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct EpsilonParser<T: Clone + Debug> {
    pub result: T,
}

impl<T: Clone + Debug> HasChildren for EpsilonParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Epsilon(NeverEq)
    }
}

impl<T> Parser<T> for EpsilonParser<T>
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

pub fn epsilon() -> EpsilonParser<()> {
    EpsilonParser { result: () }
}

pub fn epsilon_with<T: Clone + Debug>(result: T) -> EpsilonParser<T> {
    EpsilonParser { result }
}
