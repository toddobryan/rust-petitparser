use crate::core::context::Context;
use crate::core::kind::ParserKind;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

pub trait Parser<T: 'static>: Debug + HasChildren + 'static {
    fn parse_on(&self, context: &Context) -> ParseResult<T>;

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.parse_on(context).ok().map(|s| s.context.position)
    }

    fn parse(&self, input: &str) -> ParseResult<T> {
        let buffer: Rc<[char]> = input.chars().collect::<Vec<_>>().into();
        self.parse_on(&Context {
            text: Rc::new(String::from(input)),
            buffer,
            position: 0,
        })
    }
}

/// Value-type-erased view of a parser's immediate sub-parsers, used for reflection / the
/// linter. It is a *required* supertrait of `Parser<T>` (every parser must report its
/// children — no opt-out), and it is deliberately *not* generic over the value type: a
/// parser's children may each produce a different `T`, so a tree-walker needs a single
/// non-generic handle to recurse through. `Rc<dyn Parser<T>>` upcasts to
/// `Rc<dyn HasChildren>` (trait upcasting), which is what makes combinator `children()`
/// impls one-liners under the `Rc`-storage design.
pub trait HasChildren: Debug {
    fn kind(&self) -> ParserKind<'_>;

    fn children(&self) -> Vec<Rc<dyn HasChildren>>;

    /// A parser that produces a value which a surrounding flatten (`.input()`) would discard.
    /// Derived from [`kind`](HasChildren::kind) — the one remaining semantic helper now that the
    /// raw type-flags are gone; all classification goes through `kind()` (a `matches!` on the
    /// returned [`ParserKind`]).
    fn is_result_producing(&self) -> bool {
        matches!(
            self.kind(),
            ParserKind::Constant(_)
                | ParserKind::Input { .. }
                | ParserKind::Map(_)
                | ParserKind::ElementsAt { .. }
                | ParserKind::Pick { .. }
                | ParserKind::Token
                | ParserKind::OnlyIf { .. }
        )
    }
}

impl<T: 'static, P: Parser<T> + ?Sized> Parser<T> for Rc<P> {
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        (**self).parse_on(context)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        (**self).fast_parse_on(context)
    }
}

impl<P: HasChildren + ?Sized> HasChildren for Rc<P> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        (**self).children()
    }

    fn kind(&self) -> ParserKind<'_> {
        (**self).kind()
    }
}
