use crate::core::context::Context;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

// `P2` is the parser produced on the fly by `f` for each parsed value, not a stored delegate,
// so it stays a generic parameter rather than becoming an `Rc<dyn Parser<..>>` field.
#[derive(Clone, Debug)]
pub struct FlatMapParser<P2, T, T2> {
    pub delegate: Rc<dyn Parser<T>>,
    pub f: fn(&T) -> P2,
    pub delegate_type: PhantomData<T>,
    pub result_type: PhantomData<T2>,
}

impl<P2: Debug, T: Debug + 'static, T2: Debug> HasChildren for FlatMapParser<P2, T, T2> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }
}

impl<P2, T, T2> Parser<T2> for FlatMapParser<P2, T, T2>
where
    P2: Parser<T2>,
    T: Clone + Debug + 'static,
    T2: Clone + Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T2> {
        let result = self.delegate.parse_on(context)?;
        let mapped_result = (self.f)(&result.value).parse_on(&result.context)?;
        Ok(mapped_result)
    }
}
