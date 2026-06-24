use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};
use std::panic;

// star_greedy/plus_greedy/rep_greedy require `limit: impl Parser<()>`; digit() is
// `Parser<char>`, so it needs its value erased to be usable as a limiter here.
fn digit_limit() -> impl Parser<()> {
    digit().map(|_| ())
}

#[gtest]
fn opt_test() {
    let p = char('a').opt();
    assert_success!(p, "abc", Some('a'), 1);
    assert_success!(p, "bc", None, 0);
}

#[gtest]
fn plus_test() {
    let p = char('a').plus();
    assert_success!(p, "abc", &vec!['a'], 1);
    assert_success!(p, "aaabc", &vec!['a', 'a', 'a'], 3);
    assert_failure!(p, "bc", "expected 'a', but found 'b'", 0);
}

#[gtest]
fn plus_with_opt_test() {
    let p = char('x').plus().opt();
    assert_success!(p, "xxxy", &Some(vec!['x', 'x', 'x']), 3);
    assert_success!(p, "y", &None, 0);
}

#[gtest]
fn star_test() {
    let p = char('x').star();
    assert_success!(p, "a", &vec![], 0);
    assert_success!(p, "xxa", &vec!['x', 'x'], 2);
}

#[gtest]
fn star_with_opt_test() {
    let p = char('x').star().opt();
    assert_success!(p, "xxxy", &Some(vec!['x', 'x', 'x']), 3);
}

#[gtest]
fn star_with_opt_that_doesnt_consume_should_panic() {
    let p = char('x').star().opt();
    let orig = panic::take_hook();

    panic::set_hook(Box::new(|_| {}));

    let result = panic::catch_unwind(|| p.parse("y")).expect_err("the function did not panic");

    panic::set_hook(orig);

    assert_that!(
        panic_message(result.as_ref()),
        eq("PossessiveRepeatingParser { delegate: CharParser \
            { kind: Exact('x'), message: None }, min: 0, max: None } \
             must consume input")
    );
}

