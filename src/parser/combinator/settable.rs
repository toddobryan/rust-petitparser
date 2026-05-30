use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

// This can leak via Rc cycle
pub struct SettableParser<T> {
    pub delegate: Rc<RefCell<Option<Rc<dyn Parser<T>>>>>,
}

impl<T> Debug for SettableParser<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Settable - add Debug eventually")
    }
}

impl<T> SettableParser<T> {
    pub fn undefined() -> Self {
        SettableParser {
            delegate: Rc::new(RefCell::new(None)),
        }
    }

    pub fn set(&mut self, delegate: impl Parser<T> + 'static) {
        self.delegate.replace(Some(Rc::new(delegate)));
    }
}

impl<T> Clone for SettableParser<T> {
    fn clone(&self) -> Self {
        SettableParser {
            delegate: self.delegate.clone(),
        }
    }
}

impl<T> Parser<T> for SettableParser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate
            .borrow()
            .as_ref()
            .expect("SettableParser delegate not set")
            .parse_on(context)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.delegate
            .borrow()
            .as_ref()
            .expect("SettableParser delegate not set")
            .fast_parse_on(buffer, position)
    }
}
