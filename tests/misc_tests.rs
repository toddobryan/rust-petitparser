use rust_petitparser::parser::ext::ParserExt;
use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::core::token::line_and_column_of;
use rust_petitparser::parser::character::character::char;
use rust_petitparser::parser::combinator::choice::choice2;
use rust_petitparser::parser::combinator::sequence::seq2;
use rust_petitparser::parser::misc::end::eof;
use rust_petitparser::parser::misc::epsilon::{epsilon, epsilon_with};
use rust_petitparser::parser::misc::failure::{failure, failure_with_message};
use rust_petitparser::parser::misc::success::success;
use rust_petitparser::parser::predicate::predicate::string;
use std::rc::Rc;

fn buf(s: &str) -> Rc<[char]> {
    s.chars().collect::<Vec<_>>().into()
}

macro_rules! assert_success {
    ($parser:expr, $input:expr, $value:expr, $pos:expr) => {
        let result = $parser.parse($input);
        if result.is_err() {
            panic!("Expected success, but got {:?}", result.unwrap_err());
        }
        let result = result.unwrap();
        assert_that!(result.value, eq($value));
        assert_that!(result.context.position, eq($pos));

        let pos = $parser.fast_parse_on(buf($input), 0);
        if pos.is_none() {
            panic!("Expected position after successful parse, but got None");
        }
        let pos = pos.unwrap();
        assert_that!(pos, eq($pos));
    }
}

macro_rules! assert_failure {
    ($parser:expr, $input:expr, $message:expr, $pos:expr) => {
        let result = $parser.parse($input);
        if result.is_ok() {
            panic!("Expected failure, but got success {:?}", result.unwrap());
        }
        let failure = result.unwrap_err();
        assert_that!(failure.message, eq($message));
        assert_that!(failure.context.position, eq($pos));

        let pos = $parser.fast_parse_on(buf($input), 0);
        assert_that!(pos, eq(None));
    }
}

#[gtest]
fn eof_on_empty_input() {
    assert_success!(eof(), "", (), 0);
}

#[gtest]
fn eof_at_end_of_input() {
    let p = seq2(char('a'), eof());
    assert_success!(p, "a", ('a', ()), 1);
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
    assert_success!(p, "hello world", "hello", 5);
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
    assert_success!(p, "héllo world", "héllo", 5);
}

#[gtest]
fn string_emoji() {
    // 🎉 is 4 bytes in UTF-8 but 1 char
    let p = string("🎉🎂");
    assert_success!(p, "🎉🎂 party", "🎉🎂", 2);
}

// labeled

#[gtest]
fn labeled_passes_through_on_success() {
    let p = char('a').labeled("an 'a'");
    assert_success!(p, "abc", 'a', 1);
}

#[gtest]
fn labeled_replaces_message_on_wrong_char() {
    let p = char('a').labeled("an 'a'");
    assert_failure!(p, "bc", "an 'a'", 0);
}

#[gtest]
fn labeled_replaces_message_at_end_of_input() {
    let p = char('a').labeled("an 'a'");
    assert_failure!(p, "", "an 'a'", 0);
}

#[gtest]
fn labeled_works_on_compound_parser() {
    let p = string("hello").labeled("greeting");
    assert_failure!(p, "world", "greeting", 0);
}

// epsilon

#[gtest]
fn epsilon_succeeds_on_nonempty_input() {
    assert_success!(epsilon(), "abc", (), 0);
}

#[gtest]
fn epsilon_succeeds_on_empty_input() {
    assert_success!(epsilon(), "", (), 0);
}

#[gtest]
fn epsilon_does_not_consume_input() {
    let p = seq2(epsilon(), char('a'));
    assert_success!(p, "abc", ((), 'a'), 1);
}

#[gtest]
fn epsilon_with_returns_value() {
    assert_success!(epsilon_with(42), "abc", 42, 0);
}

#[gtest]
fn epsilon_with_does_not_consume_input() {
    let p = seq2(epsilon_with("ok"), char('x'));
    assert_success!(p, "xyz", ("ok", 'x'), 1);
}

// success

#[gtest]
fn success_returns_value_without_consuming() {
    assert_success!(success(99i32), "abc", 99, 0);
}

#[gtest]
fn success_works_on_empty_input() {
    assert_success!(success("done"), "", "done", 0);
}

#[gtest]
fn success_as_choice_fallback() {
    // success(-1) acts as a default when char('a') fails
    let p = choice2(char('a').map(|_| 1i32), success(-1i32));
    assert_success!(p, "abc", 1, 1);
    assert_success!(p, "xyz", -1, 0);
}

// failure

#[gtest]
fn failure_always_fails() {
    assert_failure!(failure(), "abc", "unable to parse", 0);
}

#[gtest]
fn failure_fails_on_empty_input() {
    assert_failure!(failure(), "", "unable to parse", 0);
}

#[gtest]
fn failure_with_message_uses_given_message() {
    assert_failure!(failure_with_message("no match here".to_string()), "abc", "no match here", 0);
}

#[gtest]
fn failure_with_message_preserves_position() {
    let p = seq2(char('a'), failure_with_message("stop".to_string()));
    assert_failure!(p, "abc", "stop", 1);
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
