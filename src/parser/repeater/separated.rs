use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

/// whether in a list of the form elem sep elem sep elem ...
/// the final element must, can, or cannot be followed by a separator
#[derive(Clone, Debug, PartialEq)]
pub enum Trailing {
    Disallowed,
    Allowed,
    Required,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeparatedList<T, Sep> {
    pub elements: Vec<T>,
    pub separators: Vec<Sep>,
}

#[derive(Clone, Debug)]
pub struct SeparatedListRepeatingParser<P, T, S, Sep> {
    pub delegate: P,
    pub separator: S,
    pub min: usize,
    pub max: Option<usize>,
    pub trailing: Trailing,
    pub delegate_type: PhantomData<T>,
    pub separator_type: PhantomData<Sep>,
}

impl<P, T, S, Sep> Parser<SeparatedList<T, Sep>> for SeparatedListRepeatingParser<P, T, S, Sep>
where
    P: Parser<T>,
    S: Parser<Sep>,
    T: Debug,
    Sep: Debug,
{
    fn parse_on(&self, context: &Context) -> ParseResult<SeparatedList<T, Sep>> {
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
                        return match self.trailing {
                            Trailing::Disallowed => {
                                separators.pop();
                                Ok(Success {
                                    context: previous.clone(),
                                    value: SeparatedList {
                                        elements,
                                        separators,
                                    },
                                })
                            }
                            Trailing::Allowed | Trailing::Required => Ok(Success {
                                context: current.clone(),
                                value: SeparatedList {
                                    elements,
                                    separators,
                                },
                            }),
                        };
                    }
                    break;
                }
                Ok(s) => {
                    elements.push(s.value);
                    current = s.context.clone();
                }
            }
        }
        match self.trailing {
            Trailing::Allowed | Trailing::Required => {
                if !elements.is_empty() {
                    let trailing_separator: ParseResult<Sep> = self.separator.parse_on(&current);
                    match trailing_separator {
                        Ok(success) => {
                            separators.push(success.value);
                            current = success.context.clone();
                        },
                        Err(_) if self.trailing == Trailing::Allowed => (),
                        Err(failure) /*Trailing::Required*/ => return Err(failure),
                    }
                }
            }
            Trailing::Disallowed => (),
        }
        Ok(Success {
            context: current.clone(),
            value: SeparatedList {
                elements,
                separators,
            },
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
                None => {
                    return Some(match self.trailing {
                        Trailing::Disallowed => previous,
                        Trailing::Allowed | Trailing::Required => current,
                    });
                }
                Some(pos) => {
                    count += 1;
                    current = pos;
                }
            }
        }
        match self.trailing {
            Trailing::Allowed | Trailing::Required => {
                if count > 0 {
                    let trailing_separator: Option<usize> =
                        self.separator.fast_parse_on(buffer.clone(), current);
                    match trailing_separator {
                        Some(new_position) => {
                            current = new_position;
                        },
                        None if self.trailing == Trailing::Allowed => (),
                        None /*Trailing::Required*/ => return None,
                    }
                }
            }
            Trailing::Disallowed => (),
        }
        Some(current)
    }
}
