use std::marker::PhantomData;
use std::rc::Rc;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::matcher::matches::MatchesIterable;
use crate::parser::action::map::MapParser;
use crate::parser::action::token::TokenParser;
use crate::parser::repeater::possessive::{OptParser, PlusParser, StarParser};

pub trait ParserExt<T>: Parser<T> + Sized {
    fn all_matches(self, buffer: Rc<[char]>, start: usize, overlapping: bool) -> MatchesIterable<T, Self> {
        MatchesIterable { parser: self, context: Context { buffer, position: start }, overlapping, parser_type: PhantomData }
    }

    fn map<U, F: Fn(T) -> U>(self, f: F) -> MapParser<T, Self, F> {
        MapParser {
            parser: self,
            f,
            orig_type: PhantomData,
        }
    }

    fn star(self) -> StarParser<Self> {
        StarParser { parser: self }
    }

    fn plus(self) -> PlusParser<Self> {
        PlusParser { parser: self }
    }

    fn opt(self) -> OptParser<Self> {
        OptParser { parser: self }
    }

    fn token(self) -> TokenParser<Self> {
        TokenParser { parser: self }
    }
}

impl <T, P: Parser<T>> ParserExt<T> for P {}
