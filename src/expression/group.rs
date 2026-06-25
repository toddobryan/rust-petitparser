use std::fmt::{Debug, Result};
use std::rc::Rc;

use crate::core::parser::Parser;
use crate::parser::combinator::choice::{Choice2, SELECT_SECOND, choice2};
use crate::parser::combinator::sequence::{seq2, seq3};
use crate::parser::ext::{MapTuple2, ParserExt};

#[derive(Clone, Debug)]
pub struct ExpressionGroup<T> {
    pub loopback: Rc<dyn Parser<T>>,
    pub prefix: Vec<Rc<dyn Parser<UnaryOp<T>>>>,
    pub wrapper: Vec<Rc<dyn Parser<T>>>,
    pub postfix: Vec<Rc<dyn Parser<UnaryOp<T>>>>,
    pub right: Vec<Rc<dyn Parser<BinaryOp<T>>>>,
    pub left: Vec<Rc<dyn Parser<BinaryOp<T>>>>,
    pub optional_value: Option<T>,
}

impl<T: Debug + 'static> ExpressionGroup<T> {
    pub fn new(loopback: impl Parser<T> + 'static) -> Self {
        Self {
            loopback: Rc::new(loopback),
            prefix: Vec::new(),
            wrapper: Vec::new(),
            postfix: Vec::new(),
            right: Vec::new(),
            left: Vec::new(),
            optional_value: None,
        }
    }

    pub fn wrapper<L: Debug + 'static, R: Debug + 'static>(
        &mut self,
        left: impl Parser<L> + 'static,
        right: impl Parser<R> + 'static,
        callback: fn(L, T, R) -> T,
    ) {
        let p = seq3(left, self.loopback.clone(), right).map(move |(l, t, r)| callback(l, t, r));
        self.wrapper.push(Rc::new(p));
    }

    pub fn prefix<O: Debug + 'static>(&mut self, op: impl Parser<O> + 'static, f: fn(&O, &T) -> T) {
        let p = op.map(move |operator| UnaryOp(Box::new(move |value: &T| f(&operator, value))));
        self.prefix.push(Rc::new(p));
    }

    pub fn postfix<O: Debug + 'static>(
        &mut self,
        op: impl Parser<O> + 'static,
        f: fn(&T, &O) -> T,
    ) {
        let p = op.map(move |operator| UnaryOp(Box::new(move |value: &T| f(value, &operator))));
        self.postfix.push(Rc::new(p));
    }

    pub fn right<O: Debug + 'static>(
        &mut self,
        op: impl Parser<O> + 'static,
        f: fn(&T, &O, &T) -> T,
    ) {
        let p = op.map(move |operator| BinaryOp(Box::new(move |l, r| f(l, &operator, r))));
        self.right.push(Rc::new(p));
    }

    pub fn left<O: Debug + 'static>(
        &mut self,
        op: impl Parser<O> + 'static,
        f: fn(&T, &O, &T) -> T,
    ) {
        let p = op.map(move |operator| BinaryOp(Box::new(move |l, r| f(l, &operator, r))));
        self.left.push(Rc::new(p));
    }

    pub fn optional(&mut self, value: T)
    where
        T: Clone + 'static,
    {
        assert!(
            self.optional_value.is_none(),
            "At most one optional value expected"
        );
        self.optional_value = Some(value);
    }

    pub(crate) fn build(self, inner: Rc<dyn Parser<T>>) -> Rc<dyn Parser<T>>
    where
        T: Clone + 'static,
    {
        let inner = build_wrapper(self.wrapper, inner);
        let inner = build_prefix(self.prefix, inner);
        let inner = build_postfix(self.postfix, inner);
        let inner = build_right(self.right, inner);
        let inner = build_left(self.left, inner);
        build_optional(self.optional_value, inner)
    }
}

type UnaryBox<T> = Box<dyn FnOnce(&T) -> T>;

pub struct UnaryOp<T>(UnaryBox<T>);

impl<T> Debug for UnaryOp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        f.write_str("<unary operation>")
    }
}

impl<T> UnaryOp<T> {
    fn call(self, value: &T) -> T {
        self.0(value)
    }
}

type BinaryBox<T> = Box<dyn FnOnce(&T, &T) -> T>;

pub struct BinaryOp<T>(BinaryBox<T>);

impl<T> Debug for BinaryOp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        f.write_str("<binary operation>")
    }
}

impl<T> BinaryOp<T> {
    fn call(self, left: &T, right: &T) -> T {
        self.0(left, right)
    }
}

