use googletest::prelude::*;
use rust_petitparser::character::char;
use rust_petitparser::combinator::{seq2, seq4};
use rust_petitparser::core::Parser;

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
