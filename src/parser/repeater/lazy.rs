use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LazyRepeatingParser<T, TC> {
    pub delegate: Rc<dyn Parser<T>>,
    pub limit: Rc<dyn Parser<TC>>,
    pub min: usize,
    pub max: Option<usize>,
    pub delegate_type: PhantomData<T>,
    pub limit_type: PhantomData<TC>,
}

impl<T, TC> Parser<Vec<T>> for LazyRepeatingParser<T, TC>
where
    T: Debug + 'static,
    TC: Debug + 'static,
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
        loop {
            let limiter = self.limit.parse_on(&current);
            match limiter {
                Ok(_) => return current.success(elements),
                Err(f) => {
                    if self.max.is_some() && elements.len() >= self.max.unwrap() {
                        return Err(f);
                    } else {
                        let result = self.delegate.parse_on(&current);
                        match result {
                            Err(_) => return Err(f),
                            Ok(success) => {
                                assert!(
                                    current.position() < success.position(),
                                    "Delegate parser {:?} must always consume",
                                    self.delegate
                                );
                                current = success.context;
                                elements.push(success.value);
                            }
                        }
                    }
                }
            }
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let mut count: usize = 0;
        let mut current: usize = context.position;
        while count < self.min {
            let result = self
                .delegate
                .fast_parse_on(&context.with_position(current))?;
            assert!(current < result, "{:?} must always consume", self.delegate);
            current = result;
            count += 1;
        }
        loop {
            let limiter = self.limit.fast_parse_on(&context.with_position(current));
            match limiter {
                Some(_) => return Some(current),
                None => {
                    if self.max.is_some() && count >= self.max.unwrap() {
                        return None;
                    }
                    let result = self
                        .delegate
                        .fast_parse_on(&context.with_position(current))?;
                    assert!(current < result, "{:?} must always consume", self.delegate);
                    current = result;
                    count += 1;
                }
            }
        }
    }
}
