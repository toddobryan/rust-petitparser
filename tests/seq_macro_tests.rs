use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};

#[gtest]
fn seq_macro_dispatches_to_seq2() {
    let p = seq!(char('a'), char('b'));
    assert_success!(p, "ab", ('a', 'b'));
}

#[gtest]
fn seq_macro_dispatches_to_seq3() {
    let p = seq!(char('a'), char('b'), char('c'));
    assert_success!(p, "abc", ('a', 'b', 'c'));
}

#[gtest]
fn seq_macro_dispatches_to_seq4() {
    let p = seq!(char('a'), char('b'), char('c'), char('d'));
    assert_success!(p, "abcd", ('a', 'b', 'c', 'd'));
}

#[gtest]
fn seq_macro_dispatches_to_seq5() {
    let p = seq!(char('a'), char('b'), char('c'), char('d'), char('e'));
    assert_success!(p, "abcde", ('a', 'b', 'c', 'd', 'e'));
}

#[gtest]
fn seq_macro_dispatches_to_seq6() {
    let p = seq!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f')
    );
    assert_success!(p, "abcdef", ('a', 'b', 'c', 'd', 'e', 'f'));
}

#[gtest]
fn seq_macro_dispatches_to_seq7() {
    let p = seq!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
        char('g')
    );
    assert_success!(p, "abcdefg", ('a', 'b', 'c', 'd', 'e', 'f', 'g'));
}

#[gtest]
fn seq_macro_dispatches_to_seq8() {
    let p = seq!(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
        char('g'),
        char('h')
    );
    assert_success!(p, "abcdefgh", ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'));
}

#[gtest]
fn seq_macro_dispatches_to_seq9() {
    let p = seq!(
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
    assert_success!(
        p,
        "abcdefghi",
        ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i')
    );
}

#[gtest]
fn seq_macro_propagates_failure_from_a_member_parser() {
    let p = seq!(char('a'), char('b'), char('c'));
    assert_failure!(p, "abx", "expected 'c', but found 'x'", 2);
}

#[gtest]
fn seq_macro_composes_with_map3() {
    let p = seq!(char('a'), char('b'), char('c')).map3(|a, b, c| format!("{a}{b}{c}"));
    assert_success!(p, "abc", &"abc".to_string());
}

#[gtest]
fn seq_macro_fuses_trailing_closure() {
    let p = seq!(char('a'), char('b'), char('c'), |a, b, c| format!(
        "{a}{b}{c}"
    ));
    assert_success!(p, "abc", &"abc".to_string());
}

#[gtest]
fn seq_macro_fuses_trailing_closure_at_min_arity() {
    let p = seq!(char('a'), char('b'), |a, b| (b, a));
    assert_success!(p, "ab", ('b', 'a'));
}

#[gtest]
fn seq_macro_fused_closure_failure_still_propagates() {
    let p = seq!(char('a'), char('b'), char('c'), |a, b, c| format!(
        "{a}{b}{c}"
    ));
    assert_failure!(p, "abx", "expected 'c', but found 'x'", 2);
}
