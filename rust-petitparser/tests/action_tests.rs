use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};
use std::fmt::Debug;

#[gtest]
fn map_test() {
    let p = char('a').map(|c| String::from(c));
    assert_success!(p, "abc", "a", 1);
    assert_failure!(p, "bc", "expected 'a', but found 'b'", 0);
}

#[gtest]
fn token_test() {
    let p = char('a').plus().token();
    let success = p.parse("aaab").unwrap();
    assert_that!(success.context.position, eq(3));
    assert_that!(
        success.value,
        eq(&Token::new(
            vec!['a', 'a', 'a'],
            success.context.buffer.clone(),
            0,
            3
        ))
    );
}

#[gtest]
fn token_line_and_column() {
    let p = seq2(
        seq2(char('a'), char('b')),
        seq2(char('\n'), char('c').plus().token()),
    )
    .map(|((_, _), (_, t))| t);
    let success = p.parse("ab\nccd").unwrap();
    assert_that!(success.value.line(), eq(2));
    assert_that!(success.value.column(), eq(1));
}

#[gtest]
fn trim_test() {
    let p = string("hello").trim();
    assert_success!(p, "  hello  ", "hello", 9);
}

#[gtest]
fn input_test() {
    let p = letter().plus().input();
    assert_success!(p, "abc", "abc", 3);
    assert_failure!(p, "123", "expected letter, but found '1'", 0);
}

#[gtest]
fn input_with_message_test() {
    let p = letter()
        .plus()
        .input_with_message("expected a string of letters".to_string());
    assert_success!(p, "abc", "abc", 3);
    assert_failure!(p, "123", "expected a string of letters", 0);
}

#[gtest]
fn only_if() {
    let p = digit()
        .plus()
        .input()
        .map(|s| s.parse::<u32>().unwrap())
        .only_if(|n| *n < 100);
    assert_success!(p, "99", 99u32, 2);
    assert_failure!(p, "123", "unexpected \"123\"", 0);
}

#[gtest]
fn only_if_with_message() {
    let p = digit()
        .plus()
        .input()
        .map(|s| s.parse::<u32>().unwrap())
        .only_if_with_message(|n| *n < 100, "expected int less than 100".to_string());
    assert_success!(p, "99", 99u32, 2);
    assert_failure!(p, "123", "expected int less than 100", 0);
}

#[gtest]
fn only_if_with_factory() {
    fn factory<T: Debug>(context: &Context, value: Success<T>) -> ParseResult<T> {
        Err(Failure {
            context: context.clone(),
            message: format!("{:?} is not divisible by 7", value.value),
        })
    }
    let p = digit()
        .plus()
        .input()
        .map(|s| s.parse::<u32>().unwrap())
        .only_if_with_factory(|n| *n % 7 == 0, factory);
    assert_success!(p, "7", 7u32, 1);
    assert_success!(p, "14", 14u32, 2);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "865", "865 is not divisible by 7", 0);
}

#[gtest]
fn flat_map() {
    let p = digit().flat_map::<_, Vec<char>>(|n| letter().times(n.to_digit(10).unwrap() as usize));
    assert_success!(p, "3abc", &vec!['a', 'b', 'c'], 4);
    assert_success!(p, "0", &vec![], 1);
    assert_failure!(p, "3ab*", "expected letter, but found '*'", 3);
    assert_failure!(p, "abc", "expected digit, but found 'a'", 0);
}

#[gtest]
fn flat_map_continues_correctly() {
    let p = seq2(
        digit().flat_map::<_, Vec<char>>(|n| letter().times(n.to_digit(10).unwrap() as usize)),
        char('*'),
    );
    assert_success!(p, "3abc*", &(vec!['a', 'b', 'c'], '*'), 5);
}

#[gtest]
fn flat_map_degenerate_cases() {
    let p = success(42).flat_map::<_, char>(|n| char(('0' as u8 + *n as u8) as char));
    assert_success!(p, "Z", 'Z', 1);

    let p = failure_with_message("oops".to_string()).flat_map(|_| char('x'));
    assert_failure!(p, "x", "oops", 0);
}
