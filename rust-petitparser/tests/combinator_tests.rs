use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};

#[gtest]
fn seq2_test() {
    let p = seq2(char('a'), char('b'));
    assert_success!(p, "abc", ('a', 'b'), 2);
}

#[gtest]
fn seq2_first_fails() {
    let p = seq2(char('a'), char('b'));
    assert_failure!(p, "xb", "expected 'a', but found 'x'", 0);
}

#[gtest]
fn seq2_second_fails() {
    let p = seq2(char('a'), char('b'));
    assert_failure!(p, "ax", "expected 'b', but found 'x'", 1);
}

#[gtest]
fn seq4_test() {
    let p = seq4(char('a'), char('b'), char('c'), char('d'));
    assert_success!(p, "abcd", ('a', 'b', 'c', 'd'), 4);
}

#[gtest]
fn choice2_test() {
    let p = choice2(char('a'), char('b'));
    assert_success!(p, "ac", 'a', 1);
    assert_success!(p, "bc", 'b', 1);
    assert_failure!(p, "cc", "expected 'b', but found 'c'", 0);
}

#[gtest]
fn choice2_with_joiner_test() {
    let p = Choice2 {
        joiner: SELECT_FARTHEST_JOINED,
        ..choice2(char('a'), char('b'))
    };
    assert_success!(p, "ac", 'a', 1);
    assert_success!(p, "bc", 'b', 1);
    assert_failure!(
        p,
        "cc",
        "expected 'a', but found 'c' OR expected 'b', but found 'c'",
        0
    );
}

#[gtest]
fn choice2_failure_test() {
    let p = choice2(seq2(char('a'), char('b')), seq2(char('c'), char('x')));
    assert_failure!(p, "ax", "expected 'b', but found 'x'", 1);
}

#[gtest]
fn choice3_test() {
    let p = choice3(char('a'), char('b'), char('c'));
    assert_success!(p, "ax", 'a', 1);
    assert_success!(p, "bx", 'b', 1);
    assert_success!(p, "cx", 'c', 1);
    assert_failure!(p, "dx", "expected 'c', but found 'd'", 0);
}

#[gtest]
fn nested_parens_settable_test() {
    let mut expr = SettableParser::<i32>::undefined();
    let inner = seq3(char('('), expr.clone(), char(')')).map(|(_, n, _)| n + 1);
    let leaf = char('x').map(|_| 0);
    expr.set(choice2(inner, leaf));

    assert_eq!(expr.parse("x").unwrap().value, 0);
    assert_eq!(expr.parse("(((x)))").unwrap().value, 3);
    assert!(expr.parse("(x").is_err());
}

#[gtest]
fn and_succeeds_without_advancing() {
    let p = char('a').and();
    assert_success!(p, "abc", 'a', 0);
}

#[gtest]
fn and_fails_when_inner_fails() {
    let p = char('z').and();
    assert!(p.parse("abc").is_err());
}

#[gtest]
fn and_as_lookahead_in_sequence() {
    let p = seq2(char('a'), char('b').and()).map(|(l, _)| l);
    assert_success!(p, "ab", 'a', 1);
}

#[gtest]
fn not_succeeds_when_inner_fails() {
    let p = seq2(char('b').not(), letter());
    let result = p.parse("a").unwrap();
    assert_that!(result.context.position, eq(1));
    assert_that!(result.value.1, eq('a'));

    let result = p.parse("b").unwrap_err();
    assert_that!(result.context.position, eq(0));
}

#[gtest]
fn not_fails_when_inner_succeeds() {
    let p = char('a').not();
    assert_failure!(p, "abc", "Expected failure, got success: 'a'", 0);
}

#[gtest]
fn not_succeeds_without_advancing() {
    let p = seq2(char('z').not(), letter().star()).map(|(_, ls)| ls);
    assert_success!(p, "abc", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "yz", &vec!['y', 'z'], 2);
    assert_failure!(p, "zyx", "Expected failure, got success: 'z'", 0);
}

#[gtest]
fn skip() {
    let p = string("abc").skip(char('['), char(']'));
    assert_success!(p, "[abc]", "abc", 5);
    assert_failure!(p, "abc", "expected '[', but found 'a'", 0);
    assert_failure!(p, "[xyz]", "Expected string: \"abc\"", 1);
    assert_failure!(p, "[abcd", "expected ']', but found 'd'", 4);
}
