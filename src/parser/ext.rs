use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::{Failure, ParseResult, Success};
use crate::matcher::matches::MatchesIterable;
use crate::parser::action::constant::ConstantParser;
use crate::parser::action::continuation::{Continuation, ContinuationParser};
use crate::parser::action::elements_at::ElementsAtParser;
use crate::parser::action::flat_map::FlatMapParser;
use crate::parser::action::input::InputParser;
use crate::parser::action::map::MapParser;
use crate::parser::action::only_if::OnlyIfParser;
use crate::parser::action::pick::PickParser;
use crate::parser::action::token::TokenParser;
use crate::parser::character::whitespace;
use crate::parser::combinator::lookahead::{AndParser, NotParser};
use crate::parser::combinator::sequence::seq3;
use crate::parser::combinator::skip::SkipParser;
use crate::parser::misc::end::eof;
use crate::parser::misc::epsilon::epsilon;
use crate::parser::misc::label::LabeledParser;
use crate::parser::repeater::greedy::GreedyRepeatingParser;
use crate::parser::repeater::lazy::LazyRepeatingParser;
use crate::parser::repeater::possessive::PossessiveRepeatingParser;
use crate::parser::repeater::separated::SeparatedRepeatingParser;
use crate::prelude::any;
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

    fn accept_at(&self, input: &str, start: usize) -> bool {
        let buffer: Rc<[char]> = input.chars().collect::<Vec<_>>().into();
        self.fast_parse_on(buffer, start).is_some()
    }

    fn accept(&self, input: &str) -> bool {
        self.accept_at(input, 0_usize)
    }

    fn map<U, F: Fn(T) -> U>(self, f: F) -> MapParser<T, Self, F> {
        MapParser {
            delegate: self,
            f,
            from_type: PhantomData,
        }
    }

    fn constant<V>(self, value: V) -> ConstantParser<T, Self, V> {
        ConstantParser {
            delegate: self,
            value,
            delegate_type: PhantomData,
        }
    }

    fn rep(self, min: usize, max: Option<usize>) -> PossessiveRepeatingParser<Self> {
        PossessiveRepeatingParser {
            delegate: self,
            min,
            max,
        }
    }

    fn rep_lazy<S: Debug>(
        self,
        limit: impl Parser<S>,
        min: usize,
        max: Option<usize>,
    ) -> impl Parser<Vec<T>> {
        LazyRepeatingParser {
            delegate: self,
            limit,
            min,
            max,
            delegate_type: PhantomData,
            limit_type: PhantomData,
        }
    }

    fn rep_greedy(
        self,
        limit: impl Parser<()>,
        min: usize,
        max: Option<usize>,
    ) -> impl Parser<Vec<T>> {
        GreedyRepeatingParser {
            delegate: self,
            limit,
            min,
            max,
            delegate_type: PhantomData,
        }
    }

    fn times(self, num: usize) -> PossessiveRepeatingParser<Self> {
        self.rep(num, Some(num))
    }

    fn rep_sep<S: Debug>(
        self,
        sep: impl Parser<S>,
        min: usize,
        max: Option<usize>,
    ) -> impl Parser<Vec<T>> {
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

    fn star_lazy<L: Debug>(self, limit: impl Parser<L>) -> impl Parser<Vec<T>> {
        LazyRepeatingParser {
            delegate: self,
            limit,
            min: 0,
            max: None,
            delegate_type: PhantomData,
            limit_type: PhantomData,
        }
    }

    fn star_greedy(self, limit: impl Parser<()>) -> impl Parser<Vec<T>> {
        GreedyRepeatingParser {
            delegate: self,
            limit,
            min: 0,
            max: None,
            delegate_type: PhantomData,
        }
    }

    fn plus(self) -> impl Parser<Vec<T>> {
        self.rep(1, None)
    }

    fn plus_sep<S: Debug>(self, sep: impl Parser<S>) -> impl Parser<Vec<T>> {
        self.rep_sep(sep, 1, None)
    }

    fn plus_lazy<S: Debug>(self, limit: impl Parser<S>) -> impl Parser<Vec<T>> {
        LazyRepeatingParser {
            delegate: self,
            limit,
            min: 1,
            max: None,
            delegate_type: PhantomData,
            limit_type: PhantomData,
        }
    }

    fn plus_greedy(self, limit: impl Parser<()>) -> impl Parser<Vec<T>> {
        GreedyRepeatingParser {
            delegate: self,
            limit,
            min: 1,
            max: None,
            delegate_type: PhantomData,
        }
    }

    fn opt(self) -> impl Parser<Option<T>> {
        self.rep(0, Some(1)).map(|vec| vec.into_iter().next())
    }

    fn opt_with(self, value: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        self.rep(0, Some(1))
            .map(move |vec| vec.into_iter().next().unwrap_or(value.clone()))
    }

    fn token(self) -> TokenParser<Self> {
        TokenParser { parser: self }
    }

    fn trim(self) -> impl Parser<T> {
        self.trim_with(whitespace().star(), whitespace().star())
    }

    fn trim_with<A, Aft, B, Bef>(self, before: B, after: A) -> impl Parser<T>
    where
        A: Parser<Aft>,
        B: Parser<Bef>,
        Aft: Debug,
        Bef: Debug,
    {
        seq3(before, self, after).map(|(_, val, _)| val)
    }

    fn and(self) -> impl Parser<T> {
        AndParser {
            delegate: self,
            delegate_type: PhantomData,
        }
    }

    fn not(self) -> impl Parser<Failure> {
        self.not_with_message("success not expected".to_string())
    }

    fn not_with_message(self, message: String) -> impl Parser<Failure> {
        NotParser {
            delegate: self,
            delegate_type: PhantomData,
            message,
        }
    }

    fn neg(self) -> impl Parser<char> {
        self.neg_with_message("input not expected".to_string())
    }

    fn neg_with_message(self, message: String) -> impl Parser<char> {
        any().skip_left(self.not_with_message(message))
    }

    fn call_cc<T2, H>(self, handler: H) -> ContinuationParser<T, T2, H>
    where
        Self: Sized + 'static,
        T: 'static,
        H: Fn(Continuation<T>, &Context) -> ParseResult<T2>,
    {
        ContinuationParser {
            delegate: Rc::new(self),
            handler,
            result_type: PhantomData,
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

    fn only_if_with_factory(
        self,
        pred: fn(&T) -> bool,
        factory: fn(&Context, Success<T>) -> ParseResult<T>,
    ) -> OnlyIfParser<Self, T> {
        OnlyIfParser {
            delegate: self,
            predicate: pred,
            factory: Some(factory),
            message: None,
            delegate_type: PhantomData,
        }
    }

    fn labeled(self, label: impl Into<String>) -> LabeledParser<Self, T> {
        LabeledParser {
            delegate: self,
            label: label.into(),
            delegate_type: PhantomData,
        }
    }

    fn skip<A, Aft, B, Bef>(self, before: B, after: A) -> SkipParser<A, Aft, B, Bef, Self, T>
    where
        A: Parser<Aft>,
        B: Parser<Bef>,
    {
        SkipParser {
            delegate: self,
            before,
            after,
            delegate_type: PhantomData,
            before_type: PhantomData,
            after_type: PhantomData,
        }
    }

    fn skip_left<B, Bef>(self, before: B) -> SkipParser<impl Parser<()>, (), B, Bef, Self, T>
    where
        B: Parser<Bef>,
    {
        self.skip(before, epsilon())
    }

    fn skip_right<A, Aft>(self, after: A) -> SkipParser<A, Aft, impl Parser<()>, (), Self, T>
    where
        A: Parser<Aft>,
    {
        self.skip(epsilon(), after)
    }

    fn end(self) -> impl Parser<T> {
        self.skip_right(eof())
    }

    fn flat_map<P2, T2>(self, f: fn(&T) -> P2) -> FlatMapParser<Self, P2, T, T2> {
        FlatMapParser {
            delegate: self,
            f,
            delegate_type: PhantomData,
            result_type: PhantomData,
        }
    }

    fn pick<E>(self, index: i32) -> PickParser<Self, T, E>
    where
        T: IntoIterator<Item = E>,
        E: Clone + Debug,
    {
        PickParser {
            delegate: self,
            index,
            delegate_type: PhantomData,
            iterator_type: PhantomData,
        }
    }

    fn elements_at<E>(self, indexes: Vec<i32>) -> ElementsAtParser<Self, T, E>
    where
        T: IntoIterator<Item = E>,
        E: Clone + Debug,
    {
        ElementsAtParser {
            delegate: self,
            indexes,
            delegate_type: PhantomData,
            iterator_type: PhantomData,
        }
    }

    /// alias for elements_at to match Dart usage
    fn permute<E>(self, indexes: Vec<i32>) -> ElementsAtParser<Self, T, E>
    where
        T: IntoIterator<Item = E>,
        E: Clone + Debug,
    {
        self.elements_at(indexes)
    }
}

impl<T, P: Parser<T>> ParserExt<T> for P where T: Debug {}
