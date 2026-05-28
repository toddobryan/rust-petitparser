use googletest::prelude::*;
use rust_petitparser::character::char;
use rust_petitparser::combinator::{choice2, choice2_with_joiner, seq2, seq4};
use rust_petitparser::core::Parser;
use rust_petitparser::failure_joiner::SELECT_FARTHEST_JOINED;

#[gtest]
fn character_test() {
    let ch = char('a', None);
    let success = ch.parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn char_parser_equality() {
    let a = char('a', None);
    let b = char('b', None);
    let a2 = char('a', None);

    assert_that!(a, eq(&a2));
    assert_that!(a, not(eq(&b)));
}

#[gtest]
fn seq2_test() {
    let p = seq2(char('a', None), char('b', None));
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(2));
    assert_that!(success.value.0, eq('a'));
    assert_that!(success.value.1, eq('b'));
}

#[gtest]
fn seq2_first_fails() {
    let p = seq2(char('a', None), char('b', None));
    let failure = p.parse("xb").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

#[gtest]
fn seq2_second_fails() {
    let p = seq2(char('a', None), char('b', None));
    let failure = p.parse("ax").unwrap_err();
    assert_that!(failure.context.position, eq(1));
}

#[gtest]
fn seq4_test() {
    let p = seq4(char('a', None), char('b', None), char('c', None), char('d', None));
    let success = p.parse("abcd").unwrap();
    assert_that!(success.context.position, eq(4));
    assert_that!(success.value.0, eq('a'));
    assert_that!(success.value.1, eq('b'));
    assert_that!(success.value.2, eq('c'));
    assert_that!(success.value.3, eq('d'));
}

#[gtest]
fn choice2_test() {
    let p = choice2(char('a', None), char('b', None));
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
    let p = choice2_with_joiner(char('a', None), char('b', None), SELECT_FARTHEST_JOINED);
    let success = p.parse("ac").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
    let success = p.parse("bc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('b'));
    let failure = p.parse("cc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'a', but found 'c' OR expected 'b', but found 'c'"));
}

#[gtest]
fn choice2_failure_test() {
    let p = choice2(seq2(char('a', None), char('b', None)), seq2(char('c', None), char('x', None)));
    let failure = p.parse("ax").unwrap_err();
    assert_that!(failure.context.position, eq(1));
    assert_that!(failure.message, eq("expected 'b', but found 'x'"));
}