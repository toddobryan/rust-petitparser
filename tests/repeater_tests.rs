use std::panic;
use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{char, letter};
use rust_petitparser::parser::ext::ParserExt;
use std::rc::Rc;

fn buf(s: &str) -> Rc<[char]> {
    s.chars().collect::<Vec<_>>().into()
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
fn star_with_opt_that_doesnt_consume_should_panic() {
    let p = char('x').star().opt();
    let orig = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result =
        panic::catch_unwind(|| p.parse("y"));
    panic::set_hook(orig);

}

#[gtest]
fn rep_sep() {
    let p = letter().rep_sep(char(','), 0, None);
    let success = p.parse("a,b,c").unwrap();
    assert_that!(success.context.position, eq(5));
    assert_that!(success.value, eq(&vec!['a', 'b', 'c']));
}

#[gtest]
fn star_sep_matches_multiple() {
    let p = letter().star_sep(char(','));
    let success = p.parse("a,b,c").unwrap();
    assert_that!(success.context.position, eq(5));
    assert_that!(success.value, eq(&vec!['a', 'b', 'c']));
}

#[gtest]
fn star_sep_matches_single() {
    let p = letter().star_sep(char(','));
    let success = p.parse("a").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq(&vec!['a']));
}

#[gtest]
fn star_sep_matches_empty() {
    let p = letter().star_sep(char(','));
    let success = p.parse("123").unwrap();
    assert_that!(success.context.position, eq(0));
    assert_that!(success.value, eq(&vec![]));
}

#[gtest]
fn star_sep_stops_before_trailing_sep() {
    let p = letter().star_sep(char(','));
    let success = p.parse("a,b,").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&vec!['a', 'b']));
}

#[gtest]
fn plus_sep_matches_multiple() {
    let p = letter().plus_sep(char(','));
    let success = p.parse("a,b,c").unwrap();
    assert_that!(success.context.position, eq(5));
    assert_that!(success.value, eq(&vec!['a', 'b', 'c']));
}

#[gtest]
fn plus_sep_matches_single() {
    let p = letter().plus_sep(char(','));
    let success = p.parse("a").unwrap();
    assert_that!(success.context.position, eq(1));
    assert_that!(success.value, eq(&vec!['a']));
}

#[gtest]
fn plus_sep_fails_on_empty() {
    let p = letter().plus_sep(char(','));
    let failure = p.parse("123").unwrap_err();
    assert_that!(failure.context.position, eq(0));
    assert_that!(failure.message, eq("expected letter, but found '1'"));
}

#[gtest]
fn plus_sep_stops_before_trailing_sep() {
    let p = letter().plus_sep(char(','));
    let success = p.parse("a,b,").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(success.value, eq(&vec!['a', 'b']));
}

#[gtest]
fn fast_rep_sep() {
    let p = letter().rep_sep(char(','), 0, None);
    let result = p.fast_parse_on(buf("a,b,c"), 0);
    assert_that!(result, eq(Some(5)));
}

#[gtest]
fn fast_star_sep_matches_multiple() {
    let p = letter().star_sep(char(','));
    let result = p.fast_parse_on(buf("a,b,c"), 0);
    assert_that!(result, eq(Some(5)));
}

#[gtest]
fn fast_star_sep_matches_single() {
    let p = letter().star_sep(char(','));
    let result = p.fast_parse_on(buf("a"), 0);
    assert_that!(result, eq(Some(1)));
}

#[gtest]
fn fast_star_sep_matches_empty() {
    let p = letter().star_sep(char(','));
    let result = p.fast_parse_on(buf("123"), 0);
    assert_that!(result, eq(Some(0)));
}

#[gtest]
fn fast_star_sep_stops_before_trailing_sep() {
    let p = letter().star_sep(char(','));
    let result = p.fast_parse_on(buf("a,b,"), 0);
    assert_that!(result, eq(Some(3)));
}

#[gtest]
fn fast_plus_sep_matches_multiple() {
    let p = letter().plus_sep(char(','));
    let result = p.fast_parse_on(buf("a,b,c"), 0);
    assert_that!(result, eq(Some(5)));
}

#[gtest]
fn fast_plus_sep_matches_single() {
    let p = letter().plus_sep(char(','));
    let result = p.fast_parse_on(buf("a"), 0);
    assert_that!(result, eq(Some(1)));
}

#[gtest]
fn fast_plus_sep_fails_on_empty() {
    let p = letter().plus_sep(char(','));
    let result = p.fast_parse_on(buf("123"), 0);
    assert_that!(result, eq(None));
}

#[gtest]
fn fast_plus_sep_stops_before_trailing_sep() {
    let p = letter().plus_sep(char(','));
    let result = p.fast_parse_on(buf("a,b,"), 0);
    assert_that!(result, eq(Some(3)));
}