#[gtest]
fn rep_sep() {
    let p = letter().rep_sep(char(','), 0, None);
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn star_sep_matches_multiple() {
    let p = letter().star_sep(char(','));
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn star_sep_matches_single() {
    let p = letter().star_sep(char(','));
    assert_success!(p, "a", &vec!['a'], 1);
}

#[gtest]
fn star_sep_matches_empty() {
    let p = letter().star_sep(char(','));
    assert_success!(p, "123", &vec![], 0);
}

#[gtest]
fn star_sep_stops_before_trailing_sep() {
    let p = letter().star_sep(char(','));
    assert_success!(p, "a,b,", &vec!['a', 'b'], 3);
}

#[gtest]
fn plus_sep_matches_multiple() {
    let p = letter().plus_sep(char(','));
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn plus_sep_matches_single() {
    let p = letter().plus_sep(char(','));
    assert_success!(p, "a", &vec!['a'], 1);
}

#[gtest]
fn plus_sep_fails_on_empty() {
    let p = letter().plus_sep(char(','));
    assert_failure!(p, "123", "expected letter, but found '1'", 0);
}

#[gtest]
fn plus_sep_stops_before_trailing_sep() {
    let p = letter().plus_sep(char(','));
    assert_success!(p, "a,b,", &vec!['a', 'b'], 3);
}

// lazy

#[gtest]
fn star_lazy() {
    let p = word().star_lazy(digit());
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "a", "expected digit, but reached end of input", 1);
    assert_failure!(p, "ab", "expected digit, but reached end of input", 2);
    assert_success!(p, "1", &vec![], 0);
    assert_success!(p, "a1", &vec!['a'], 1);
    assert_success!(p, "ab1", &vec!['a', 'b'], 2);
    assert_success!(p, "abc1", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "12", &vec![], 0);
    assert_success!(p, "a12", &vec!['a'], 1);
    assert_success!(p, "ab12", &vec!['a', 'b'], 2);
    assert_success!(p, "abc12", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "123", &vec![], 0);
    assert_success!(p, "a123", &vec!['a'], 1);
    assert_success!(p, "ab123", &vec!['a', 'b'], 2);
    assert_success!(p, "abc123", &vec!['a', 'b', 'c'], 3);
}

#[gtest]
fn plus_lazy() {
    let p = word().plus_lazy(digit());
    assert_failure!(
        p,
        "",
        "expected word character (letter, digit, or '_'), but reached end of input",
        0
    );
    assert_failure!(p, "a", "expected digit, but reached end of input", 1);
    assert_failure!(p, "ab", "expected digit, but reached end of input", 2);
    assert_failure!(p, "1", "expected digit, but reached end of input", 1);
    assert_success!(p, "a1", &vec!['a'], 1);
    assert_success!(p, "ab1", &vec!['a', 'b'], 2);
    assert_success!(p, "abc1", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "12", &vec!['1'], 1);
    assert_success!(p, "a12", &vec!['a'], 1);
    assert_success!(p, "ab12", &vec!['a', 'b'], 2);
    assert_success!(p, "abc123", &vec!['a', 'b', 'c'], 3);
}

#[gtest]
fn repeat_lazy() {
    let p = word().rep_lazy(digit(), 2, Some(4));
    assert_failure!(
        p,
        "",
        "expected word character (letter, digit, or '_'), but reached end of input",
        0
    );
    assert_failure!(
        p,
        "a",
        "expected word character (letter, digit, or '_'), but reached end of input",
        1
    );
    assert_failure!(p, "ab", "expected digit, but reached end of input", 2);
    assert_failure!(p, "abc", "expected digit, but reached end of input", 3);
    assert_failure!(p, "abcd", "expected digit, but reached end of input", 4);
    assert_failure!(p, "abcde", "expected digit, but found 'e'", 4);
    assert_failure!(
        p,
        "1",
        "expected word character (letter, digit, or '_'), but reached end of input",
        1
    );
    assert_failure!(p, "a1", "expected digit, but reached end of input", 2);
    assert_success!(p, "ab1", &vec!['a', 'b'], 2);
    assert_success!(p, "abc1", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "abcd1", &vec!['a', 'b', 'c', 'd'], 4);
    assert_failure!(p, "abcde1", "expected digit, but found 'e'", 4);
    assert_failure!(p, "12", "expected digit, but reached end of input", 2);
    assert_success!(p, "a12", &vec!['a', '1'], 2);
    assert_success!(p, "ab12", &vec!['a', 'b'], 2);
    assert_success!(p, "abc12", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "abcd12", &vec!['a', 'b', 'c', 'd'], 4);
    assert_failure!(p, "abcde12", "expected digit, but found 'e'", 4);
    assert_success!(p, "123", &vec!['1', '2'], 2);
    assert_success!(p, "a123", &vec!['a', '1'], 2);
    assert_success!(p, "ab123", &vec!['a', 'b'], 2);
    assert_success!(p, "abc123", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "abcd123", &vec!['a', 'b', 'c', 'd'], 4);
    assert_failure!(p, "abcde123", "expected digit, but found 'e'", 4);
}

#[gtest]
fn repeat_lazy_unbounded() {
    let p = word().rep_lazy(digit(), 2, None);
    let input = format!("{}1111", "a".repeat(100_000));
    assert_success!(p, &input, &vec!['a'; 100_000], 100_000);
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.as_str()
    } else {
        ""
    }
}

#[gtest]
fn star_lazy_with_non_consuming_delegate_should_panic() {
    let p = epsilon().star_lazy(failure());
    let orig = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let result = panic::catch_unwind(|| p.parse("")).expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("Delegate parser EpsilonParser { result: () } must always consume")
    );

    let result = panic::catch_unwind(|| p.fast_parse_on("".chars().collect::<Vec<_>>().into(), 0))
        .expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    panic::set_hook(orig);
}

#[gtest]
fn plus_lazy_with_non_consuming_delegate_should_panic() {
    let p = epsilon().plus_lazy(failure());
    let orig = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let result = panic::catch_unwind(|| p.parse("")).expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    let result = panic::catch_unwind(|| p.fast_parse_on("".chars().collect::<Vec<_>>().into(), 0))
        .expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    panic::set_hook(orig);
}

// possessive: unbounded repeat

#[gtest]
fn possessive_repeat_unbounded() {
    let p = char('a').rep(2, None);
    let input = "a".repeat(100_000);
    assert_success!(p, &input, &vec!['a'; 100_000], 100_000);
}

// greedy

#[gtest]
fn star_greedy() {
    let p = word().star_greedy(digit_limit());
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "a", "expected digit, but found 'a'", 0);
    assert_failure!(p, "ab", "expected digit, but found 'a'", 0);
    assert_success!(p, "1", &vec![], 0);
    assert_success!(p, "a1", &vec!['a'], 1);
    assert_success!(p, "ab1", &vec!['a', 'b'], 2);
    assert_success!(p, "abc1", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "12", &vec!['1'], 1);
    assert_success!(p, "a12", &vec!['a', '1'], 2);
    assert_success!(p, "ab12", &vec!['a', 'b', '1'], 3);
    assert_success!(p, "abc12", &vec!['a', 'b', 'c', '1'], 4);
    assert_success!(p, "123", &vec!['1', '2'], 2);
    assert_success!(p, "a123", &vec!['a', '1', '2'], 3);
    assert_success!(p, "ab123", &vec!['a', 'b', '1', '2'], 4);
    assert_success!(p, "abc123", &vec!['a', 'b', 'c', '1', '2'], 5);
}

