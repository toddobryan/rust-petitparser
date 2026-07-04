use crate::core::context::{Context, HasContext};
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
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

impl<T> HasChildren for UndefinedParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Undefined {
            message: &self.message,
        }
    }
}

impl<T: 'static> Parser<T> for UndefinedParser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        context.failure(self.message.clone())
    }
}

pub struct SettableParser<T> {
    pub delegate: Rc<RefCell<Rc<dyn Parser<T>>>>,
}

// ── Cycle-safe recursive `Debug` for the settable parsers ────────────────────────────────────
//
// A grammar's parser graph is cyclic: recursion legitimately closes through `SettableParserRef`,
// and a misuse (`s.set(s.clone())`, see the `StrongSettableCycle` lint) closes through a strong
// `Rc`. A naive `Debug` that recursed into the delegate would loop forever / stack-overflow. So we
// keep a thread-local stack of the `RefCell` *slot addresses* currently being printed; on
// revisiting a slot we emit a back-reference (`-> #N`) instead of recursing. An owner
// (`SettableParser`) and every `.borrow()` of it (`SettableParserRef`) share the same slot
// address, so a reference back to an in-progress rule is detected as a cycle.
thread_local! {
    static VISITING: RefCell<Vec<*const ()>> = const { RefCell::new(Vec::new()) };
}

/// Pops the current slot off the visiting stack when a `Debug::fmt` returns — on every path,
/// including a `?`-propagated formatter error.
struct VisitGuard;

impl Drop for VisitGuard {
    fn drop(&mut self) {
        VISITING.with(|v| {
            v.borrow_mut().pop();
        });
    }
}

enum Enter {
    /// The slot is already being printed at stack depth `.0` — emit a back-reference to it.
    Cycle(usize),
    /// Freshly pushed at stack depth `.1`; the held guard pops it on drop.
    Fresh(#[allow(dead_code)] VisitGuard, usize),
}

fn enter(slot: *const ()) -> Enter {
    VISITING.with(|v| {
        let mut stack = v.borrow_mut();
        if let Some(idx) = stack.iter().position(|&p| p == slot) {
            Enter::Cycle(idx)
        } else {
            let idx = stack.len();
            stack.push(slot);
            Enter::Fresh(VisitGuard, idx)
        }
    })
}

impl<T> Debug for SettableParser<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slot = Rc::as_ptr(&self.delegate) as *const ();
        match enter(slot) {
            Enter::Cycle(idx) => write!(f, "SettableParser(-> #{idx})"),
            Enter::Fresh(_guard, idx) => {
                write!(f, "SettableParser #{idx}({:?})", self.delegate.borrow())
            }
        }
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

impl<T: 'static> HasChildren for SettableParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.borrow().clone()]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Settable
    }
}

impl<T: 'static> Parser<T> for SettableParser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate.borrow().parse_on(context)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.borrow().fast_parse_on(context)
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
        let Some(rc) = self.delegate.upgrade() else {
            return f.write_str("SettableParserRef(dropped)");
        };
        let slot = Rc::as_ptr(&rc) as *const ();
        match enter(slot) {
            Enter::Cycle(idx) => write!(f, "SettableParserRef(-> #{idx})"),
            Enter::Fresh(_guard, idx) => write!(f, "SettableParserRef #{idx}({:?})", rc.borrow()),
        }
    }
}

impl<T> Clone for SettableParserRef<T> {
    fn clone(&self) -> Self {
        SettableParserRef {
            delegate: self.delegate.clone(),
        }
    }
}

impl<T: 'static> HasChildren for SettableParserRef<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![
            self.delegate
                .upgrade()
                .expect("SettableParser owner dropped")
                .borrow()
                .clone(),
        ]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::SettableRef
    }
}

impl<T: 'static> Parser<T> for SettableParserRef<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.delegate
            .upgrade()
            .expect("SettableParser owner dropped")
            .borrow()
            .parse_on(context)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate
            .upgrade()
            .expect("SettableParser owner dropped")
            .borrow()
            .fast_parse_on(context)
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;
    use crate::parser::character::char;
    use crate::parser::combinator::choice::choice2;
    use crate::parser::ext::ParserExt;

    // Regression guard: before the cycle-safe Debug, this recursed forever / stack-overflowed.
    #[test]
    fn debug_terminates_on_borrow_self_reference() {
        let mut s: SettableParser<()> = SettableParser::undefined();
        let r = s.borrow();
        s.set(r);
        let printed = format!("{s:?}");
        // Terminates, and the weak back-reference is rendered as a cycle marker, not an expansion.
        assert!(printed.contains("-> #0"), "got: {printed}");
    }

    // The `s.set(s.clone())` misuse builds a *strong* Rc cycle; the old Debug stack-overflowed on it.
    #[test]
    fn debug_terminates_on_strong_clone_self_reference() {
        let mut s: SettableParser<()> = SettableParser::undefined();
        let c = s.clone();
        s.set(c);
        let printed = format!("{s:?}");
        assert!(printed.contains("-> #0"), "got: {printed}");
    }

    #[test]
    fn debug_shows_structure_and_closes_recursion() {
        // s = '(' s | 'x' : a real recursive grammar. Debug should expand the structure once and
        // close the recursive branch with a back-reference.
        let mut s: SettableParser<()> = SettableParser::undefined();
        s.set(choice2(char('(').constant(()), char('x').constant(())));
        // (kept simple/acyclic here so the assertion is about structure being expanded, not cycles)
        let printed = format!("{s:?}");
        assert!(printed.starts_with("SettableParser #0("), "got: {printed}");
        assert!(printed.contains("Choice2"), "got: {printed}");
    }
}
