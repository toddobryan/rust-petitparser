use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};

#[gtest]
fn choice_macro_dispatches_to_choice2() {
    let p = choice!(char('a'), char('b'));
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "b", 'b', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice3() {
    let p = choice!(char('a'), char('b'), char('c'));
    assert_success!(p, "c", 'c', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice4() {
    let p = choice!(char('a'), char('b'), char('c'), char('d'));
    assert_success!(p, "d", 'd', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice5() {
    let p = choice!(char('a'), char('b'), char('c'), char('d'), char('e'));
    assert_success!(p, "e", 'e', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice6() {
    let p = choice!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f')
    );
    assert_success!(p, "f", 'f', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice7() {
    let p = choice!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
        char('g')
    );
    assert_success!(p, "g", 'g', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice8() {
    let p = choice!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
        char('g'),
        char('h')
    );
    assert_success!(p, "h", 'h', 1);
}

#[gtest]
fn choice_macro_dispatches_to_choice9() {
    let p = choice!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
        char('g'),
        char('h'),
        char('i')
    );
    assert_success!(p, "i", 'i', 1);
}

#[gtest]
fn choice_macro_fails_when_no_alternative_matches() {
    let p = choice!(char('a'), char('b'), char('c'));
    assert_failure!(p, "x", "expected 'c', but found 'x'", 0);
}