#[gtest]
fn plus_greedy() {
    let p = word().plus_greedy(digit_limit());
    assert_failure!(
        p,
        "",
        "expected word character (letter, digit, or '_'), but reached end of input",
        0
    );
    assert_failure!(p, "a", "expected digit, but reached end of input", 1);
    assert_failure!(p, "ab", "expected digit, but found 'b'", 1);
    assert_failure!(p, "1", "expected digit, but reached end of input", 1);
    assert_success!(p, "a1", &vec!['a'], 1);
    assert_success!(p, "ab1", &vec!['a', 'b'], 2);
    assert_success!(p, "abc1", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "12", &vec!['1'], 1);
    assert_success!(p, "a12", &vec!['a', '1'], 2);
    assert_success!(p, "ab12", &vec!['a', 'b', '1'], 3);
    assert_success!(p, "abc12", &vec!['a', 'b', 'c', '1'], 4);
    assert_success!(p, "123", &vec!['1', '2'], 2);
    assert_success!(p, "a123", &vec!['a', '1', '2'], 3);
    assert_success!(p, "ab123", &vec!['a', 'b', '1', '2'], 4);
    assert_success!(p, "abc123", &vec!['a', 'b', 'c', '1', '2'], 5);
}

#[gtest]
fn repeat_greedy() {
    let p = word().rep_greedy(digit_limit(), 2, Some(4));
    assert_failure!(
        p,
        "",
        "expected word character (letter, digit, or '_'), but reached end of input",
        0
    );
    assert_failure!(
        p,
        "a",
        "expected word character (letter, digit, or '_'), but reached end of input",
        1
    );
    assert_failure!(p, "ab", "expected digit, but reached end of input", 2);
    assert_failure!(p, "abc", "expected digit, but found 'c'", 2);
    assert_failure!(p, "abcd", "expected digit, but found 'c'", 2);
    assert_failure!(p, "abcde", "expected digit, but found 'c'", 2);
    assert_failure!(
        p,
        "1",
        "expected word character (letter, digit, or '_'), but reached end of input",
        1
    );
    assert_failure!(p, "a1", "expected digit, but reached end of input", 2);
    assert_success!(p, "ab1", &vec!['a', 'b'], 2);
    assert_success!(p, "abc1", &vec!['a', 'b', 'c'], 3);
    assert_success!(p, "abcd1", &vec!['a', 'b', 'c', 'd'], 4);
    assert_failure!(p, "abcde1", "expected digit, but found 'c'", 2);
    assert_failure!(p, "12", "expected digit, but reached end of input", 2);
    assert_success!(p, "a12", &vec!['a', '1'], 2);
    assert_success!(p, "ab12", &vec!['a', 'b', '1'], 3);
    assert_success!(p, "abc12", &vec!['a', 'b', 'c', '1'], 4);
    assert_success!(p, "abcd12", &vec!['a', 'b', 'c', 'd'], 4);
    assert_failure!(p, "abcde12", "expected digit, but found 'c'", 2);
    assert_success!(p, "123", &vec!['1', '2'], 2);
    assert_success!(p, "a123", &vec!['a', '1', '2'], 3);
    assert_success!(p, "ab123", &vec!['a', 'b', '1', '2'], 4);
    assert_success!(p, "abc123", &vec!['a', 'b', 'c', '1'], 4);
    assert_success!(p, "abcd123", &vec!['a', 'b', 'c', 'd'], 4);
    assert_failure!(p, "abcde123", "expected digit, but found 'c'", 2);
}

#[gtest]
fn repeat_greedy_unbounded() {
    let p = word().rep_greedy(digit_limit(), 2, None);

    let letters = format!("{}1", "a".repeat(100_000));
    assert_success!(p, &letters, &vec!['a'; 100_000], 100_000);

    let digits = format!("{}1", "1".repeat(100_000));
    assert_success!(p, &digits, &vec!['1'; 100_000], 100_000);
}

#[gtest]
fn star_greedy_with_non_consuming_delegate_should_panic() {
    let p = epsilon().star_greedy(failure());
    let orig = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let result = panic::catch_unwind(|| p.parse("")).expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    let result = panic::catch_unwind(|| p.fast_parse_on("".chars().collect::<Vec<_>>().into(), 0))
        .expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    panic::set_hook(orig);
}

#[gtest]
fn plus_greedy_with_non_consuming_delegate_should_panic() {
    let p = epsilon().plus_greedy(failure());
    let orig = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let result = panic::catch_unwind(|| p.parse("")).expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    let result = panic::catch_unwind(|| p.fast_parse_on("".chars().collect::<Vec<_>>().into(), 0))
        .expect_err("the function did not panic");
    assert_that!(
        panic_message(result.as_ref()),
        eq("EpsilonParser { result: () } must always consume")
    );

    panic::set_hook(orig);
}
