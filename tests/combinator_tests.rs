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
    assert_success!(p, "abcd*", ('a', 'b', 'c', 'd'), 4);
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_failure!(p, "*", "expected 'a', but found '*'", 0);
    assert_failure!(p, "a", "expected 'b', but reached end of input", 1);
    assert_failure!(p, "a*", "expected 'b', but found '*'", 1);
    assert_failure!(p, "ab", "expected 'c', but reached end of input", 2);
    assert_failure!(p, "ab*", "expected 'c', but found '*'", 2);
    assert_failure!(p, "abc", "expected 'd', but reached end of input", 3);
    assert_failure!(p, "abc*", "expected 'd', but found '*'", 3);
}

#[gtest]
fn seq9_test() {
    // Representative check that the seqN macro-generated arities hold all the way up to the
    // current max (9), not just the small ones exercised elsewhere.
    let p = seq9(
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
        char('g'),
        char('h'),
        char('i'),
    );
    let record = ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i');
    assert_success!(p, "abcdefghi", record, 9);
    assert_success!(p, "abcdefghi*", record, 9);
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_failure!(p, "abcdefgh", "expected 'i', but reached end of input", 8);
    assert_failure!(p, "abcdefgh*", "expected 'i', but found '*'", 8);
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
    let inner = seq!(char('('), expr.clone(), char(')'), |_, n, _| n + 1);
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
    let p = seq!(char('a'), char('b').and(), |l, _| l);
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
    assert_failure!(p, "abc", "success not expected", 0);
}

#[gtest]
fn not_succeeds_without_advancing() {
    let p = seq!(char('z').not(), letter().star(), |_, ls| ls);
    assert_success!(p, "abc", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "yz", &vec!['y', 'z'], 2);
    assert_failure!(p, "zyx", "success not expected", 0);
}

#[gtest]
fn not_with_message() {
    let p = seq2(
        char('z').not_with_message("expected any character but 'z'".to_string()),
        letter().star(),
    )
    .map2(|_, ls| ls);
    assert_success!(p, "abc", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "yz", &vec!['y', 'z'], 2);
    assert_failure!(p, "zyx", "expected any character but 'z'", 0);
}

#[gtest]
fn skip() {
    let p = string("abc").skip(char('['), char(']'));
    assert_success!(p, "[abc]", "abc", 5);
    assert_failure!(p, "abc", "expected '[', but found 'a'", 0);
    assert_failure!(p, "[xyz]", "Expected string: \"abc\"", 1);
    assert_failure!(p, "[abcd", "expected ']', but found 'd'", 4);
}

// skip: none / before-only / after-only (skip(open, close) with before&after is covered above)

#[gtest]
fn skip_none() {
    // no delimiters at all: skip(epsilon(), epsilon()) just passes the inner value through
    let p = digit().skip(epsilon(), epsilon());
    assert_success!(p, "1", '1', 1);
    assert_success!(p, "2", '2', 1);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
}

#[gtest]
fn skip_before_only() {
    let p = digit().skip_left(char('<'));
    assert_success!(p, "<1", '1', 2);
    assert_success!(p, "<2", '2', 2);
    assert_failure!(p, "", "expected '<', but reached end of input", 0);
    assert_failure!(p, "1", "expected '<', but found '1'", 0);
    assert_failure!(p, "<", "expected digit, but reached end of input", 1);
    assert_failure!(p, "<a", "expected digit, but found 'a'", 1);
}

#[gtest]
fn skip_after_only() {
    let p = digit().skip_right(char('>'));
    assert_success!(p, "1>", '1', 2);
    assert_success!(p, "2>", '2', 2);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "1", "expected '>', but reached end of input", 1);
    assert_failure!(p, "1!", "expected '>', but found '!'", 1);
    assert_failure!(p, ">", "expected digit, but found '>'", 0);
    assert_failure!(p, "a>", "expected digit, but found 'a'", 0);
}

