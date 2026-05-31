use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{Failure, ParseResult, Success};
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
use crate::parser::action::input::InputParser;
use crate::parser::action::only_if::OnlyIfParser;
use crate::parser::repeater::separated::SeparatedRepeatingParser;

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

    fn rep(self, min: usize, max: Option<usize>) -> PossessiveRepeatingParser<Self> {
        PossessiveRepeatingParser {
            delegate: self,
            min,
            max,
        }
    }

    fn times(self, num: usize) -> PossessiveRepeatingParser<Self>  {
        self.rep(num, Some(num))
    }

    fn rep_sep<S: Debug>(self, sep: impl Parser<S>, min: usize, max: Option<usize>) -> impl Parser<Vec<T>> {
        SeparatedRepeatingParser {
            delegate: self,
            separator: sep,
            min,
            max,
            delegate_type: PhantomData,
            separator_type: PhantomData,
        }
    }

    fn star(self) -> impl Parser<Vec<T>> {
        self.rep(0, None)
    }

    fn star_sep<S: Debug>(self, sep: impl Parser<S>) -> impl Parser<Vec<T>> {
        self.rep_sep(sep, 0, None)
    }

    fn plus(self) -> impl Parser<Vec<T>> {
        self.rep(1, None)
    }

    fn plus_sep<S: Debug>(self, sep: impl Parser<S>) -> impl Parser<Vec<T>> {
        self.rep_sep(sep, 1, None)
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

    fn input(self) -> impl Parser<String> {
        InputParser {
            delegate: self,
            message: None,
            delegate_type: PhantomData,
        }
    }

    fn input_with_message(self, message: String) -> impl Parser<String> {
        InputParser {
            delegate: self,
            message: Some(message),
            delegate_type: PhantomData,
        }
    }

    fn only_if(self, pred: fn(&T) -> bool) -> OnlyIfParser<Self, T> {
        OnlyIfParser {
            delegate: self,
            predicate: pred,
            factory: None,
            message: None,
            delegate_type: PhantomData,
        }
    }

    fn only_if_with_message(self, pred: fn(&T) -> bool, message: String) -> OnlyIfParser<Self, T> {
        OnlyIfParser {
            delegate: self,
            predicate: pred,
            factory: None,
            message: Some(message),
            delegate_type: PhantomData,
        }
    }

    fn only_if_with_factory(self, pred: fn(&T) -> bool, factory: fn(&Context, Success<T>) -> ParseResult<T>) -> OnlyIfParser<Self, T> {
        OnlyIfParser {
            delegate: self,
            predicate: pred,
            factory: Some(factory),
            message: None,
            delegate_type: PhantomData,
        }
    }
}

impl<T, P: Parser<T>> ParserExt<T> for P where T: Debug {}
