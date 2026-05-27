use googletest::prelude::*;
use rust_petitparser::context::{Context, ParseResult, Success};
use rust_petitparser::core::{Parser, Shared};
use rust_petitparser::parser::character::parsers::char;

#[gtest]
fn character_test() {
    let ch = char('a', None);
    let result = ch.parse("abc");
    let success = match result {
        ParseResult::Success(s) => Some(s),
        _ => None,
    }
    .unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(**success.value, eq('a'));
}

#[gtest]
fn char_parser_equality() {
    let a = char('a', None);
    let b = char('b', None);
    let a2 = char('a', None);

    assert!(a == a2);
    assert!(a != b);
}
