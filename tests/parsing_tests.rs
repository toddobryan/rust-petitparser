use rust_petitparser::parser::ext::ParserExt;
use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::core::token::Token;
use rust_petitparser::parser::character::character::char;
use rust_petitparser::parser::combinator::choice::{choice2, choice3, Choice2, SELECT_FARTHEST_JOINED};
use rust_petitparser::parser::combinator::sequence::{seq2, seq4};

#[gtest]
fn character_test() {
    let ch = char('a');
    let success = ch.parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn char_parser_equality() {
    let a = char('a');
    let b = char('b');
    let a2 = char('a');

    assert_that!(a, eq(&a2));
    assert_that!(a, not(eq(&b)));
}

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
    let p = Choice2 { joiner: SELECT_FARTHEST_JOINED, ..choice2(char('a'), char('b')) };
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
fn opt_test() {
    let p = char('a').opt();
    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq(Some('a')));
    let success = p.parse("bc").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(None));
}

#[gtest]
fn plus_test() {
    let p = char('a').plus();

    let success = p.parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq(&vec!['a']));

    let success = p.parse("aaabc").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&vec!['a', 'a', 'a']));

    let failure = p.parse("bc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'a', but found 'b'"));
}

#[gtest]
fn plus_with_epsilon_test() {
    let p = char('x').opt().plus();
    let success = p.parse("xxxy").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&vec![Some('x'), Some('x'), Some('x')]));

    let success = p.parse("y").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(&vec![None]));
}

#[gtest]
fn star_test() {
    let p = char('x').star();

    let success = p.parse("a").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(&vec![]));

    let success = p.parse("xxa").unwrap();
    assert_that!(success.context.position, eq(2));
    assert_that!(success.value, eq(&vec!['x', 'x']));
}

#[gtest]
fn star_with_epsilon_test() {
    let p = char('x').opt().star();
    let success = p.parse("xxxy").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&vec![Some('x'), Some('x'), Some('x')]));

    let success = p.parse("y").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(&vec![]));
}

#[gtest]
fn token_test() {
    let p = char('a').plus().token();
    let success = p.parse("aaab").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value,
        eq(&Token::new(
            vec!['a', 'a', 'a'],
            success.context.buffer.clone(),
            0,
            3
        ))
    );
}