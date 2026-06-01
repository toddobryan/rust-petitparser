use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;

#[derive(Clone, Debug)]
pub struct SkipParser<A, Aft, B, Bef, P, T> {
    pub delegate: P,
    pub before: B,
    pub after: A,
    pub delegate_type: PhantomData<T>,
    pub before_type: PhantomData<Bef>,
    pub after_type: PhantomData<Aft>,
}

impl <A, Aft, B, Bef, P, T> Parser<T> for SkipParser<A, Aft, B, Bef, P, T>
where
    P: Parser<T>,
    B: Parser<Bef>,
    A: Parser<Aft>,
    T: Clone + Debug,
    Bef: Clone + Debug,
    Aft: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        let before = self.before.parse_on(context)?;
        let result = self.delegate.parse_on(&before.context)?;

        let after = self.after.parse_on(&result.context)?;
        after.context.success(result.value, after.context.position)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        let before = self.before.fast_parse_on(buffer.clone(), position)?;
        let result = self.delegate.fast_parse_on(buffer.clone(), before)?;
        self.after.fast_parse_on(buffer.clone(), result)
    }
}