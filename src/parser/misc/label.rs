use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Clone, Debug)]
pub struct LabeledParser<P, T> {
    pub delegate: P,
    pub label: String,
    pub delegate_type: PhantomData<T>,
}

impl <P, T> Parser<T> for LabeledParser<P, T>
where
    P: Parser<T>,
    T: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate.parse_on(context).map_err(|mut f| {
            f.message = self.label.clone();
            f
        })
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.delegate.fast_parse_on(buffer, position)
    }
}
