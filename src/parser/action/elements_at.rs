use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::core::{
    context::Context,
    kind::ParserKind,
    parser::{HasChildren, Parser},
    result::{ParseResult, Success},
};

#[derive(Clone, Debug)]
pub struct ElementsAtParser<I, T> {
    pub delegate: Rc<dyn Parser<I>>,
    pub indexes: Vec<i32>,
    pub iterator_type: PhantomData<I>,
    pub delegate_type: PhantomData<T>,
}

impl<I: Debug + 'static, T: Debug> HasChildren for ElementsAtParser<I, T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::ElementsAt {
            indexes: self.indexes.clone(),
        }
    }

    fn is_elements_at_parser(&self) -> bool {
        true
    }
}

impl<I, T> Parser<Vec<T>> for ElementsAtParser<I, T>
where
    I: Debug + IntoIterator<Item = T> + 'static,
    T: Clone + Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<T>> {
        let success: Success<I> = self.delegate.parse_on(context)?;
        let values: Vec<T> = success.value.into_iter().collect();
        let mut return_values: Vec<T> = Vec::new();
        for index in self.indexes.iter() {
            let norm_index: usize = if *index < 0 {
                (values.len() as i32 + *index) as usize
            } else {
                *index as usize
            };
            return_values.push(values[norm_index].clone());
        }
        Ok(Success {
            context: success.context,
            value: return_values,
        })
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
