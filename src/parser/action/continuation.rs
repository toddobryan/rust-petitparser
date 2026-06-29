use crate::core::parser::HasChildren;
use crate::prelude::{Context, ParseResult, Parser};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

type ContFnRc<T> = Rc<dyn Fn(&Context) -> ParseResult<T>>;

#[derive(Clone)]
pub struct Continuation<T> {
    f: ContFnRc<T>,
}

impl<T> Continuation<T> {
    pub fn resume(&self, context: &Context) -> ParseResult<T> {
        (*self.f)(context)
    }
}

impl<T> Debug for Continuation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Continuation(<function>)")
    }
}

#[derive(Clone)]
pub struct ContinuationParser<T, T2, H> {
    pub delegate: Rc<dyn Parser<T>>,
    pub handler: H,
    pub result_type: PhantomData<T2>,
}

impl<T, T2, H> Debug for ContinuationParser<T, T2, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContinuationParser")
            .field("delegate", &self.delegate)
            .field("handler", &format_args!("<function>"))
            .finish_non_exhaustive()
    }
}

impl<T: 'static, T2, H> HasChildren for ContinuationParser<T, T2, H> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }
}

impl<T, T2, H> Parser<T2> for ContinuationParser<T, T2, H>
where
    T: Debug + 'static,
    T2: Debug + 'static,
    H: Fn(Continuation<T>, &Context) -> ParseResult<T2> + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T2> {
        let delegate = self.delegate.clone();
        let continuation = Continuation {
            f: Rc::new(move |ctx: &Context| delegate.parse_on(ctx)),
        };
        (self.handler)(continuation, context)
    }
}
