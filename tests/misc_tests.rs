use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::core::token::line_and_column_of;
use rust_petitparser::parser::character::character::char;
use rust_petitparser::parser::combinator::sequence::seq2;
use rust_petitparser::parser::misc::end::eof;
use rust_petitparser::parser::predicate::predicate::string;
use std::rc::Rc;

fn buf(s: &str) -> Rc<[char]> {
    s.chars().collect::<Vec<_>>().into()
}

#[gtest]
fn eof_on_empty_input() {
    let p = eof();
    let success = p.parse("").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(()));
}

#[gtest]
fn eof_at_end_of_input() {
    let p = seq2(char('a'), eof());
    let success = p.parse("a").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value.0, eq('a'));
}

#[gtest]
fn eof_fails_mid_input() {
    let p = eof();
    let failure = p.parse("a").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

#[gtest]
fn string_matches_literal() {
    let p = string("hello");
    let success = p.parse("hello world").unwrap();
    assert_that!(success.context.position, eq(5));
    assert_that!(success.value, eq("hello"));
}

#[gtest]
fn string_fails_on_mismatch() {
    let p = string("hello");
    assert!(p.parse("world").is_err());
}

#[gtest]
fn string_fails_on_short_input() {
    let p = string("hello");
    assert!(p.parse("hi").is_err());
}

#[gtest]
fn string_followed_by_eof() {
    let p = seq2(string("ok"), eof());
    assert!(p.parse("ok").is_ok());
    assert!(p.parse("ok!").is_err());
}

#[gtest]
fn string_unicode() {
    // "é" is 2 bytes in UTF-8 but 1 char — length must be in chars, not bytes
    let p = string("héllo");
    let success = p.parse("héllo world").unwrap();
    assert_that!(success.context.position, eq(5));
    assert_that!(success.value, eq("héllo"));
}

#[gtest]
fn string_emoji() {
    // 🎉 is 4 bytes in UTF-8 but 1 char
    let p = string("🎉🎂");
    let success = p.parse("🎉🎂 party").unwrap();
    assert_that!(success.context.position, eq(2));
    assert_that!(success.value, eq("🎉🎂"));
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
