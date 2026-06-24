use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};

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
    assert_failure!(
        any(),
        "",
        "expected any character, but reached end of input",
        0
    );
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
    assert_failure!(
        digit_with_radix(16),
        "gz",
        "expected digit (radix 16), but found 'g'",
        0
    );
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
    assert_failure!(
        lowercase(),
        "Abc",
        "expected lowercase letter, but found 'A'",
        0
    );
}

#[gtest]
fn uppercase_matches() {
    assert_success!(uppercase(), "ABC", 'A', 1);
}

#[gtest]
fn uppercase_fails_on_lowercase() {
    assert_failure!(
        uppercase(),
        "abc",
        "expected uppercase letter, but found 'a'",
        0
    );
}

// pattern
#[gtest]
fn single_character_pattern() {
    let p = pattern("y");
    assert_success!(p, "y", 'y', 1);
    assert_failure!(p, "x", "expected 'y', but found 'x'", 0);
    assert_failure!(p, "z", "expected 'y', but found 'z'", 0);
    assert_failure!(p, "5", "expected 'y', but found '5'", 0);
    assert_failure!(p, "Y", "expected 'y', but found 'Y'", 0);
    assert_failure!(p, "\0", "expected 'y', but found '\0'", 0);
    assert_failure!(p, "😮", "expected 'y', but found '😮'", 0);
}

#[gtest]
fn single_character_pattern_ci() {
    let p = pattern_ci("a");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "A", 'A', 1);
    assert_failure!(p, "b", "expected 'a' (case-insensitive), but found 'b'", 0);
    assert_failure!(p, "B", "expected 'a' (case-insensitive), but found 'B'", 0);
    assert_failure!(
        p,
        "\0",
        "expected 'a' (case-insensitive), but found '\0'",
        0
    );
    assert_failure!(p, "&", "expected 'a' (case-insensitive), but found '&'", 0);
}

#[gtest]
fn unicode() {
    let p = pattern("😮");
    assert_success!(p, "😮", '😮', 1);
    assert_failure!(p, "x", "expected '😮', but found 'x'", 0);
    assert_failure!(p, "z", "expected '😮', but found 'z'", 0);
    assert_failure!(p, "5", "expected '😮', but found '5'", 0);
    assert_failure!(p, "\0", "expected '😮', but found '\0'", 0);
    assert_failure!(p, "😃", "expected '😮', but found '😃'", 0);
}

#[gtest]
fn negated_pattern() {
    let p = pattern("^y");
    assert_success!(p, "x", 'x', 1);
    assert_success!(p, "z", 'z', 1);
    assert_success!(p, "5", '5', 1);
    assert_success!(p, "\0", '\0', 1);
    assert_failure!(p, "y", "expected ^'y', but found 'y'", 0);
}

#[gtest]
fn default_multi_character_pattern() {
    let p = pattern("ab-");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "-", '-', 1);
    assert_failure!(p, "d", "expected [ab-], but found 'd'", 0);
    assert_failure!(p, "e", "expected [ab-], but found 'e'", 0);
    assert_failure!(p, "A", "expected [ab-], but found 'A'", 0);
    assert_failure!(p, "B", "expected [ab-], but found 'B'", 0);
    assert_failure!(p, "f", "expected [ab-], but found 'f'", 0);
}

#[gtest]
fn unicode_multi_character_pattern() {
    let p = pattern("y😃💕");
    assert_success!(p, "y", 'y', 1);
    assert_success!(p, "😃", '😃', 1);
    assert_success!(p, "💕", '💕', 1);
    assert_failure!(p, "x", "expected [y😃💕], but found 'x'", 0);
    assert_failure!(p, "z", "expected [y😃💕], but found 'z'", 0);
    assert_failure!(p, "💞", "expected [y😃💕], but found '💞'", 0);
}

