use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};
use std::panic;

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

    let msg = if let Some(s) = result.downcast_ref::<&str>() {
        *s
    } else if let Some(s) = result.downcast_ref::<String>() {
        s.as_str()
    } else {
        ""
    };

    assert_that!(
        msg,
        eq(
            "PossessiveRepeatingParser { delegate: CharParser { kind: Exact('x'), message: None }, min: 0, max: None } must consume input"
        )
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
}
/*

expect(parser, isParseSuccess('ab1', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc1', result: ['a', 'b', 'c'], position: 3),
);
expect(
parser,
isParseSuccess('abcd1', result: ['a', 'b', 'c', 'd'], position: 4),
);
expect(
parser,
isParseFailure('abcde1', position: 4, message: 'digit expected'),
);
expect(
parser,
isParseFailure('12', position: 2, message: 'digit expected'),
);
expect(parser, isParseSuccess('a12', result: ['a', '1'], position: 2));
expect(parser, isParseSuccess('ab12', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc12', result: ['a', 'b', 'c'], position: 3),
);
expect(
parser,
isParseSuccess('abcd12', result: ['a', 'b', 'c', 'd'], position: 4),
);
expect(
parser,
isParseFailure('abcde12', position: 4, message: 'digit expected'),
);
expect(parser, isParseSuccess('123', result: ['1', '2'], position: 2));
expect(parser, isParseSuccess('a123', result: ['a', '1'], position: 2));
expect(parser, isParseSuccess('ab123', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc123', result: ['a', 'b', 'c'], position: 3),
);
expect(
parser,
isParseSuccess('abcd123', result: ['a', 'b', 'c', 'd'], position: 4),
);
expect(
parser,
isParseFailure('abcde123', position: 4, message: 'digit expected'),
);
});
test('repeat unbounded', () {
final input = List.filled(100000, 'a');
final parser = word().repeatLazy(digit(), 2, unbounded);
expect(
parser,
isParseSuccess(
'${input.join()}1111',
result: input,
position: input.length,
),
);
});
test('infinite loop', () {
final inner = epsilon(), limiter = failure<void>();
expect(
() => inner.starLazy(limiter).parse(''),
throwsA(
isAssertionError.having(
(exception) => exception.message,
'message',
'$inner must always consume',
),
),
);
expect(
() => inner.starLazy(limiter).fastParseOn('', 0),
throwsA(
isAssertionError.having(
(exception) => exception.message,
'message',
'$inner must always consume',
),
),
);
expect(
() => inner.plusLazy(limiter).parse(''),
throwsA(
isAssertionError.having(
(exception) => exception.message,
'message',
'$inner must always consume',
),
),
);
expect(
() => inner.plusLazy(limiter).fastParseOn('', 0),
throwsA(
isAssertionError.having(
(exception) => exception.message,
'message',
'$inner must always consume',
),
),
);
}, skip: !hasAssertionsEnabled());
});
*/
