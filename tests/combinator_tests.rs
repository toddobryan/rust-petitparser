use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{char, letter};
use rust_petitparser::parser::combinator::choice::{Choice2, SELECT_FARTHEST_JOINED, choice2, choice3};
use rust_petitparser::parser::combinator::sequence::{seq2, seq3, seq4};
use rust_petitparser::parser::combinator::settable::SettableParser;
use rust_petitparser::parser::ext::ParserExt;

#[gtest]
fn seq2_test() {
    let p = seq2(char('a'), char('b'));
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(2));
    assert_that!(success.value.0, eq('a'));
    assert_that!(success.value.1, eq('b'));
}

#[gtest]
fn seq2_first_fails() {
    let p = seq2(char('a'), char('b'));
    let failure = p.parse("xb").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

#[gtest]
fn seq2_second_fails() {
    let p = seq2(char('a'), char('b'));
    let failure = p.parse("ax").unwrap_err();
    assert_that!(failure.context.position, eq(1));
}

#[gtest]
fn seq4_test() {
    let p = seq4(char('a'), char('b'), char('c'), char('d'));
    let success = p.parse("abcd").unwrap();
    assert_that!(success.context.position, eq(4));
    assert_that!(success.value.0, eq('a'));
    assert_that!(success.value.1, eq('b'));
    assert_that!(success.value.2, eq('c'));
    assert_that!(success.value.3, eq('d'));
}

#[gtest]
fn choice2_test() {
    let p = choice2(char('a'), char('b'));
    let success = p.parse("ac").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
    let success = p.parse("bc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('b'));
    let failure = p.parse("cc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'b', but found 'c'"));
}

#[gtest]
fn choice2_with_joiner_test() {
    let p = Choice2 {
        joiner: SELECT_FARTHEST_JOINED,
        ..choice2(char('a'), char('b'))
    };
    let success = p.parse("ac").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
    let success = p.parse("bc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('b'));
    let failure = p.parse("cc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected 'a', but found 'c' OR expected 'b', but found 'c'")
    );
}

#[gtest]
fn choice2_failure_test() {
    let p = choice2(seq2(char('a'), char('b')), seq2(char('c'), char('x')));
    let failure = p.parse("ax").unwrap_err();
    assert_that!(failure.context.position, eq(1));
    assert_that!(failure.message, eq("expected 'b', but found 'x'"));
}

#[gtest]
fn choice3_test() {
    let p = choice3(char('a'), char('b'), char('c'));
    let success = p.parse("ax").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
    let success = p.parse("bx").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('b'));
    let success = p.parse("cx").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('c'));
    let failure = p.parse("dx").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'c', but found 'd'"));
}

#[gtest]
fn nested_parens_settable_test() {
    let mut expr = SettableParser::<i32>::undefined();
    let inner = seq3(char('('), expr.clone(), char(')')).map(|(_, n, _)| n + 1);
    let leaf = char('x').map(|_| 0);
    expr.set(choice2(inner, leaf));

    assert_eq!(expr.parse("x").unwrap().value, 0);
    assert_eq!(expr.parse("(((x)))").unwrap().value, 3);
    assert!(expr.parse("(x").is_err());
}

#[gtest]
fn and_succeeds_without_advancing() {
    let p = char('a').and();
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn and_fails_when_inner_fails() {
    let p = char('z').and();
    assert!(p.parse("abc").is_err());
}

#[gtest]
fn and_as_lookahead_in_sequence() {
    let p = seq2(char('a'), char('b').and()).map(|(l, _)| l);
    let success = p.parse("ab").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn not_succeeds_when_inner_fails() {
    let p = seq2(char('b').not(), letter());
    let result = p.parse("a").unwrap();
    assert_that!(result.context.position, eq(1));
    assert_that!(result.value.1, eq('a'));

    let result = p.parse("b").unwrap_err();
    assert_that!(result.context.position, eq(0));
}

#[gtest]
fn not_fails_when_inner_succeeds() {
    let p = char('a').not();
    let failure = p.parse("abc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("Expected failure, got success: 'a'"))
}

#[gtest]
fn not_succeeds_without_advancing() {
    let p = seq2(char('z').not(), letter().star()).map(|(_, ls)| ls);
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&vec!['a', 'b', 'c']));

    let success = p.parse("yz").unwrap();
    assert_that!(success.context.position, eq(2));
    assert_that!(success.value, eq(&vec!['y', 'z']));

    let failure = p.parse("zyx").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("Expected failure, got success: 'z'"));
}
