use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{
    any, char, char_ci, digit, digit_with_radix, letter, lowercase, none_of, none_of_ci, one_of,
    one_of_ci, predicate, uppercase, whitespace, word,
};
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

// char

#[gtest]
fn char_matches() {
    assert_success!(char('a'), "abc", 'a', 1);
}

#[gtest]
fn char_fails_on_wrong_char() {
    assert_failure!(char('a'), "bc", "expected 'a', but found 'b'", 0);
}

#[gtest]
fn char_fails_at_end_of_input() {
    assert_failure!(char('a'), "", "expected 'a', but reached end of input", 0);
}

#[gtest]
fn char_parser_equality() {
    let a = char('a');
    let b = char('b');
    let a2 = char('a');

    assert_that!(a, eq(&a2));
    assert_that!(a, not(eq(&b)));
}

// any

#[gtest]
fn any_matches_any_char() {
    assert_success!(any(), "xyz", 'x', 1);
}

#[gtest]
fn any_fails_at_end_of_input() {
    assert_failure!(any(), "", "expected any character, but reached end of input", 0);
}

// letter

#[gtest]
fn letter_matches_letter() {
    assert_success!(letter(), "abc", 'a', 1);
}

#[gtest]
fn letter_fails_on_digit() {
    assert_failure!(letter(), "1bc", "expected letter, but found '1'", 0);
}

#[gtest]
fn letter_fails_at_end_of_input() {
    assert_failure!(letter(), "", "expected letter, but reached end of input", 0);
}

// digit

#[gtest]
fn digit_matches_decimal() {
    assert_success!(digit(), "5abc", '5', 1);
}

#[gtest]
fn digit_fails_on_letter() {
    assert_failure!(digit(), "abc", "expected digit, but found 'a'", 0);
}

#[gtest]
fn digit_hex_matches_hex_char() {
    assert_success!(digit_with_radix(16), "f0", 'f', 1);
}

#[gtest]
fn digit_hex_fails_on_non_hex() {
    assert_failure!(digit_with_radix(16), "gz", "expected digit (radix 16), but found 'g'", 0);
}

// one_of / none_of

#[gtest]
fn one_of_matches() {
    let p = one_of("aeiou");
    assert_success!(p, "eel", 'e', 1);
}

#[gtest]
fn one_of_fails_on_non_member() {
    let p = one_of("aeiou");
    let failure = p.parse("xyz").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

#[gtest]
fn none_of_matches_non_member() {
    let p = none_of("aeiou");
    assert_success!(p, "xyz", 'x', 1);
}

#[gtest]
fn none_of_fails_on_member() {
    let p = none_of("aeiou");
    let failure = p.parse("eel").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

// lowercase / uppercase

#[gtest]
fn lowercase_matches() {
    assert_success!(lowercase(), "abc", 'a', 1);
}

#[gtest]
fn lowercase_fails_on_uppercase() {
    assert_failure!(lowercase(), "Abc", "expected lowercase letter, but found 'A'", 0);
}

#[gtest]
fn uppercase_matches() {
    assert_success!(uppercase(), "ABC", 'A', 1);
}

#[gtest]
fn uppercase_fails_on_lowercase() {
    assert_failure!(uppercase(), "abc", "expected uppercase letter, but found 'a'", 0);
}

// whitespace

#[gtest]
fn whitespace_matches_space() {
    assert_success!(whitespace(), " x", ' ', 1);
}

#[gtest]
fn whitespace_matches_tab() {
    assert_success!(whitespace(), "\tx", '\t', 1);
}

#[gtest]
fn whitespace_fails_on_letter() {
    assert_failure!(whitespace(), "abc", "expected whitespace, but found 'a'", 0);
}

// word

#[gtest]
fn word_matches_letter() {
    assert_success!(word(), "abc", 'a', 1);
}

#[gtest]
fn word_matches_digit() {
    assert_success!(word(), "1bc", '1', 1);
}

#[gtest]
fn word_matches_underscore() {
    assert_success!(word(), "_x", '_', 1);
}

#[gtest]
fn word_fails_on_space() {
    assert_failure!(word(), " abc", "expected word character (letter, digit, or '_'), but found ' '", 0);
}

// predicate

#[gtest]
fn predicate_matches() {
    let p = predicate(|c| c == 'x' || c == 'y', "x or y");
    assert_success!(p, "xz", 'x', 1);
}

#[gtest]
fn predicate_fails() {
    let p = predicate(|c| c == 'x' || c == 'y', "x or y");
    assert_failure!(p, "az", "expected x or y, but found 'a'", 0);
}

// char_ci

#[gtest]
fn char_ci_matches_same_case() {
    assert_success!(char_ci('a'), "abc", 'a', 1);
}

#[gtest]
fn char_ci_matches_opposite_case() {
    assert_success!(char_ci('a'), "Abc", 'A', 1);
}

#[gtest]
fn char_ci_uppercase_pattern_matches_lowercase() {
    assert_success!(char_ci('A'), "abc", 'a', 1);
}

#[gtest]
fn char_ci_fails_on_different_char() {
    assert_failure!(char_ci('a'), "bcd", "expected 'a' (case-insensitive), but found 'b'", 0);
}

#[gtest]
fn char_ci_fails_at_end_of_input() {
    assert_failure!(char_ci('a'), "", "expected 'a' (case-insensitive), but reached end of input", 0);
}

// one_of_ci

#[gtest]
fn one_of_ci_matches_lowercase() {
    assert_success!(one_of_ci("aeiou"), "eel", 'e', 1);
}

#[gtest]
fn one_of_ci_matches_uppercase() {
    assert_success!(one_of_ci("aeiou"), "Eel", 'E', 1);
}

#[gtest]
fn one_of_ci_uppercase_pattern_matches_lowercase() {
    assert_success!(one_of_ci("AEIOU"), "eel", 'e', 1);
}

#[gtest]
fn one_of_ci_fails_on_non_member() {
    let failure = one_of_ci("aeiou").parse("xyz").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

// none_of_ci

#[gtest]
fn none_of_ci_matches_non_member() {
    assert_success!(none_of_ci("aeiou"), "xyz", 'x', 1);
}

#[gtest]
fn none_of_ci_fails_on_lowercase_member() {
    let failure = none_of_ci("aeiou").parse("eel").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

#[gtest]
fn none_of_ci_fails_on_uppercase_member() {
    let failure = none_of_ci("aeiou").parse("Eel").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

#[gtest]
fn none_of_ci_uppercase_pattern_fails_on_lowercase() {
    let failure = none_of_ci("AEIOU").parse("eel").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

// with_message

#[gtest]
fn with_message_overrides_default() {
    let p = char('a').with_message("need an 'a' here");
    assert_failure!(p, "b", "need an 'a' here", 0);
}

#[gtest]
fn with_message_at_end_of_input() {
    let p = letter().with_message("expected a letter");
    assert_failure!(p, "", "expected a letter", 0);
}