#[gtest]
fn negated_multi_character_pattern() {
    let p = pattern("^ab-");
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "e", 'e', 1);
    assert_success!(p, "f", 'f', 1);
    assert_failure!(p, "a", "expected [^ab-], but found 'a'", 0);
    assert_failure!(p, "b", "expected [^ab-], but found 'b'", 0);
    assert_failure!(p, "-", "expected [^ab-], but found '-'", 0);
}

#[gtest]
fn range() {
    let p = pattern("a-c");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "c", 'c', 1);
    assert_failure!(p, "d", "expected [a-c], but found 'd'", 0);
    assert_failure!(p, "e", "expected [a-c], but found 'e'", 0);
    assert_failure!(p, "f", "expected [a-c], but found 'f'", 0);
}

#[gtest]
fn negated_range() {
    let p = pattern("^a-c");
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "e", 'e', 1);
    assert_success!(p, "f", 'f', 1);
    assert_failure!(p, "a", "expected [^a-c], but found 'a'", 0);
    assert_failure!(p, "b", "expected [^a-c], but found 'b'", 0);
    assert_failure!(p, "c", "expected [^a-c], but found 'c'", 0);
}

#[gtest]
fn overlapping_range() {
    let p = pattern("b-da-c");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "c", 'c', 1);
    assert_success!(p, "d", 'd', 1);
    assert_failure!(p, "e", "expected [b-da-c], but found 'e'", 0);
    assert_failure!(p, "f", "expected [b-da-c], but found 'f'", 0);
    assert_failure!(p, "g", "expected [b-da-c], but found 'g'", 0);
}

#[gtest]
fn adjacent_range() {
    let p = pattern("c-ea-c");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "c", 'c', 1);
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "e", 'e', 1);
    assert_failure!(p, "f", "expected [c-ea-c], but found 'f'", 0);
}

#[gtest]
fn prefix_range() {
    let p = pattern("a-ea-c");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "c", 'c', 1);
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "e", 'e', 1);
    assert_failure!(p, "f", "expected [a-ea-c], but found 'f'", 0);
}

#[gtest]
fn postfix_range() {
    let p = pattern("a-ec-e");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "c", 'c', 1);
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "e", 'e', 1);
    assert_failure!(p, "f", "expected [a-ec-e], but found 'f'", 0);
}

#[gtest]
fn repeated_range() {
    let p = pattern("a-ea-e");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "c", 'c', 1);
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "e", 'e', 1);
    assert_failure!(p, "f", "expected [a-ea-e], but found 'f'", 0);
}

#[gtest]
fn composed_range() {
    let p = pattern("ac-df-");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "c", 'c', 1);
    assert_success!(p, "d", 'd', 1);
    assert_success!(p, "f", 'f', 1);
    assert_success!(p, "-", '-', 1);
    assert_failure!(p, "b", "expected [ac-df-], but found 'b'", 0);
    assert_failure!(p, "e", "expected [ac-df-], but found 'e'", 0);
    assert_failure!(p, "g", "expected [ac-df-], but found 'g'", 0);
}

#[gtest]
fn multiple_pattern_ci() {
    let p = pattern_ci("ab-");
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "A", 'A', 1);
    assert_success!(p, "b", 'b', 1);
    assert_success!(p, "B", 'B', 1);
    assert_success!(p, "-", '-', 1);
    assert_failure!(
        p,
        "c",
        "expected [ab-] (case-insensitive), but found 'c'",
        0
    );
    assert_failure!(
        p,
        "C",
        "expected [ab-] (case-insensitive), but found 'C'",
        0
    );
    assert_failure!(
        p,
        "\0",
        "expected [ab-] (case-insensitive), but found '\0'",
        0
    );
    assert_failure!(
        p,
        "&",
        "expected [ab-] (case-insensitive), but found '&'",
        0
    );
}

#[gtest]
fn everything_range() {
    let p = pattern("\u{0}-\u{10ffff}");
    assert_success!(p, "\0", '\0', 1);
    assert_success!(p, "\u{ffff}", '\u{ffff}', 1);
    assert_success!(p, "\u{10ffff}", '\u{10ffff}', 1);
}

