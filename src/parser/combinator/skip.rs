use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SkipParser<Aft, Bef, T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub before: Rc<dyn Parser<Bef>>,
    pub after: Rc<dyn Parser<Aft>>,
    pub delegate_type: PhantomData<T>,
    pub before_type: PhantomData<Bef>,
    pub after_type: PhantomData<Aft>,
}

impl<Aft, Bef, T> Parser<T> for SkipParser<Aft, Bef, T>
where
    T: Debug + 'static,
    Bef: Debug + 'static,
    Aft: Debug + 'static,
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
