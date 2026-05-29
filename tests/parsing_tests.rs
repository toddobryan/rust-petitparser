use std::rc::Rc;
use rust_petitparser::parser::ext::ParserExt;
use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::core::token::{line_and_column_of, Token};
use rust_petitparser::parser::character::character::char;
use rust_petitparser::parser::combinator::choice::{choice2, choice3, Choice2, SELECT_FARTHEST_JOINED};
use rust_petitparser::parser::combinator::sequence::{seq2, seq3, seq4};
use rust_petitparser::parser::combinator::settable::SettableParser;

fn buf(s: &str) -> Rc<[char]> {
    s.chars().collect::<Vec<_>>().into()
}

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
fn plus_with_opt_test() {
    let p = char('x').plus().opt();
    let success = p.parse("xxxy").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&Some(vec!['x', 'x', 'x'])));

    let success = p.parse("y").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(&None));
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
fn star_with_opt_test() {
    let p = char('x').star().opt();
    let success = p.parse("xxxy").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&Some(vec!['x', 'x', 'x'])));
}

#[gtest]
#[should_panic(expected="")]
fn star_with_opt_that_doesnt_consume_should_panic() {
    let p = char('x').star().opt();
    let success = p.parse("y").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(&None));
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

#[gtest]
fn all_matches_basic() {
    let buffer: Rc<[char]> = "abacada".chars().collect::<Vec<_>>().into();
    let matches: Vec<char> = char('a').all_matches(buffer, 0, false).into_iter().collect();
    assert_that!(matches, eq(&vec!['a', 'a', 'a', 'a']));
}

#[gtest]
fn all_matches_none() {
    let buffer: Rc<[char]> = "bcdef".chars().collect::<Vec<_>>().into();
    let matches: Vec<char> = char('a').all_matches(buffer, 0, false).into_iter().collect();
    assert_that!(matches, eq(&vec![]));
}

#[gtest]
fn all_matches_start_position() {
    let buffer: Rc<[char]> = "aaba".chars().collect::<Vec<_>>().into();
    let matches: Vec<char> = char('a').all_matches(buffer, 2, false).into_iter().collect();
    assert_that!(matches, eq(&vec!['a']));
}

#[gtest]
fn all_matches_non_overlapping() {
    let buffer: Rc<[char]> = "aaaa".chars().collect::<Vec<_>>().into();
    let matches: Vec<(char, char)> = seq2(char('a'), char('a'))
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![('a', 'a'), ('a', 'a')]));
}

#[gtest]
fn line_and_column_of_single_line() {
    let b = buf("abc");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    assert_that!(line_and_column_of(b.clone(), 1), eq((1, 2)));
    assert_that!(line_and_column_of(b.clone(), 2), eq((1, 3)));
}

#[gtest]
fn line_and_column_of_empty_buffer() {
    let b = buf("");
    assert_that!(line_and_column_of(b, 0), eq((1, 1)));
}

#[gtest]
fn line_and_column_of_lf_newline() {
    let b = buf("ab\ncd");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    assert_that!(line_and_column_of(b.clone(), 1), eq((1, 2)));
    // position on '\n' itself is still treated as line 1
    assert_that!(line_and_column_of(b.clone(), 2), eq((1, 3)));
    // first character after newline
    assert_that!(line_and_column_of(b.clone(), 3), eq((2, 1)));
    assert_that!(line_and_column_of(b.clone(), 4), eq((2, 2)));
}

#[gtest]
fn line_and_column_of_crlf_newline() {
    let b = buf("a\r\nb");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    // position on '\r' — still line 1
    assert_that!(line_and_column_of(b.clone(), 1), eq((1, 2)));
    // position on '\n' inside the \r\n token — still line 1
    assert_that!(line_and_column_of(b.clone(), 2), eq((1, 3)));
    // first character after \r\n
    assert_that!(line_and_column_of(b.clone(), 3), eq((2, 1)));
}

#[gtest]
fn line_and_column_of_cr_newline() {
    let b = buf("a\rb");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    // position on '\r' — still line 1
    assert_that!(line_and_column_of(b.clone(), 1), eq((1, 2)));
    // first character after \r
    assert_that!(line_and_column_of(b.clone(), 2), eq((2, 1)));
}

#[gtest]
fn line_and_column_of_multiple_lines() {
    let b = buf("a\nb\nc");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    assert_that!(line_and_column_of(b.clone(), 1), eq((1, 2)));
    assert_that!(line_and_column_of(b.clone(), 2), eq((2, 1)));
    assert_that!(line_and_column_of(b.clone(), 3), eq((2, 2)));
    assert_that!(line_and_column_of(b.clone(), 4), eq((3, 1)));
}

#[gtest]
fn line_and_column_of_position_past_end() {
    let b = buf("ab\ncd");
    // position equal to buffer length: column should be (len - last_offset + 1)
    assert_that!(line_and_column_of(b, 5), eq((2, 3)));
}

#[gtest]
fn line_and_column_of_blank_line() {
    // two newlines in a row create an empty middle line
    let b = buf("a\n\nb");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    assert_that!(line_and_column_of(b.clone(), 2), eq((2, 1)));
    assert_that!(line_and_column_of(b.clone(), 3), eq((3, 1)));
}

#[gtest]
fn line_and_column_of_mixed_line_endings() {
    // \n then \r then \r\n
    let b = buf("a\nb\rc\r\nd");
    assert_that!(line_and_column_of(b.clone(), 0), eq((1, 1)));
    assert_that!(line_and_column_of(b.clone(), 2), eq((2, 1))); // 'b'
    assert_that!(line_and_column_of(b.clone(), 4), eq((3, 1))); // 'c'
    assert_that!(line_and_column_of(b.clone(), 7), eq((4, 1))); // 'd'
}

#[gtest]
fn token_line_and_column() {
    // Verify Token::line() / Token::column() use the token's start position
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
fn all_matches_overlapping() {
    let buffer: Rc<[char]> = "aaaa".chars().collect::<Vec<_>>().into();
    let matches: Vec<(char, char)> = seq2(char('a'), char('a'))
        .all_matches(buffer, 0, true)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![('a', 'a'), ('a', 'a'), ('a', 'a')]));
}

#[gtest]
fn nested_parens_settable_test() {
    let mut expr = SettableParser::<i32>::undefined();
    let inner = seq3(char('('), expr.clone(), char(')')).map(|(_,n,_)| n + 1);
    let leaf = char('x').map(|_| 0);
    expr.set(choice2(inner, leaf));

    assert_eq!(expr.parse("x").unwrap().value, 0);
    assert_eq!(expr.parse("(((x)))").unwrap().value, 3);
    assert!(expr.parse("(x").is_err());
}