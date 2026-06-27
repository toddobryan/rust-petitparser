use std::fmt::Debug;
use std::marker::PhantomData;

use crate::core::{
    context::Context,
    parser::Parser,
    result::{ParseResult, Success},
};

#[derive(Clone, Debug)]
pub struct PickParser<P, I, T> {
    pub delegate: P,
    pub index: i32,
    pub iterator_type: PhantomData<I>,
    pub delegate_type: PhantomData<T>,
}

impl<P, I, T> Parser<T> for PickParser<P, I, T>
where
    P: Parser<I>,
    I: Debug + IntoIterator<Item = T>,
    T: Clone + Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        let success = self.delegate.parse_on(context)?;
        let values: Vec<T> = success.value.into_iter().collect();
        let norm_index: usize = if self.index < 0 {
            (values.len() as i32 + self.index) as usize
        } else {
            self.index as usize
        };
        Ok(Success {
            context: success.context,
            value: values[norm_index].clone(),
        })
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
