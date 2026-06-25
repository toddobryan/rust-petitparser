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
    let p = letter().rep_sep(char(','), 0, None, Trailing::Disallowed);
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn times_sep_matches_exact_count() {
    let p = letter().times_sep(3, char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn times_sep_fails_when_too_few() {
    let p = letter().times_sep(3, char(','), Trailing::Disallowed);
    assert_failure!(p, "a,b", "expected ',', but reached end of input", 3);
}

#[gtest]
fn times_sep_ignores_extra_elements() {
    let p = letter().times_sep(2, char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,c", &vec!['a', 'b'], 3);
}

#[gtest]
fn star_sep_matches_multiple() {
    let p = letter().star_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn star_sep_matches_single() {
    let p = letter().star_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a", &vec!['a'], 1);
}

#[gtest]
fn star_sep_matches_empty() {
    let p = letter().star_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "123", &vec![], 0);
}

#[gtest]
fn star_sep_stops_before_trailing_sep() {
    let p = letter().star_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,", &vec!['a', 'b'], 3);
}

#[gtest]
fn plus_sep_matches_multiple() {
    let p = letter().plus_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn plus_sep_matches_single() {
    let p = letter().plus_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a", &vec!['a'], 1);
}

#[gtest]
fn plus_sep_fails_on_empty() {
    let p = letter().plus_sep(char(','), Trailing::Disallowed);
    assert_failure!(p, "123", "expected letter, but found '1'", 0);
}

#[gtest]
fn plus_sep_stops_before_trailing_sep() {
    let p = letter().plus_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,", &vec!['a', 'b'], 3);
}

// Trailing::Allowed/Trailing::Required on the flattened (Vec<T>-returning) _sep family.
// Vec<T> doesn't carry separators, so the only observable differences from Trailing::Disallowed
// are whether the trailing separator gets consumed (final position) and whether its absence is
// tolerated or a hard failure -- otherwise this is the same three-way behavior already covered
// in detail for the _with_sep family above.

#[gtest]
fn sep_trailing_allowed_consumes_trailing_separator() {
    let p = letter().star_sep(char(','), Trailing::Allowed);
    assert_success!(p, "a,b,c,", &vec!['a', 'b', 'c'], 6);
}

#[gtest]
fn sep_trailing_disallowed_stops_before_trailing_separator() {
    let p = letter().star_sep(char(','), Trailing::Disallowed);
    assert_success!(p, "a,b,c,", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn sep_trailing_allowed_without_trailing_separator_is_unaffected() {
    let p = letter().star_sep(char(','), Trailing::Allowed);
    assert_success!(p, "a,b,c", &vec!['a', 'b', 'c'], 5);
}

#[gtest]
fn sep_trailing_required_consumes_trailing_separator() {
    let p = letter().plus_sep(char(','), Trailing::Required);
    assert_success!(p, "a,b,c,", &vec!['a', 'b', 'c'], 6);
}

#[gtest]
fn sep_trailing_required_fails_without_trailing_separator() {
    let p = letter().plus_sep(char(','), Trailing::Required);
    assert_failure!(p, "a,b,c", "expected ',', but reached end of input", 5);
}

#[gtest]
fn sep_trailing_required_on_empty_match_succeeds_vacuously() {
    let p = letter().star_sep(char(','), Trailing::Required);
    assert_success!(p, "123", &vec![], 0);
}

#[gtest]
fn sep_trailing_allowed_on_times_at_exact_count() {
    // min == max (times_sep), so the max-loop body never runs at all -- the post-loop
    // trailing probe is what handles this, independent of the loop bodies.
    let p = letter().times_sep(2, char(','), Trailing::Allowed);
    assert_success!(p, "a,b,", &vec!['a', 'b'], 4);
}

#[gtest]
fn sep_trailing_required_on_times_at_exact_count_fails_without_separator() {
    let p = letter().times_sep(2, char(','), Trailing::Required);
    assert_failure!(p, "a,b", "expected ',', but reached end of input", 3);
}

// with_sep (SeparatedList-returning family)
//
// star/plus/times/repeat groups below are ported from dart's `parser_repeater_test.dart`'s
// `separated` group (digit() elements, letter() separator) with Trailing::Disallowed, matching
// dart's current (trailing-less) behavior exactly. Messages use this project's "expected X, but
// found/reached Y" convention rather than dart's flat "X expected".

#[gtest]
fn star_with_sep_test() {
    let p = digit().star_with_sep(letter(), Trailing::Disallowed);
    assert_success!(
        p,
        "",
        &SeparatedList {
            elements: vec![],
            separators: vec![]
        },
        0
    );
    assert_success!(
        p,
        "a",
        &SeparatedList {
            elements: vec![],
            separators: vec![]
        },
        0
    );
    assert_success!(
        p,
        "1",
        &SeparatedList {
            elements: vec!['1'],
            separators: vec![]
        },
        1
    );
    assert_success!(
        p,
        "1a",
        &SeparatedList {
            elements: vec!['1'],
            separators: vec![]
        },
        1
    );
    assert_success!(
        p,
        "1a2",
        &SeparatedList {
            elements: vec!['1', '2'],
            separators: vec!['a']
        },
        3
    );
    assert_success!(
        p,
        "1a2b",
        &SeparatedList {
            elements: vec!['1', '2'],
            separators: vec!['a']
        },
        3
    );
    assert_success!(
        p,
        "1a2b3",
        &SeparatedList {
            elements: vec!['1', '2', '3'],
            separators: vec!['a', 'b']
        },
        5
    );
    assert_success!(
        p,
        "1a2b3c4",
        &SeparatedList {
            elements: vec!['1', '2', '3', '4'],
            separators: vec!['a', 'b', 'c']
        },
        7
    );
}

#[gtest]
fn plus_with_sep_test() {
    let p = digit().plus_with_sep(letter(), Trailing::Disallowed);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "a", "expected digit, but found 'a'", 0);
    assert_success!(
        p,
        "1",
        &SeparatedList {
            elements: vec!['1'],
            separators: vec![]
        },
        1
    );
    assert_success!(
        p,
        "1a2",
        &SeparatedList {
            elements: vec!['1', '2'],
            separators: vec!['a']
        },
        3
    );
    assert_success!(
        p,
        "1a2b3c4",
        &SeparatedList {
            elements: vec!['1', '2', '3', '4'],
            separators: vec!['a', 'b', 'c']
        },
        7
    );
}

#[gtest]
fn times_with_sep_test() {
    let p = digit().times_with_sep(3, letter(), Trailing::Disallowed);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "a", "expected digit, but found 'a'", 0);
    assert_failure!(p, "1", "expected letter, but reached end of input", 1);
    assert_failure!(p, "1a", "expected digit, but reached end of input", 2);
    assert_failure!(p, "1a2", "expected letter, but reached end of input", 3);
    assert_failure!(p, "1a2b", "expected digit, but reached end of input", 4);
    assert_success!(
        p,
        "1a2b3",
        &SeparatedList {
            elements: vec!['1', '2', '3'],
            separators: vec!['a', 'b']
        },
        5
    );
    assert_success!(
        p,
        "1a2b3c4d",
        &SeparatedList {
            elements: vec!['1', '2', '3'],
            separators: vec!['a', 'b']
        },
        5
    );
}

#[gtest]
fn rep_with_sep_test() {
    let p = digit().rep_with_sep(letter(), 2, Some(3), Trailing::Disallowed);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "a", "expected digit, but found 'a'", 0);
    assert_failure!(p, "1", "expected letter, but reached end of input", 1);
    assert_failure!(p, "1a", "expected digit, but reached end of input", 2);
    assert_success!(
        p,
        "1a2",
        &SeparatedList {
            elements: vec!['1', '2'],
            separators: vec!['a']
        },
        3
    );
    assert_success!(
        p,
        "1a2b",
        &SeparatedList {
            elements: vec!['1', '2'],
            separators: vec!['a']
        },
        3
    );
    assert_success!(
        p,
        "1a2b3",
        &SeparatedList {
            elements: vec!['1', '2', '3'],
            separators: vec!['a', 'b']
        },
        5
    );
    assert_success!(
        p,
        "1a2b3c4d",
        &SeparatedList {
            elements: vec!['1', '2', '3'],
            separators: vec!['a', 'b']
        },
        5
    );
}

// Trailing::Allowed/Trailing::Required: our own extension beyond dart, which has no trailing-
// separator concept at all. These lock in the three-way enum's behavior, including the
// empty-input + Required edge case (zero elements vacuously satisfies "every element followed
// by a required separator" — must not be confused with "a required separator must be found").

#[gtest]
fn with_sep_trailing_allowed_consumes_trailing_separator() {
    let p = letter().star_with_sep(char(','), Trailing::Allowed);
    assert_success!(
        p,
        "a,b,c,",
        &SeparatedList {
            elements: vec!['a', 'b', 'c'],
            separators: vec![',', ',', ',']
        },
        6
    );
}

#[gtest]
fn with_sep_trailing_disallowed_stops_before_trailing_separator() {
    let p = letter().star_with_sep(char(','), Trailing::Disallowed);
    assert_success!(
        p,
        "a,b,c,",
        &SeparatedList {
            elements: vec!['a', 'b', 'c'],
            separators: vec![',', ',']
        },
        5
    );
}

#[gtest]
fn with_sep_trailing_allowed_without_trailing_separator_is_unaffected() {
    let p = letter().star_with_sep(char(','), Trailing::Allowed);
    assert_success!(
        p,
        "a,b,c",
        &SeparatedList {
            elements: vec!['a', 'b', 'c'],
            separators: vec![',', ',']
        },
        5
    );
}

#[gtest]
fn with_sep_trailing_required_consumes_trailing_separator() {
    let p = letter().star_with_sep(char(','), Trailing::Required);
    assert_success!(
        p,
        "a,b,c,",
        &SeparatedList {
            elements: vec!['a', 'b', 'c'],
            separators: vec![',', ',', ',']
        },
        6
    );
}

#[gtest]
fn with_sep_trailing_required_fails_without_trailing_separator() {
    let p = letter().star_with_sep(char(','), Trailing::Required);
    assert_failure!(p, "a,b,c", "expected ',', but reached end of input", 5);
}

#[gtest]
fn with_sep_trailing_required_on_empty_match_succeeds_vacuously() {
    // Zero elements means there's nothing that needs a trailing separator -- this must not
    // fail just because no separator was found. Regression test for a real bug: an earlier
    // draft attempted the trailing-separator parse unconditionally, so `Required` on empty
    // input hard-failed even though `star`'s empty match should always succeed.
    let p = letter().star_with_sep(char(','), Trailing::Required);
    assert_success!(
        p,
        "123",
        &SeparatedList {
            elements: vec![],
            separators: vec![]
        },
        0
    );
}

#[gtest]
fn with_sep_trailing_allowed_on_times_at_exact_count() {
    // min == max (times_with_sep), so the max-loop body never runs at all -- this is exactly
    // the edge case the post-loop trailing probe exists to cover independently of either loop.
    let p = letter().times_with_sep(2, char(','), Trailing::Allowed);
    assert_success!(
        p,
        "a,b,",
        &SeparatedList {
            elements: vec!['a', 'b'],
            separators: vec![',', ',']
        },
        4
    );
}

#[gtest]
fn with_sep_trailing_required_on_times_at_exact_count_fails_without_separator() {
    let p = letter().times_with_sep(2, char(','), Trailing::Required);
    assert_failure!(p, "a,b", "expected ',', but reached end of input", 3);
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

// string (star_string/plus_string/times_string/rep_string)
//
// Ported from dart's `parser_repeater_test.dart`'s `string` group. Dropped: the `isA<
// RepeatingCharacterParser>()` checks (no reflection/type-introspection here, so there's no way
// to assert "the fast path was taken" short of re-introducing the `Any`-downcast machinery we
// deliberately avoided); the `any (unicode)` case (no `unicode:` flag concept in this port); and
// `repeat erroneous` (dart asserts `min >= 0`/`max >= min`, but `min`/`max` aren't validated
// anywhere else in this codebase's repeaters either — `min: usize` already makes "min >= 0"
// vacuous in Rust, and adding new `max >= min` validation here would be new scope, not a port).
// Failure messages use this project's "expected X, but found/reached Y" convention rather than
// dart's flat "X expected" — same deliberate divergence as everywhere else in this port.

#[gtest]
fn star_string_test() {
    let p = char('a').star_string();
    assert_success!(p, "", "", 0);
    assert_success!(p, "a", "a", 1);
    assert_success!(p, "aa", "aa", 2);
    assert_success!(p, "aaa", "aaa", 3);
}

#[gtest]
fn plus_string_test() {
    let p = char('a').plus_string();
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_success!(p, "a", "a", 1);
    assert_success!(p, "aa", "aa", 2);
    assert_success!(p, "aaa", "aaa", 3);
}

#[gtest]
fn times_string_test() {
    let p = char('a').times_string(2);
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_failure!(p, "a", "expected 'a', but reached end of input", 1);
    assert_success!(p, "aa", "aa", 2);
    assert_success!(p, "aaa", "aa", 2);
}

#[gtest]
fn rep_string_test() {
    let p = char('a').rep_string(2, Some(3));
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_failure!(p, "a", "expected 'a', but reached end of input", 1);
    assert_success!(p, "aa", "aa", 2);
    assert_success!(p, "aaa", "aaa", 3);
    assert_success!(p, "aaaa", "aaa", 3);
}

#[gtest]
fn rep_string_unbounded_test() {
    let input = "a".repeat(100_000);
    let p = char('a').rep_string(2, None);
    assert_success!(p, &input, input.as_str(), 100_000);
}

#[gtest]
fn any_plus_string_test() {
    let p = any().plus_string();
    assert_failure!(p, "", "expected any character, but reached end of input", 0);
    assert_success!(p, "a", "a", 1);
    assert_success!(p, "aa", "aa", 2);
    assert_success!(p, "aaa", "aaa", 3);
}

#[gtest]
fn plus_string_fallback_test() {
    // Anything other than CharParser/PredicateCharParser falls through to
    // CharacterRepeatingParserExt's generic default (.rep(min, max).input()) instead of the
    // inherent fast path — exercised here via .map(), which produces a MapParser, not a
    // CharParser.
    let p = char('a').map(|c| c).plus_string();
    assert_failure!(p, "", "expected 'a', but reached end of input", 0);
    assert_success!(p, "a", "a", 1);
    assert_success!(p, "aa", "aa", 2);
    assert_success!(p, "aaa", "aaa", 3);
}

// SeparatedList<T, Sep>'s own utility methods -- ported from dart's
// `parser_repeater_test.dart`'s "separated list" group (elements/separators/sequence/foldLeft/
// foldRight/toString). Dart's toString bakes the generic type arguments into the output via
// `$runtimeType` (e.g. "SeparatedList<String, String>(...)") -- not replicated here (no clean
// equivalent to `$runtimeType` in Rust), so the ported display test checks our actual
// "SeparatedList(...)" format instead of dart's substring-only check.

#[gtest]
fn separated_list_elements() {
    let empty: SeparatedList<String, String> = SeparatedList {
        elements: vec![],
        separators: vec![],
    };
    let single: SeparatedList<String, String> = SeparatedList {
        elements: vec!["1".to_string()],
        separators: vec![],
    };
    let double = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string()],
        separators: vec!["+".to_string()],
    };
    let triple = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    let quadruple = SeparatedList {
        elements: vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ],
        separators: vec!["+".to_string(), "-".to_string(), "*".to_string()],
    };
    let mixed = SeparatedList {
        elements: vec![1, 2, 3],
        separators: vec!["+".to_string(), "-".to_string()],
    };

    assert_that!(empty.elements, eq(&Vec::<String>::new()));
    assert_that!(single.elements, eq(&vec!["1".to_string()]));
    assert_that!(double.elements, eq(&vec!["1".to_string(), "2".to_string()]));
    assert_that!(
        triple.elements,
        eq(&vec!["1".to_string(), "2".to_string(), "3".to_string()])
    );
    assert_that!(
        quadruple.elements,
        eq(&vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ])
    );
    assert_that!(mixed.elements, eq(&vec![1, 2, 3]));
}

