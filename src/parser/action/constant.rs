use crate::core::context::Context;
use crate::core::kind::{NeverEq, ParserKind};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::{ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ConstantParser<T, V> {
    pub delegate: Rc<dyn Parser<T>>,
    pub value: V,
    pub delegate_type: PhantomData<T>,
}

impl<T: Debug + 'static, V: Debug> HasChildren for ConstantParser<T, V> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Constant(NeverEq)
    }
}

impl<T, V> Parser<V> for ConstantParser<T, V>
where
    T: Debug + 'static,
    V: Clone + Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<V> {
        self.delegate.parse_on(context).map(|s| Success {
            context: s.context,
            value: self.value.clone(),
        })
    }

    // The replacement value doesn't depend on the delegate's value, so the fast path never
    // needs to clone `self.value` — it only needs to know where the delegate stopped.
    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
