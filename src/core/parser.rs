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

    fn is_directly_nullable(&self) -> bool {
        false
    }

    fn is_sequence(&self) -> bool {
        false
    }

    fn is_choice(&self) -> bool {
        false
    }

    fn is_repeating(&self) -> bool {
        false
    } // possessive/lazy/greedy only

    fn is_separated_repeating(&self) -> bool {
        false
    }

    fn is_possessive_repeating(&self) -> bool {
        false
    }

    fn repeating_min(&self) -> Option<usize> {
        None
    }

    fn is_settable(&self) -> bool {
        false
    }

    fn is_undefined(&self) -> bool {
        false
    } // UndefinedParser sentinel

    fn is_char(&self) -> bool {
        false
    } // CharParser or PredicateCharParser

    fn is_string_predicate(&self) -> bool {
        false
    } // PredicateParser (string)

    fn is_input(&self) -> bool {
        false
    } // InputParser

    fn input_message(&self) -> Option<&str> {
        None
    } // InputParser's message field

    fn is_newline(&self) -> bool {
        false
    }

    fn is_char_repeating(&self) -> bool {
        false
    } // CharacterRepeatingParser

    fn is_map_parser(&self) -> bool {
        false
    }

    fn is_constant_parser(&self) -> bool {
        false
    }

    fn is_token_parser(&self) -> bool {
        false
    }

    fn is_only_if(&self) -> bool {
        false
    }

    fn is_elements_at_parser(&self) -> bool {
        false
    }

    fn is_pick_parser(&self) -> bool {
        false
    }

    fn is_result_producing(&self) -> bool {
        self.is_constant_parser()
            || self.is_input()
            || self.is_map_parser()
            || self.is_elements_at_parser()
            || self.is_pick_parser()
            || self.is_token_parser()
            || self.is_only_if()
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

    fn is_directly_nullable(&self) -> bool {
        (**self).is_directly_nullable()
    }

    fn is_sequence(&self) -> bool {
        (**self).is_sequence()
    }

    fn is_choice(&self) -> bool {
        (**self).is_choice()
    }

    fn is_repeating(&self) -> bool {
        (**self).is_repeating()
    }

    fn is_separated_repeating(&self) -> bool {
        (**self).is_separated_repeating()
    }

    fn is_possessive_repeating(&self) -> bool {
        (**self).is_possessive_repeating()
    }

    fn repeating_min(&self) -> Option<usize> {
        (**self).repeating_min()
    }

    fn is_settable(&self) -> bool {
        (**self).is_settable()
    }

    fn is_undefined(&self) -> bool {
        (**self).is_undefined()
    }

    fn is_char(&self) -> bool {
        (**self).is_char()
    }

    fn is_string_predicate(&self) -> bool {
        (**self).is_string_predicate()
    }

    fn is_input(&self) -> bool {
        (**self).is_input()
    }

    fn input_message(&self) -> Option<&str> {
        (**self).input_message()
    }

    fn is_newline(&self) -> bool {
        (**self).is_newline()
    }

    fn is_char_repeating(&self) -> bool {
        (**self).is_char_repeating()
    }

    fn is_map_parser(&self) -> bool {
        (**self).is_map_parser()
    }

    fn is_constant_parser(&self) -> bool {
        (**self).is_constant_parser()
    }

    fn is_token_parser(&self) -> bool {
        (**self).is_token_parser()
    }

    fn is_only_if(&self) -> bool {
        (**self).is_only_if()
    }
}