#[gtest]
fn separated_list_separators() {
    let empty: SeparatedList<String, String> = SeparatedList {
        elements: vec![],
        separators: vec![],
    };
    let single: SeparatedList<String, String> = SeparatedList {
        elements: vec!["1".to_string()],
        separators: vec![],
    };
    let double = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string()],
        separators: vec!["+".to_string()],
    };
    let triple = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    let quadruple = SeparatedList {
        elements: vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ],
        separators: vec!["+".to_string(), "-".to_string(), "*".to_string()],
    };
    let mixed = SeparatedList {
        elements: vec![1, 2, 3],
        separators: vec!["+".to_string(), "-".to_string()],
    };

    assert_that!(empty.separators, eq(&Vec::<String>::new()));
    assert_that!(single.separators, eq(&Vec::<String>::new()));
    assert_that!(double.separators, eq(&vec!["+".to_string()]));
    assert_that!(
        triple.separators,
        eq(&vec!["+".to_string(), "-".to_string()])
    );
    assert_that!(
        quadruple.separators,
        eq(&vec!["+".to_string(), "-".to_string(), "*".to_string()])
    );
    assert_that!(
        mixed.separators,
        eq(&vec!["+".to_string(), "-".to_string()])
    );
}

#[gtest]
fn separated_list_sequential() {
    let empty: SeparatedList<String, String> = SeparatedList {
        elements: vec![],
        separators: vec![],
    };
    assert_that!(empty.sequential().collect::<Vec<_>>(), eq(&vec![]));

    let single: SeparatedList<String, String> = SeparatedList {
        elements: vec!["1".to_string()],
        separators: vec![],
    };
    assert_that!(
        single.sequential().collect::<Vec<_>>(),
        eq(&vec![Interleaved::Element(&single.elements[0])])
    );

    let triple = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    assert_that!(
        triple.sequential().collect::<Vec<_>>(),
        eq(&vec![
            Interleaved::Element(&triple.elements[0]),
            Interleaved::Separator(&triple.separators[0]),
            Interleaved::Element(&triple.elements[1]),
            Interleaved::Separator(&triple.separators[1]),
            Interleaved::Element(&triple.elements[2]),
        ])
    );

    let mixed = SeparatedList {
        elements: vec![1, 2, 3],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    assert_that!(
        mixed.sequential().collect::<Vec<_>>(),
        eq(&vec![
            Interleaved::Element(&mixed.elements[0]),
            Interleaved::Separator(&mixed.separators[0]),
            Interleaved::Element(&mixed.elements[1]),
            Interleaved::Separator(&mixed.separators[1]),
            Interleaved::Element(&mixed.elements[2]),
        ])
    );
}

fn paren_combine(first: String, sep: String, second: String) -> String {
    format!("({first}{sep}{second})")
}

#[gtest]
fn separated_list_fold() {
    let single: SeparatedList<String, String> = SeparatedList {
        elements: vec!["1".to_string()],
        separators: vec![],
    };
    assert_that!(single.fold(paren_combine), eq("1"));

    let double = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string()],
        separators: vec!["+".to_string()],
    };
    assert_that!(double.fold(paren_combine), eq("(1+2)"));

    let triple = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    assert_that!(triple.fold(paren_combine), eq("((1+2)-3)"));

    let quadruple = SeparatedList {
        elements: vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ],
        separators: vec!["+".to_string(), "-".to_string(), "*".to_string()],
    };
    assert_that!(quadruple.fold(paren_combine), eq("(((1+2)-3)*4)"));
}