#[gtest]
fn negated_everything_range() {
    let p = pattern("^\u{0}-\u{10ffff}");
    assert_failure!(p, "\0", "expected [^\0-\u{10ffff}], but found '\0'", 0);
    assert_failure!(
        p,
        "\u{ffff}",
        "expected [^\0-\u{10ffff}], but found '\u{ffff}'",
        0
    );
    assert_failure!(
        p,
        "\u{10ffff}",
        "expected [^\0-\u{10ffff}], but found '\u{10ffff}'",
        0
    );
}

#[gtest]
fn nothing_pattern() {
    let p = pattern("");
    assert_failure!(p, "\0", "expected [], but found '\0'", 0);
    assert_failure!(p, "\u{ffff}", "expected [], but found '\u{ffff}'", 0);
}

#[gtest]
fn negated_nothing_pattern() {
    let p = pattern("^");
    assert_success!(p, "\0", '\0', 1);
    assert_success!(p, "\u{ffff}", '\u{ffff}', 1);
}

#[gtest]
fn large_range() {
    let p = pattern("\u{2200}-\u{22ff}\u{27c0}-\u{27ef}\u{2980}-\u{29ff}");
    assert_success!(p, "\u{2209}", '\u{2209}', 1); // ∉
    assert_success!(p, "\u{27c3}", '\u{27c3}', 1); // ⟃
    assert_success!(p, "\u{29fb}", '\u{29fb}', 1); // ⦻
    assert_failure!(
        p,
        "a",
        "expected [\u{2200}-\u{22ff}\u{27c0}-\u{27ef}\u{2980}-\u{29ff}], but found 'a'",
        0
    );
    assert_failure!(
        p,
        "9",
        "expected [\u{2200}-\u{22ff}\u{27c0}-\u{27ef}\u{2980}-\u{29ff}], but found '9'",
        0
    );
    assert_failure!(
        p,
        "*",
        "expected [\u{2200}-\u{22ff}\u{27c0}-\u{27ef}\u{2980}-\u{29ff}], but found '*'",
        0
    );
}

// Dart's `pattern('c-a')` throws an assertion error (`RangeCharPredicate` asserts
// `start <= stop`). We deliberately don't validate/normalize reversed ranges here, so a
// reversed range just matches nothing rather than panicking or being swapped.
#[gtest]
fn reversed_range_matches_nothing() {
    let p = pattern("c-a");
    assert_failure!(p, "a", "expected [c-a], but found 'a'", 0);
    assert_failure!(p, "b", "expected [c-a], but found 'b'", 0);
    assert_failure!(p, "c", "expected [c-a], but found 'c'", 0);
}

// range

#[gtest]
fn range_default() {
    let p = rust_petitparser::prelude::range('e', 'o');
    for c in ['e', 'i', 'o'] {
        assert_success!(p, c.to_string().as_str(), c, 1);
    }
    for c in ['p', 'd', '9'] {
        assert_failure!(
            p,
            c.to_string().as_str(),
            format!("expected [e-o], but found '{}'", c).as_str(),
            0
        );
    }
}

#[gtest]
fn range_message() {
    let p = rust_petitparser::prelude::range('x', 'z').with_message("variable expected");
    for c in ['x', 'y', 'z'] {
        assert_success!(p, c.to_string().as_str(), c, 1);
    }
    for c in ['p', 'd', '9'] {
        assert_failure!(p, c.to_string().as_str(), "variable expected", 0);
    }
}

#[gtest]
fn range_unicode() {
    let p = rust_petitparser::prelude::range('😁', '😄');
    for c in ['😁', '😃', '😄'] {
        assert_success!(p, c.to_string().as_str(), c, 1);
    }
    for c in ['😀', '😅', '9'] {
        assert_failure!(
            p,
            c.to_string().as_str(),
            format!("expected [😁-😄], but found '{}'", c).as_str(),
            0,
        );
    }
}