#[gtest]
fn skip_before_and_after() {
    let p = digit().skip(char('<'), char('>'));
    assert_success!(p, "<1>", '1', 3);
    assert_success!(p, "<2>", '2', 3);
    assert_failure!(p, "", "expected '<', but reached end of input", 0);
    assert_failure!(p, "1", "expected '<', but found '1'", 0);
    assert_failure!(p, "1>", "expected '<', but found '1'", 0);
    assert_failure!(p, "<", "expected digit, but reached end of input", 1);
    assert_failure!(p, "<1", "expected '>', but reached end of input", 2);
    assert_failure!(p, "<1!", "expected '>', but found '!'", 2);
}

// settable

#[gtest]
fn settable_passes_through_delegate() {
    let mut s = SettableParser::<char>::undefined();
    s.set(char('a'));
    assert_success!(s, "a", 'a', 1);
    assert_failure!(s, "b", "expected 'a', but found 'b'", 0);
    assert_failure!(s, "", "expected 'a', but reached end of input", 0);
}

#[gtest]
fn settable_undefined_fails_gracefully_until_set() {
    let mut p = SettableParser::<String>::undefined();
    assert_failure!(p, "", "undefined parser", 0);
    assert_failure!(p, "a", "undefined parser", 0);
    p.set(char('a').map(String::from));
    assert_success!(p, "a", "a", 1);
}

#[gtest]
fn settable_undefined_with_message() {
    let p = SettableParser::<char>::undefined_with_message("not wired up yet".to_string());
    assert_failure!(p, "", "not wired up yet", 0);
}

// neg

#[gtest]
fn neg_with_custom_message() {
    let p = digit().neg_with_message("no digit expected".to_string());
    assert_failure!(p, "1", "no digit expected", 0);
    assert_failure!(p, "9", "no digit expected", 0);
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, " ", ' ', 1);
    assert_failure!(p, "", "expected any character, but reached end of input", 0);
}

// seq3 — success and per-position failures

#[gtest]
fn seq3_success() {
    let p = seq3(char('a'), char('b'), char('c'));
    assert_success!(p, "abc", ('a', 'b', 'c'), 3);
    assert_success!(p, "abc*", ('a', 'b', 'c'), 3);
}

#[gtest]
fn seq3_failure_at_0() {
    let p = seq3(char('a'), char('b'), char('c'));
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_failure!(p, "*", "expected 'a', but found '*'", 0);
}

#[gtest]
fn seq3_failure_at_1() {
    let p = seq3(char('a'), char('b'), char('c'));
    assert_failure!(p, "a", "expected 'b', but reached end of input", 1);
    assert_failure!(p, "a*", "expected 'b', but found '*'", 1);
}

#[gtest]
fn seq3_failure_at_2() {
    let p = seq3(char('a'), char('b'), char('c'));
    assert_failure!(p, "ab", "expected 'c', but reached end of input", 2);
    assert_failure!(p, "ab*", "expected 'c', but found '*'", 2);
}

// choice failure-joining matrix: three overlapping-prefix alternatives (ab/12, ac/13,
// ad/14), checked under each of the four FailureJoiner strategies. Positions match
// dart's matrix exactly; message text follows our own "expected X, but found/reached Y"
// convention rather than dart's "X expected" suffix style.

fn ab12() -> impl Parser<String> {
    seq2(one_of("ab").plus(), one_of("12").plus()).input()
}

fn ac13() -> impl Parser<String> {
    seq2(one_of("ac").plus(), one_of("13").plus()).input()
}

fn ad14() -> impl Parser<String> {
    seq2(one_of("ad").plus(), one_of("14").plus()).input()
}