#[gtest]
#[should_panic(expected = "Can't call fold on an empty SeparatedList")]
fn separated_list_fold_on_empty_panics() {
    let empty: SeparatedList<String, String> = SeparatedList {
        elements: vec![],
        separators: vec![],
    };
    let _ = empty.fold(paren_combine);
}

#[gtest]
fn separated_list_rfold() {
    let single: SeparatedList<String, String> = SeparatedList {
        elements: vec!["1".to_string()],
        separators: vec![],
    };
    assert_that!(single.rfold(paren_combine), eq("1"));

    let double = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string()],
        separators: vec!["+".to_string()],
    };
    assert_that!(double.rfold(paren_combine), eq("(1+2)"));

    let triple = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    assert_that!(triple.rfold(paren_combine), eq("(1+(2-3))"));

    let quadruple = SeparatedList {
        elements: vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ],
        separators: vec!["+".to_string(), "-".to_string(), "*".to_string()],
    };
    assert_that!(quadruple.rfold(paren_combine), eq("(1+(2-(3*4)))"));
}

#[gtest]
#[should_panic(expected = "Can't call rfold on an empty SeparatedList")]
fn separated_list_rfold_on_empty_panics() {
    let empty: SeparatedList<String, String> = SeparatedList {
        elements: vec![],
        separators: vec![],
    };
    let _ = empty.rfold(paren_combine);
}

#[gtest]
fn separated_list_display() {
    let empty: SeparatedList<String, String> = SeparatedList {
        elements: vec![],
        separators: vec![],
    };
    assert_that!(empty.to_string(), eq("SeparatedList()"));

    let single: SeparatedList<String, String> = SeparatedList {
        elements: vec!["1".to_string()],
        separators: vec![],
    };
    assert_that!(single.to_string(), eq("SeparatedList(1)"));

    let double = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string()],
        separators: vec!["+".to_string()],
    };
    assert_that!(double.to_string(), eq("SeparatedList(1, +, 2)"));

    let triple = SeparatedList {
        elements: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    assert_that!(triple.to_string(), eq("SeparatedList(1, +, 2, -, 3)"));

    let quadruple = SeparatedList {
        elements: vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ],
        separators: vec!["+".to_string(), "-".to_string(), "*".to_string()],
    };
    assert_that!(
        quadruple.to_string(),
        eq("SeparatedList(1, +, 2, -, 3, *, 4)")
    );

    let mixed = SeparatedList {
        elements: vec![1, 2, 3],
        separators: vec!["+".to_string(), "-".to_string()],
    };
    assert_that!(mixed.to_string(), eq("SeparatedList(1, +, 2, -, 3)"));
}
