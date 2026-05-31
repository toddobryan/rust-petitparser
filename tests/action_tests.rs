use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::core::token::Token;
use rust_petitparser::parser::character::character::{char, letter};
use rust_petitparser::parser::combinator::sequence::seq2;
use rust_petitparser::parser::ext::ParserExt;
use rust_petitparser::parser::predicate::predicate::string;

#[gtest]
fn map_test() {
    let p = char('a').map(|c| String::from(c));
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq("a"));
    let failure = p.parse("bc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'a', but found 'b'"));
}

#[gtest]
fn token_test() {
    let p = char('a').plus().token();
    let success = p.parse("aaab").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(
        success.value,
        eq(&Token::new(
            vec!['a', 'a', 'a'],
            success.context.buffer.clone(),
            0,
            3
        ))
    );
}

#[gtest]
fn token_line_and_column() {
    let p = seq2(
        seq2(char('a'), char('b')),
        seq2(char('\n'), char('c').plus().token()),
    )
    .map(|((_, _), (_, t))| t);
    let success = p.parse("ab\nccd").unwrap();
    assert_that!(success.value.line(), eq(2));
    assert_that!(success.value.column(), eq(1));
}

#[gtest]
fn trim_test() {
    let p = string("hello").trim();
    let success = p.parse("  hello  ").unwrap();
    assert_that!(success.context.position, eq(9));
    assert_that!(success.value, eq("hello"));
}

#[gtest]
fn input_test() {
    let p = letter().plus().input();
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq("abc"));

    let failure = p.parse("123").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected letter, but found '1'"));
}

#[gtest]
fn input_with_message_test() {
    let p = letter().plus().input_with_message("expected a string of letters".to_string());
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq("abc"));

    let failure = p.parse("123").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected a string of letters"));
}
