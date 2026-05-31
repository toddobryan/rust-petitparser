use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{
    any, char, char_ci, digit, digit_with_radix, letter, lowercase, none_of, none_of_ci, one_of,
    one_of_ci, predicate, uppercase, whitespace, word,
};

// char

#[gtest]
fn char_matches() {
    let success = char('a').parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn char_fails_on_wrong_char() {
    let failure = char('a').parse("bc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'a', but found 'b'"));
}

#[gtest]
fn char_fails_at_end_of_input() {
    let failure = char('a').parse("").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected 'a', but reached end of input")
    );
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
    let success = any().parse("xyz").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('x'));
}

#[gtest]
fn any_fails_at_end_of_input() {
    let failure = any().parse("").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected any character, but reached end of input")
    );
}

// letter

#[gtest]
fn letter_matches_letter() {
    let success = letter().parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn letter_fails_on_digit() {
    let failure = letter().parse("1bc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected letter, but found '1'"));
}

#[gtest]
fn letter_fails_at_end_of_input() {
    let failure = letter().parse("").unwrap_err();
    assert_that!(
        failure.message,
        eq("expected letter, but reached end of input")
    );
}

// digit

#[gtest]
fn digit_matches_decimal() {
    let success = digit().parse("5abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('5'));
}

#[gtest]
fn digit_fails_on_letter() {
    let failure = digit().parse("abc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected digit, but found 'a'"));
}

#[gtest]
fn digit_hex_matches_hex_char() {
    let success = digit_with_radix(16).parse("f0").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('f'));
}

#[gtest]
fn digit_hex_fails_on_non_hex() {
    let failure = digit_with_radix(16).parse("gz").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected digit (radix 16), but found 'g'")
    );
}

// one_of / none_of

#[gtest]
fn one_of_matches() {
    let p = one_of("aeiou");
    let success = p.parse("eel").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('e'));
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
    let success = p.parse("xyz").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('x'));
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
    let success = lowercase().parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn lowercase_fails_on_uppercase() {
    let failure = lowercase().parse("Abc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected lowercase letter, but found 'A'")
    );
}

#[gtest]
fn uppercase_matches() {
    let success = uppercase().parse("ABC").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('A'));
}

#[gtest]
fn uppercase_fails_on_lowercase() {
    let failure = uppercase().parse("abc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected uppercase letter, but found 'a'")
    );
}

// whitespace

#[gtest]
fn whitespace_matches_space() {
    let success = whitespace().parse(" x").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq(' '));
}

#[gtest]
fn whitespace_matches_tab() {
    let success = whitespace().parse("\tx").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('\t'));
}

#[gtest]
fn whitespace_fails_on_letter() {
    let failure = whitespace().parse("abc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected whitespace, but found 'a'"));
}

// word

#[gtest]
fn word_matches_letter() {
    let success = word().parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn word_matches_digit() {
    let success = word().parse("1bc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('1'));
}

#[gtest]
fn word_matches_underscore() {
    let success = word().parse("_x").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('_'));
}

#[gtest]
fn word_fails_on_space() {
    let failure = word().parse(" abc").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(
        failure.message,
        eq("expected word character (letter, digit, or '_'), but found ' '")
    );
}

// predicate

#[gtest]
fn predicate_matches() {
    let p = predicate(|c| c == 'x' || c == 'y', "x or y");
    let success = p.parse("xz").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('x'));
}

#[gtest]
fn predicate_fails() {
    let p = predicate(|c| c == 'x' || c == 'y', "x or y");
    let failure = p.parse("az").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected x or y, but found 'a'"));
}

// char_ci

#[gtest]
fn char_ci_matches_same_case() {
    let success = char_ci('a').parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn char_ci_matches_opposite_case() {
    let success = char_ci('a').parse("Abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('A'));
}

#[gtest]
fn char_ci_uppercase_pattern_matches_lowercase() {
    let success = char_ci('A').parse("abc").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('a'));
}

#[gtest]
fn char_ci_fails_on_different_char() {
    let failure = char_ci('a').parse("bcd").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected 'a' (case-insensitive), but found 'b'"));
}

#[gtest]
fn char_ci_fails_at_end_of_input() {
    let failure = char_ci('a').parse("").unwrap_err();
    assert_that!(failure.message, eq("expected 'a' (case-insensitive), but reached end of input"));
}

// one_of_ci

#[gtest]
fn one_of_ci_matches_lowercase() {
    let success = one_of_ci("aeiou").parse("eel").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('e'));
}

#[gtest]
fn one_of_ci_matches_uppercase() {
    let success = one_of_ci("aeiou").parse("Eel").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('E'));
}

#[gtest]
fn one_of_ci_uppercase_pattern_matches_lowercase() {
    let success = one_of_ci("AEIOU").parse("eel").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('e'));
}

#[gtest]
fn one_of_ci_fails_on_non_member() {
    let failure = one_of_ci("aeiou").parse("xyz").unwrap_err();
    assert_that!(failure.context.position, eq(0));
}

// none_of_ci

#[gtest]
fn none_of_ci_matches_non_member() {
    let success = none_of_ci("aeiou").parse("xyz").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq('x'));
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
    let failure = p.parse("b").unwrap_err();
    assert_that!(failure.message, eq("need an 'a' here"));
}

#[gtest]
fn with_message_at_end_of_input() {
    let p = letter().with_message("expected a letter");
    let failure = p.parse("").unwrap_err();
    assert_that!(failure.message, eq("expected a letter"));
}
