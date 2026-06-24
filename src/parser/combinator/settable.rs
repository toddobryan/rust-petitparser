use crate::core::context::{Context, HasContext};
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

/// The default delegate of an `undefined()` `SettableParser` — always fails with a
/// configurable message, rather than the delegate slot being absent. This means
/// `SettableParser`/`SettableParserRef` never need to handle a "not set yet" case: the
/// slot always holds *some* `Rc<dyn Parser<T>>`, so a forgotten `.set()` call surfaces as
/// an ordinary parse failure instead of a panic.
struct UndefinedParser<T> {
    message: String,
    result_type: PhantomData<T>,
}

impl<T> Debug for UndefinedParser<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndefinedParser")
            .field("message", &self.message)
            .finish()
    }
}

impl<T> Clone for UndefinedParser<T> {
    fn clone(&self) -> Self {
        UndefinedParser {
            message: self.message.clone(),
            result_type: PhantomData,
        }
    }
}

impl<T> Parser<T> for UndefinedParser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        context.failure(self.message.clone())
    }
}

pub struct SettableParser<T> {
    pub delegate: Rc<RefCell<Rc<dyn Parser<T>>>>,
}

impl<T> Debug for SettableParser<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Settable - add Debug eventually")
    }
}

impl<T: 'static> SettableParser<T> {
    pub fn undefined() -> Self {
        Self::undefined_with_message("undefined parser".to_string())
    }

    pub fn undefined_with_message(message: String) -> Self {
        SettableParser {
            delegate: Rc::new(RefCell::new(Rc::new(UndefinedParser {
                message,
                result_type: PhantomData,
            }))),
        }
    }

    pub fn set(&mut self, delegate: impl Parser<T> + 'static) {
        self.delegate.replace(Rc::new(delegate));
    }

    pub fn borrow(&self) -> SettableParserRef<T> {
        SettableParserRef {
            delegate: Rc::downgrade(&self.delegate),
        }
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
        self.delegate.borrow().parse_on(context)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.delegate.borrow().fast_parse_on(buffer, position)
    }
}

/// A weak reference to a `SettableParser`, used when embedding a recursive parser
/// inside other parsers. Holds a `Weak` pointer to break the `Rc` cycle that would
/// otherwise prevent the grammar from being dropped.
pub struct SettableParserRef<T> {
    pub delegate: Weak<RefCell<Rc<dyn Parser<T>>>>,
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
        }
    }
}

impl<T> Parser<T> for SettableParserRef<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate
            .upgrade()
            .expect("SettableParser owner dropped")
            .borrow()
            .parse_on(context)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        self.delegate
            .upgrade()
            .expect("SettableParser owner dropped")
            .borrow()
            .fast_parse_on(buffer, position)
    }
}
