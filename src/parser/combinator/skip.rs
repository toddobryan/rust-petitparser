use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct SkipParser<A, Aft, B, Bef, P, T> {
    pub delegate: P,
    pub before: B,
    pub after: A,
    pub delegate_type: PhantomData<T>,
    pub before_type: PhantomData<Bef>,
    pub after_type: PhantomData<Aft>,
}

impl<A, Aft, B, Bef, P, T> Parser<T> for SkipParser<A, Aft, B, Bef, P, T>
where
    P: Parser<T>,
    B: Parser<Bef>,
    A: Parser<Aft>,
    T: Debug,
    Bef: Debug,
    Aft: Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        let before = self.before.parse_on(context)?;
        let result = self.delegate.parse_on(&before.context)?;

        let after = self.after.parse_on(&result.context)?;
        after
            .context
            .success_with_position(result.value, after.context.position)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let before = self.before.fast_parse_on(context)?;
        let result = self.delegate.fast_parse_on(&Context {
            text: context.text.clone(),
            buffer: context.buffer.clone(),
            position: before,
        })?;
        self.after.fast_parse_on(&Context {
            text: context.text.clone(),
            buffer: context.buffer.clone(),
            position: result,
        })
    }
}
