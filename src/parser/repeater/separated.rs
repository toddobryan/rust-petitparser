use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SeparatedRepeatingParser<P, T, S, Sep> {
    pub delegate: P,
    pub separator: S,
    pub min: usize,
    pub max: Option<usize>,
    pub delegate_type: PhantomData<T>,
    pub separator_type: PhantomData<Sep>,
}

impl<P, T: Debug, S, Sep: Debug> Parser<Vec<T>> for SeparatedRepeatingParser<P, T, S, Sep>
where
    P: Parser<T>,
    S: Parser<Sep>,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<T>> {
        let mut elements: Vec<T> = vec![];
        let mut separators: Vec<Sep> = vec![];
        let mut current: Context = context.clone();
        while elements.len() < self.min {
            if !elements.is_empty() {
                let sep = self.separator.parse_on(&current)?;
                current = sep.context.clone();
                separators.push(sep.value);
            }
            let result = self.delegate.parse_on(&current)?;
            elements.push(result.value);
            current = result.context.clone();
        }
        while self.max.is_none() || elements.len() < self.max.unwrap() {
            let previous = current.clone();
            if !elements.is_empty() {
                let sep = self.separator.parse_on(&current);
                match sep {
                    Err(_) => break,
                    Ok(s) => {
                        separators.push(s.value);
                        current = s.context.clone();
                    }
                }
            }
            let result = self.delegate.parse_on(&current);
            match result {
                Err(_) => {
                    if !elements.is_empty() {
                        separators.pop();
                        return Ok(Success {
                            context: previous.clone(),
                            value: elements,
                        });
                    }
                    break;
                }
                Ok(s) => {
                    elements.push(s.value);
                    current = s.context.clone();
                }
            }
        }
        Ok(Success {
            context: current.clone(),
            value: elements,
        })
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        let mut count: usize = 0;
        let mut current: usize = position;
        while count < self.min {
            if count > 0 {
                let sep = self.separator.fast_parse_on(buffer.clone(), current)?;
                current = sep;
            }
            let result = self.delegate.fast_parse_on(buffer.clone(), current)?;
            count += 1;
            current = result;
        }
        while self.max.is_none() || count < self.max.unwrap() {
            let previous = current;
            if count > 0 {
                let sep = self.separator.fast_parse_on(buffer.clone(), current);
                match sep {
                    None => break,
                    Some(pos) => current = pos,
                }
            }
            let result = self.delegate.fast_parse_on(buffer.clone(), current);
            match result {
                None => return Some(previous),
                Some(pos) => {
                    count += 1;
                    current = pos;
                }
            }
        }
        Some(current)
    }
}
