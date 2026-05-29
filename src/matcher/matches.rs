use std::marker::PhantomData;
use crate::core::context::Context;
use crate::core::parser::Parser;

pub struct MatchesIterator<T, P> {
    pub parser: P,
    pub context: Context,
    pub overlapping: bool,
    _parsed: PhantomData<T>,
}

impl <T, P> Iterator for MatchesIterator<T, P> where P: Parser<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.context.position < self.context.buffer.len() {
            match self.parser.parse_on(&mut self.context) {
                Ok(s) => {
                    if self.overlapping || self.context.position == s.context.position {
                        self.context.position += 1;
                    } else {
                        self.context.position = s.context.position;
                    }
                    return Some(s.value);
                },
                Err(_) => {
                    self.context.position += 1;
                }
            }
        }
        None
    }
}

pub struct MatchesIterable<T, P> {
    pub parser: P,
    pub context: Context,
    pub overlapping: bool,
    pub parser_type: PhantomData<T>,
}

impl <T, P> IntoIterator for MatchesIterable<T, P> where P: Parser<T> {
    type Item = T;
    type IntoIter = MatchesIterator<Self::Item, P>;

    fn into_iter(self) -> Self::IntoIter {
        MatchesIterator {
            parser: self.parser,
            context: self.context.clone(),
            overlapping: self.overlapping,
            _parsed: PhantomData,
        }
    }
}