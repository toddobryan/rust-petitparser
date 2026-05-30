use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::Failure;
use crate::matcher::matches::MatchesIterable;
use crate::parser::action::map::MapParser;
use crate::parser::action::token::TokenParser;
use crate::parser::character::character::whitespace;
use crate::parser::combinator::lookahead::{AndParser, NotParser};
use crate::parser::combinator::sequence::seq3;
use crate::parser::repeater::possessive::PossessiveRepeatingParser;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

pub trait ParserExt<T>: Parser<T> + Sized
where
    T: Debug,
{
    fn all_matches(
        self,
        buffer: Rc<[char]>,
        start: usize,
        overlapping: bool,
    ) -> MatchesIterable<T, Self> {
        MatchesIterable {
            parser: self,
            context: Context {
                buffer,
                position: start,
            },
            overlapping,
            parser_type: PhantomData,
        }
    }

    fn map<U, F: Fn(T) -> U>(self, f: F) -> MapParser<T, Self, F> {
        MapParser {
            delegate: self,
            f,
            from_type: PhantomData,
        }
    }

    fn rep(self, min: usize, max: Option<usize>) -> impl Parser<Vec<T>> {
        PossessiveRepeatingParser {
            delegate: self,
            min,
            max,
        }
    }

    fn star(self) -> impl Parser<Vec<T>> {
        self.rep(0, None)
    }

    fn plus(self) -> impl Parser<Vec<T>> {
        self.rep(1, None)
    }

    fn opt(self) -> impl Parser<Option<T>> {
        self.rep(0, Some(1)).map(|vec| vec.into_iter().next())
    }

    fn token(self) -> TokenParser<Self> {
        TokenParser { parser: self }
    }

    fn trim(self) -> impl Parser<T> {
        seq3(whitespace().star(), self, whitespace().star()).map(|(_, val, _)| val)
    }

    fn and(self) -> impl Parser<T> {
        AndParser {
            delegate: self,
            delegate_type: PhantomData,
        }
    }

    fn not(self) -> impl Parser<Failure> {
        NotParser {
            delegate: self,
            delegate_type: PhantomData,
        }
    }
}

impl<T, P: Parser<T>> ParserExt<T> for P where T: Debug {}