#[gtest]
fn choice_failure_joining_select_first() {
    let p = Choice3 {
        joiner: SELECT_FIRST,
        ..choice3(ab12(), ac13(), ad14())
    };
    assert_success!(p, "ab12", &"ab12".to_string());
    assert_success!(p, "ac13", &"ac13".to_string());
    assert_success!(p, "ad14", &"ad14".to_string());
    assert_failure!(
        p,
        "",
        "expected any of ['a', 'b'], but reached end of input",
        0
    );
    assert_failure!(
        p,
        "a",
        "expected any of ['1', '2'], but reached end of input",
        1
    );
    assert_failure!(
        p,
        "ab",
        "expected any of ['1', '2'], but reached end of input",
        2
    );
    assert_failure!(p, "ac", "expected any of ['1', '2'], but found 'c'", 1);
    assert_failure!(p, "ad", "expected any of ['1', '2'], but found 'd'", 1);
}

#[gtest]
fn choice_failure_joining_select_last() {
    let p = Choice3 {
        joiner: SELECT_SECOND,
        ..choice3(ab12(), ac13(), ad14())
    };
    assert_success!(p, "ab12", &"ab12".to_string());
    assert_success!(p, "ac13", &"ac13".to_string());
    assert_success!(p, "ad14", &"ad14".to_string());
    assert_failure!(
        p,
        "",
        "expected any of ['a', 'd'], but reached end of input",
        0
    );
    assert_failure!(
        p,
        "a",
        "expected any of ['1', '4'], but reached end of input",
        1
    );
    assert_failure!(p, "ab", "expected any of ['1', '4'], but found 'b'", 1);
    assert_failure!(p, "ac", "expected any of ['1', '4'], but found 'c'", 1);
    assert_failure!(
        p,
        "ad",
        "expected any of ['1', '4'], but reached end of input",
        2
    );
}

#[gtest]
fn choice_failure_joining_farthest_failure() {
    // SELECT_FARTHEST is choiceN's default joiner; this just makes that explicit.
    let p = choice3(ab12(), ac13(), ad14());
    assert_success!(p, "ab12", &"ab12".to_string());
    assert_success!(p, "ac13", &"ac13".to_string());
    assert_success!(p, "ad14", &"ad14".to_string());
    assert_failure!(
        p,
        "",
        "expected any of ['a', 'd'], but reached end of input",
        0
    );
    assert_failure!(
        p,
        "a",
        "expected any of ['1', '4'], but reached end of input",
        1
    );
    assert_failure!(
        p,
        "ab",
        "expected any of ['1', '2'], but reached end of input",
        2
    );
    assert_failure!(
        p,
        "ac",
        "expected any of ['1', '3'], but reached end of input",
        2
    );
    assert_failure!(
        p,
        "ad",
        "expected any of ['1', '4'], but reached end of input",
        2
    );
}

#[gtest]
fn choice_failure_joining_farthest_failure_and_joined() {
    let p = Choice3 {
        joiner: SELECT_FARTHEST_JOINED,
        ..choice3(ab12(), ac13(), ad14())
    };
    assert_success!(p, "ab12", &"ab12".to_string());
    assert_success!(p, "ac13", &"ac13".to_string());
    assert_success!(p, "ad14", &"ad14".to_string());
    assert_failure!(
        p,
        "",
        "expected any of ['a', 'b'], but reached end of input OR \
         expected any of ['a', 'c'], but reached end of input OR \
         expected any of ['a', 'd'], but reached end of input",
        0
    );
    assert_failure!(
        p,
        "a",
        "expected any of ['1', '2'], but reached end of input OR \
         expected any of ['1', '3'], but reached end of input OR \
         expected any of ['1', '4'], but reached end of input",
        1
    );
    assert_failure!(
        p,
        "ab",
        "expected any of ['1', '2'], but reached end of input",
        2
    );
    assert_failure!(
        p,
        "ac",
        "expected any of ['1', '3'], but reached end of input",
        2
    );
    assert_failure!(
        p,
        "ad",
        "expected any of ['1', '4'], but reached end of input",
        2
    );
}