#[gtest]
#[should_panic(expected = "In range, start must be <= end")]
fn invalid_range() {
    let _ = rust_petitparser::prelude::range('o', 'e');
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
    assert_failure!(
        word(),
        " abc",
        "expected word character (letter, digit, or '_'), but found ' '",
        0
    );
}

// predicate

#[gtest]
fn predicate_matches() {
    let p = char_if(|c| c == 'x' || c == 'y', "x or y");
    assert_success!(p, "xz", 'x', 1);
}

#[gtest]
fn predicate_fails() {
    let p = char_if(|c| c == 'x' || c == 'y', "x or y");
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
    assert_failure!(
        char_ci('a'),
        "bcd",
        "expected 'a' (case-insensitive), but found 'b'",
        0
    );
}

#[gtest]
fn char_ci_fails_at_end_of_input() {
    assert_failure!(
        char_ci('a'),
        "",
        "expected 'a' (case-insensitive), but reached end of input",
        0
    );
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

// any: accepts arbitrary code points (including astral/emoji)

#[gtest]
fn any_matches_unicode() {
    assert_success!(any(), "λx", 'λ', 1);
    assert_success!(any(), "🤔!", '🤔', 1);
}

// empty-input failures (each char parser must fail at position 0 on "")

#[gtest]
fn digit_fails_at_end_of_input() {
    assert_failure!(digit(), "", "expected digit, but reached end of input", 0);
}

#[gtest]
fn lowercase_fails_at_end_of_input() {
    assert_failure!(
        lowercase(),
        "",
        "expected lowercase letter, but reached end of input",
        0
    );
}

#[gtest]
fn uppercase_fails_at_end_of_input() {
    assert_failure!(
        uppercase(),
        "",
        "expected uppercase letter, but reached end of input",
        0
    );
}

#[gtest]
fn whitespace_fails_at_end_of_input() {
    assert_failure!(
        whitespace(),
        "",
        "expected whitespace, but reached end of input",
        0
    );
}

#[gtest]
fn word_fails_at_end_of_input() {
    assert_failure!(
        word(),
        "",
        "expected word character (letter, digit, or '_'), but reached end of input",
        0
    );
}

// whitespace: full set of accepted code points + non-whitespace rejection

#[gtest]
fn whitespace_matches_newline() {
    assert_success!(whitespace(), "\nx", '\n', 1);
}

#[gtest]
fn whitespace_matches_carriage_return() {
    assert_success!(whitespace(), "\rx", '\r', 1);
}

#[gtest]
fn whitespace_matches_form_feed() {
    assert_success!(whitespace(), "\u{0c}x", '\u{0c}', 1);
}

#[gtest]
fn whitespace_matches_vertical_tab() {
    assert_success!(whitespace(), "\u{0b}x", '\u{0b}', 1);
}

#[gtest]
fn whitespace_fails_on_digit() {
    assert_failure!(whitespace(), "1x", "expected whitespace, but found '1'", 0);
}

// per-parser custom messages (with_message)

#[gtest]
fn any_with_message() {
    assert_failure!(
        any().with_message("something expected"),
        "",
        "something expected",
        0
    );
}

#[gtest]
fn digit_with_message() {
    assert_failure!(
        digit().with_message("number expected"),
        "a",
        "number expected",
        0
    );
}

#[gtest]
fn whitespace_with_message() {
    assert_failure!(
        whitespace().with_message("gimme space"),
        "a",
        "gimme space",
        0
    );
}

#[gtest]
fn one_of_with_message() {
    let p = one_of("02468").with_message("even digit");
    assert_failure!(p, "1", "even digit", 0);
}

#[gtest]
fn none_of_with_message() {
    let p = none_of("02468").with_message("no even digit");
    assert_failure!(p, "2", "no even digit", 0);
}
