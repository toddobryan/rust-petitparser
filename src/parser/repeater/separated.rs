use crate::core::context::{Context, HasContext};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::{ParseResult, Success};
use std::fmt::{Debug, Display};
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

#[derive(Clone, Debug, PartialEq)]
pub enum Interleaved<T, S> {
    Element(T),
    Separator(S),
}

impl<T, Sep> SeparatedList<T, Sep> {
    pub fn sequential(&self) -> impl Iterator<Item = Interleaved<&T, &Sep>> {
        let mut interleaved: Vec<Interleaved<&T, &Sep>> = Vec::new();
        let mut i: usize = 0;
        while i < self.elements.len() {
            interleaved.push(Interleaved::Element(&self.elements[i]));
            if i < self.separators.len() {
                interleaved.push(Interleaved::Separator(&self.separators[i]));
            }
            i += 1;
        }
        interleaved.into_iter()
    }

    /// folds over the list of elements and separators; requires there to
    /// be one more element than separator
    pub fn fold(self, f: impl Fn(T, Sep, T) -> T) -> T {
        assert!(
            !self.elements.is_empty(),
            "Can't call fold on an empty SeparatedList"
        );
        assert!(
            self.elements.len() - 1 == self.separators.len(),
            "Can't call fold unless there is exactly one more element than separator"
        );
        let mut elements = self.elements.into_iter();
        let mut separators = self.separators.into_iter();
        let mut result = elements.next().unwrap();
        loop {
            let s = separators.next();
            let e = elements.next();
            match (s, e) {
                (Some(s), Some(e)) => result = f(result, s, e),
                _ => break,
            }
        }
        result
    }

    /// folds over the list of elements and separators back to front;
    /// requires there to be one more element than separator
    pub fn rfold(self, f: impl Fn(T, Sep, T) -> T) -> T {
        assert!(
            !self.elements.is_empty(),
            "Can't call rfold on an empty SeparatedList"
        );
        assert!(
            self.elements.len() - 1 == self.separators.len(),
            "Can't call rfold unless there is exactly one more element than separator"
        );
        let mut elements = self.elements.into_iter();
        let mut separators = self.separators.into_iter();
        let mut result = elements.next_back().unwrap();
        loop {
            let s = separators.next_back();
            let e = elements.next_back();
            match (s, e) {
                (Some(s), Some(e)) => result = f(e, s, result),
                _ => break,
            }
        }
        result
    }
}

impl<T: Display, Sep: Display> Display for SeparatedList<T, Sep> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SeparatedList({})",
            self.sequential()
                .map(|e_or_s| match e_or_s {
                    Interleaved::Element(e) => format!("{e}"),
                    Interleaved::Separator(s) => format!("{s}"),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Debug)]
pub struct SeparatedListRepeatingParser<T, Sep> {
    pub delegate: Rc<dyn Parser<T>>,
    pub separator: Rc<dyn Parser<Sep>>,
    pub min: usize,
    pub max: Option<usize>,
    pub trailing: Trailing,
    pub delegate_type: PhantomData<T>,
    pub separator_type: PhantomData<Sep>,
}

impl<T: Debug + 'static, Sep: Debug + 'static> HasChildren
    for SeparatedListRepeatingParser<T, Sep>
{
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone(), self.separator.clone()]
    }
}

impl<T, Sep> Parser<SeparatedList<T, Sep>> for SeparatedListRepeatingParser<T, Sep>
where
    T: Debug + 'static,
    Sep: Debug + 'static,
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

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let mut count: usize = 0;
        let mut current: usize = context.position;
        while count < self.min {
            if count > 0 {
                let sep = self
                    .separator
                    .fast_parse_on(&context.with_position(current))?;
                current = sep;
            }
            let result = self
                .delegate
                .fast_parse_on(&context.with_position(current))?;
            count += 1;
            current = result;
        }
        while self.max.is_none() || count < self.max.unwrap() {
            let previous = current;
            if count > 0 {
                let sep = self
                    .separator
                    .fast_parse_on(&context.with_position(current));
                match sep {
                    None => break,
                    Some(pos) => current = pos,
                }
            }
            let result = self.delegate.fast_parse_on(&context.with_position(current));
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
                    let trailing_separator: Option<usize> = self
                        .separator
                        .fast_parse_on(&context.with_position(current));
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
