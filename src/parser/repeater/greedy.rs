use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::core::{
    context::{Context, HasContext},
    kind::ParserKind,
    parser::{HasChildren, Parser},
    result::ParseResult,
};

#[derive(Clone, Debug)]
pub struct GreedyRepeatingParser<T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub limit: Rc<dyn Parser<()>>,
    pub min: usize,
    pub max: Option<usize>,
    pub delegate_type: PhantomData<T>,
}

impl<T: Debug + 'static> HasChildren for GreedyRepeatingParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone(), self.limit.clone()]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::GreedyRepeating {
            min: self.min,
            max: self.max,
        }
    }

    fn is_repeating(&self) -> bool {
        true
    }

    fn is_directly_nullable(&self) -> bool {
        self.min == 0
    }

    fn repeating_min(&self) -> Option<usize> {
        Some(self.min)
    }
}

impl<T> Parser<Vec<T>> for GreedyRepeatingParser<T>
where
    T: Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<T>> {
        let mut elements: Vec<T> = Vec::new();
        let mut current = context.clone();

        while elements.len() < self.min {
            let result = self.delegate.parse_on(&current)?;
            assert!(
                current.position() < result.position(),
                "{:?} must always consume",
                self.delegate
            );
            elements.push(result.value);
            current = result.context;
        }
        let mut contexts = vec![current.clone()];
        while self.max.is_none() || elements.len() < self.max.unwrap() {
            match self.delegate.parse_on(&current) {
                Ok(result) => {
                    assert!(
                        current.position() < result.position(),
                        "{:?} must always consume",
                        self.delegate,
                    );
                    elements.push(result.value);
                    current = result.context;
                    contexts.push(current.clone());
                }
                Err(_) => break,
            }
        }
        loop {
            let limiter = self.limit.parse_on(contexts.last().unwrap());
            match limiter {
                Ok(_) => return contexts.last().unwrap().success(elements),
                Err(f) => {
                    if elements.is_empty() {
                        return Err(f);
                    }
                    contexts.pop();
                    elements.pop();
                    if contexts.is_empty() {
                        return Err(f);
                    }
                }
            }
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let mut count = 0;
        let mut current: usize = context.position;
        while count < self.min {
            let result = self
                .delegate
                .fast_parse_on(&context.with_position(current))?;
            assert!(current < result, "{:?} must always consume", self.delegate);
            current = result;
            count += 1;
        }
        let mut positions: Vec<usize> = vec![current];
        while self.max.is_none() || count < self.max.unwrap() {
            let result = self.delegate.fast_parse_on(&context.with_position(current));
            if result.is_none() {
                break;
            }
            let result = result.unwrap();
            assert!(current < result, "{:?} must always consume", self.delegate);
            current = result;
            positions.push(current);
            count += 1;
        }
        loop {
            let limiter = self
                .limit
                .fast_parse_on(&context.with_position(*positions.last().unwrap()));
            match limiter {
                None => {
                    if count == 0 {
                        return None;
                    }
                    positions.pop();
                    count -= 1;
                    if positions.is_empty() {
                        return None;
                    }
                }
                Some(_) => return positions.pop(),
            }
        }
    }
}
