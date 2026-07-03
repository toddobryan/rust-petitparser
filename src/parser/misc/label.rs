use crate::core::context::Context;
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LabeledParser<T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub label: String,
    pub delegate_type: PhantomData<T>,
}

impl<T: Clone + Debug + 'static> HasChildren for LabeledParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::Labeled {
            label: self.label.clone(),
        }
    }
}

impl<T> Parser<T> for LabeledParser<T>
where
    T: Clone + Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate.parse_on(context).map_err(|mut f| {
            f.message = self.label.clone();
            f
        })
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
