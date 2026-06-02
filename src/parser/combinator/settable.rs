use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

pub struct SettableParser<T> {
    pub delegate: Rc<RefCell<Option<Rc<dyn Parser<T>>>>>,
    pub delegate_type: PhantomData<T>,
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
            delegate_type: PhantomData,
        }
    }

    pub fn set(&mut self, delegate: impl Parser<T> + 'static) {
        self.delegate.replace(Some(Rc::new(delegate)));
    }

    pub fn borrow(&self) -> SettableParserRef<T> {
        SettableParserRef {
            delegate: Rc::downgrade(&self.delegate),
            delegate_type: PhantomData,
        }
    }
}

impl<T> Clone for SettableParser<T> {
    fn clone(&self) -> Self {
        SettableParser {
            delegate: self.delegate.clone(),
            delegate_type: PhantomData,
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

/// A weak reference to a `SettableParser`, used when embedding a recursive parser
/// inside other parsers. Holds a `Weak` pointer to break the `Rc` cycle that would
/// otherwise prevent the grammar from being dropped.
pub struct SettableParserRef<T> {
    pub delegate: Weak<RefCell<Option<Rc<dyn Parser<T>>>>>,
    pub delegate_type: PhantomData<T>,
}

impl<T> Debug for SettableParserRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("SettableRef - add Debug eventually")
    }
}

impl<T> Clone for SettableParserRef<T> {
    fn clone(&self) -> Self {
        SettableParserRef {
            delegate: self.delegate.clone(),
            delegate_type: PhantomData,
        }
    }
}

impl<T> Parser<T> for SettableParserRef<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate
            .upgrade()
            .expect("SettableParser owner dropped")
            .borrow()
            .as_ref()
            .expect("SettableParser delegate not set")
            .parse_on(context)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.delegate
            .upgrade()
            .expect("SettableParser owner dropped")
            .borrow()
            .as_ref()
            .expect("SettableParser delegate not set")
            .fast_parse_on(buffer, position)
    }
}