/// Builds a choice with the same failure-reporting semantics as dart's `buildChoice`: dart's
/// `ChoiceParser` defaults to `selectLast` (always the failure of whichever alternative was
/// tried last, regardless of how far any other alternative got), whereas our own `choice2`
/// defaults to `SELECT_FARTHEST`. `ExpressionBuilder`/`ExpressionGroup` need to match dart's
/// behavior specifically (e.g. a failed `wrapper()` that recurses deep before failing must
/// NOT shadow the immediate, shallower failure of the plain primitive listed after it) — so
/// every pairwise fold here overrides the joiner to `SELECT_SECOND`, which is exactly
/// `selectLast` one step at a time.
fn select_last<T: Debug + 'static>(
    a: Rc<dyn Parser<T>>,
    b: Rc<dyn Parser<T>>,
) -> Rc<dyn Parser<T>> {
    Rc::new(Choice2 {
        joiner: SELECT_SECOND,
        ..choice2(a, b)
    })
}

pub(crate) fn choice_of<T: Debug + 'static>(parsers: Vec<Rc<dyn Parser<T>>>) -> Rc<dyn Parser<T>> {
    fn helper<U: Debug + 'static>(
        acc: Rc<dyn Parser<U>>,
        rest: &mut impl Iterator<Item = Rc<dyn Parser<U>>>,
    ) -> Rc<dyn Parser<U>> {
        match rest.next() {
            None => acc,
            Some(p) => helper(select_last(acc, p), rest),
        }
    }

    assert!(
        !parsers.is_empty(),
        "We should have at least one parser here"
    );
    let mut ps = parsers.into_iter();
    let first = ps.next();
    let second = ps.next();
    match (first, second) {
        (Some(p1), Some(p2)) => helper(select_last(p1, p2), &mut ps),
        (Some(p1), None) => p1,
        (None, _) => panic!("This should be impossible because of the assert! above"),
    }
}

fn build_wrapper<T: Debug + 'static>(
    wrapper: Vec<Rc<dyn Parser<T>>>,
    inner: Rc<dyn Parser<T>>,
) -> Rc<dyn Parser<T>> {
    let mut parsers = wrapper.clone();
    parsers.push(inner);
    choice_of(parsers)
}

fn build_prefix<T: Debug + 'static>(
    prefix: Vec<Rc<dyn Parser<UnaryOp<T>>>>,
    inner: Rc<dyn Parser<T>>,
) -> Rc<dyn Parser<T>> {
    if prefix.is_empty() {
        inner
    } else {
        Rc::new(
            seq2(choice_of(prefix.clone()).star(), inner).map2(|prefix, value| {
                prefix
                    .into_iter()
                    .rev()
                    .fold(value, |acc, op| op.call(&acc))
            }),
        )
    }
}

fn build_postfix<T: Debug + 'static>(
    postfix: Vec<Rc<dyn Parser<UnaryOp<T>>>>,
    inner: Rc<dyn Parser<T>>,
) -> Rc<dyn Parser<T>> {
    if postfix.is_empty() {
        inner
    } else {
        Rc::new(
            seq2(inner, choice_of(postfix.clone()).star())
                .map2(|value, postfix| postfix.into_iter().fold(value, |acc, op| op.call(&acc))),
        )
    }
}

fn build_right<T: Debug + 'static>(
    right: Vec<Rc<dyn Parser<BinaryOp<T>>>>,
    inner: Rc<dyn Parser<T>>,
) -> Rc<dyn Parser<T>> {
    if right.is_empty() {
        inner
    } else {
        Rc::new(
            seq2(
                inner.clone(),
                seq2(choice_of(right.clone()), inner.clone()).star(),
            )
            .map2(|first, rest| {
                // `rest` is `[(op1, t1), (op2, t2), ...]`; right-fold means
                // `first op1 (t1 op2 (t2 ...))`, so walk the terms/ops from the end.
                let mut terms = vec![first];
                let mut ops = Vec::with_capacity(rest.len());
                for (op, term) in rest {
                    ops.push(op);
                    terms.push(term);
                }
                let mut acc = terms.pop().unwrap();
                while let (Some(op), Some(term)) = (ops.pop(), terms.pop()) {
                    acc = op.call(&term, &acc);
                }
                acc
            }),
        )
    }
}

fn build_left<T: Debug + 'static>(
    left: Vec<Rc<dyn Parser<BinaryOp<T>>>>,
    inner: Rc<dyn Parser<T>>,
) -> Rc<dyn Parser<T>> {
    if left.is_empty() {
        inner
    } else {
        Rc::new(
            seq2(
                inner.clone(),
                seq2(choice_of(left.clone()), inner.clone()).star(),
            )
            .map2(|first, rest| {
                rest.into_iter()
                    .fold(first, |acc, (op, term)| op.call(&acc, &term))
            }),
        )
    }
}

fn build_optional<T>(optional_value: Option<T>, inner: Rc<dyn Parser<T>>) -> Rc<dyn Parser<T>>
where
    T: Clone + Debug + 'static,
{
    match optional_value {
        None => inner,
        Some(value) => Rc::new(inner.opt_with(value)),
    }
}
