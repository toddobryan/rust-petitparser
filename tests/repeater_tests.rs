use std::panic;
use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{char, digit, letter, word};
use rust_petitparser::parser::ext::ParserExt;
use std::rc::Rc;

fn buf(s: &str) -> Rc<[char]> {
    s.chars().collect::<Vec<_>>().into()
}

macro_rules! assert_success {
    ($parser:expr, $input:expr, $value:expr, $pos:expr) => {
        let result = $parser.parse($input);
        if result.is_err() {
            panic!("Expected success, but got {:?}", result.unwrap_err());
        }
        let result = result.unwrap();
        assert_that!(result.value, eq($value));
        assert_that!(result.context.position, eq($pos));

        let pos = $parser.fast_parse_on(buf($input), 0);
        if (pos.is_none()) {
            panic!("Expected position after successful parse, but got None");
        }
        let pos = pos.unwrap();
        assert_that!(pos, eq($pos));
    }
}

macro_rules! assert_failure {
    ($parser:expr, $input:expr, $message:expr, $pos:expr) => {
        let result = $parser.parse($input);
        if result.is_ok() {
            panic!("Expected failure, but got success {:?}", result.unwrap());
        }
        let failure = result.unwrap_err();
        assert_that!(failure.message, eq($message));
        assert_that!(failure.context.position, eq($pos));

        let pos = $parser.fast_parse_on(buf($input), 0);
        assert_that!(pos, eq(None));
    }
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

    let result =
        panic::catch_unwind(|| p.parse("y")).expect_err("the function did not panic");

    panic::set_hook(orig);

    let msg = if let Some(s) = result.downcast_ref::<&str>() {
        *s
    } else if let Some(s) = result.downcast_ref::<String>() {
        s.as_str()
    } else {
        ""
    };

    assert_that!(msg, eq("PossessiveRepeatingParser { delegate: CharParser { kind: Exact('x'), message: None }, min: 0, max: None } must consume input"));
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
}
/*expect(parser, isParseSuccess('12', result: isEmpty, position: 0));
expect(parser, isParseSuccess('a12', result: ['a'], position: 1));
expect(parser, isParseSuccess('ab12', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc12', result: ['a', 'b', 'c'], position: 3),
);
expect(parser, isParseSuccess('123', result: isEmpty, position: 0));
expect(parser, isParseSuccess('a123', result: ['a'], position: 1));
expect(parser, isParseSuccess('ab123', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc123', result: ['a', 'b', 'c'], position: 3),
);
});
test('plus', () {
final parser = word().plusLazy(digit());
expect(parser, isParseFailure(''));
expect(
parser,
isParseFailure('a', position: 1, message: 'digit expected'),
);
expect(
parser,
isParseFailure('ab', position: 2, message: 'digit expected'),
);
expect(
parser,
isParseFailure('1', position: 1, message: 'digit expected'),
);
expect(parser, isParseSuccess('a1', result: ['a'], position: 1));
expect(parser, isParseSuccess('ab1', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc1', result: ['a', 'b', 'c'], position: 3),
);
expect(parser, isParseSuccess('12', result: ['1'], position: 1));
expect(parser, isParseSuccess('a12', result: ['a'], position: 1));
expect(parser, isParseSuccess('ab12', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc12', result: ['a', 'b', 'c'], position: 3),
);
expect(parser, isParseSuccess('123', result: ['1'], position: 1));
expect(parser, isParseSuccess('a123', result: ['a'], position: 1));
expect(parser, isParseSuccess('ab123', result: ['a', 'b'], position: 2));
expect(
parser,
isParseSuccess('abc123', result: ['a', 'b', 'c'], position: 3),
);
});
test('repeat', () {
final parser = word().repeatLazy(digit(), 2, 4);
expect(parser, isParseFailure('', message: 'letter or digit expected'));
expect(
parser,
isParseFailure('a', position: 1, message: 'letter or digit expected'),
);
expect(
parser,
isParseFailure('ab', position: 2, message: 'digit expected'),
);
expect(
parser,
isParseFailure('abc', position: 3, message: 'digit expected'),
);
expect(
parser,
isParseFailure('abcd', position: 4, message: 'digit expected'),
);
expect(
parser,
isParseFailure('abcde', position: 4, message: 'digit expected'),
);
expect(
parser,
isParseFailure('1', position: 1, message: 'letter or digit expected'),
);
expect(
parser,
isParseFailure('a1', position: 2, message: 'digit expected'),
);
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
