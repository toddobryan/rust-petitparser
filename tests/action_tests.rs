use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[gtest]
fn map_test() {
    let p = char('a').map(String::from);
    assert_success!(p, "abc", "a", 1);
    assert_failure!(p, "bc", "expected 'a', but found 'b'", 0);
}

#[gtest]
fn map_digit_to_value() {
    let p = digit().map(|c| c as u32 - '0' as u32);
    assert_success!(p, "1", 1u32, 1);
    assert_success!(p, "4", 4u32, 1);
    assert_success!(p, "9", 9u32, 1);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "a", "expected digit, but found 'a'", 0);
}

#[gtest]
fn map_fast_parse_on_skips_the_mapping_function() {
    let calls = Rc::new(RefCell::new(0));
    let calls_in_closure = calls.clone();
    let p = digit().map(move |c| {
        *calls_in_closure.borrow_mut() += 1;
        c
    });

    let pos = p.fast_parse_on("1".chars().collect::<Vec<_>>().into(), 0);
    assert_that!(pos, eq(Some(1)));
    assert_that!(*calls.borrow(), eq(0));

    // parse_on still calls it normally.
    assert_success!(p, "1", '1', 1);
    assert_that!(*calls.borrow(), eq(1));
}

#[gtest]
fn elements_at_parser() {
    let p = seq2(digit(), letter())
        .map(|(one, two)| vec![one, two])
        .elements_at(vec![-1, 0]);
    assert_success!(p, "1a", &vec!['a', '1'], 2);
    assert_success!(p, "2b", &vec!['b', '2'], 2);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "1", "expected letter, but reached end of input", 1);
    assert_failure!(p, "12", "expected letter, but found '2'", 1);
}

#[gtest]
fn pick_from_start() {
    let p = seq2(digit(), letter())
        .map(|(one, two)| vec![one, two])
        .pick(1);
    assert_success!(p, "1a", 'a', 2);
    assert_success!(p, "2b", 'b', 2);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "1", "expected letter, but reached end of input", 1);
    assert_failure!(p, "12", "expected letter, but found '2'", 1);
}

#[gtest]
fn pick_from_end() {
    let p = seq2(digit(), letter())
        .map(|(one, two)| vec![one, two])
        .pick(-1);
    assert_success!(p, "1a", 'a', 2);
    assert_success!(p, "2b", 'b', 2);
    assert_failure!(p, "", "expected digit, but reached end of input", 0);
    assert_failure!(p, "1", "expected letter, but reached end of input", 1);
    assert_failure!(p, "12", "expected letter, but found '2'", 1);
}

#[derive(Debug)]
struct CountedClone {
    clones: Rc<RefCell<usize>>,
}

impl Clone for CountedClone {
    fn clone(&self) -> Self {
        *self.clones.borrow_mut() += 1;
        CountedClone {
            clones: self.clones.clone(),
        }
    }
}

#[gtest]
fn constant_fast_parse_on_skips_cloning_the_replacement_value() {
    let clones = Rc::new(RefCell::new(0));
    let value = CountedClone {
        clones: clones.clone(),
    };
    let p = char('a').constant(value);

    let pos = p.fast_parse_on("a".chars().collect::<Vec<_>>().into(), 0);
    assert_that!(pos, eq(Some(1)));
    assert_that!(*clones.borrow(), eq(0));

    // parse_on still clones the value to produce it.
    let _ = p.parse("a").unwrap();
    assert_that!(*clones.borrow(), eq(1));
}

#[gtest]
fn token_fast_parse_on_skips_building_the_token_and_inner_side_effects() {
    // Token wraps a map()'d delegate, proving the skip composes: neither the inner mapping
    // function nor the Token construction runs on the fast path.
    let calls = Rc::new(RefCell::new(0));
    let calls_in_closure = calls.clone();
    let p = digit()
        .map(move |c| {
            *calls_in_closure.borrow_mut() += 1;
            c
        })
        .token();

    let pos = p.fast_parse_on("1".chars().collect::<Vec<_>>().into(), 0);
    assert_that!(pos, eq(Some(1)));
    assert_that!(*calls.borrow(), eq(0));
}

#[gtest]
fn token_test() {
    let p = char('a').plus().token();
    let success = p.parse("aaab").unwrap();
    let tok = Token::new(vec!['a', 'a', 'a'], success.context.buffer.clone(), 0, 3);
    assert_success!(p, "aaab", &tok, 3);
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
fn token_accessors_across_mixed_line_endings() {
    // Same fixture as dart's token test: \n, \r, \r\n, and \n line endings in one buffer,
    // exercising every Token accessor (value/buffer/start/end/length/line/column/input)
    // in lockstep with `any().token().star()`.
    let buffer = "1\r12\r\n123\n1234";
    let p = any().token().star();
    let tokens = p.parse(buffer).unwrap().value;

    let expected_values: Vec<char> = buffer.chars().collect();
    assert_that!(
        &tokens.iter().map(|t| t.value).collect::<Vec<_>>(),
        eq(&expected_values)
    );

    let expected_starts: Vec<usize> = (0..14).collect();
    assert_that!(
        &tokens.iter().map(|t| t.start).collect::<Vec<_>>(),
        eq(&expected_starts)
    );

    let expected_ends: Vec<usize> = (1..15).collect();
    assert_that!(
        &tokens.iter().map(|t| t.end).collect::<Vec<_>>(),
        eq(&expected_ends)
    );

    assert_that!(
        &tokens.iter().map(|t| t.length()).collect::<Vec<_>>(),
        eq(&vec![1usize; 14])
    );

    let expected_lines = vec![1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
    assert_that!(
        &tokens.iter().map(|t| t.line()).collect::<Vec<_>>(),
        eq(&expected_lines)
    );

    let expected_columns = vec![1, 2, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];
    assert_that!(
        &tokens.iter().map(|t| t.column()).collect::<Vec<_>>(),
        eq(&expected_columns)
    );

    let expected_inputs: Vec<Vec<char>> = expected_values.iter().map(|c| vec![*c]).collect();
    assert_that!(
        &tokens
            .iter()
            .map(|t| t.input().to_vec())
            .collect::<Vec<_>>(),
        eq(&expected_inputs)
    );

    for token in &tokens {
        assert_that!(token.buffer.iter().collect::<String>(), eq(buffer));
    }
}

#[gtest]
fn token_join_normal() {
    let buffer = "1\r12\r\n123\n1234";
    let tokens = any().token().star().parse(buffer).unwrap().value;

    let joined = Token::join(tokens);
    let expected_values: Vec<char> = buffer.chars().collect();
    assert_that!(joined.value, eq(&expected_values));
    assert_that!(joined.start, eq(0));
    assert_that!(joined.end, eq(buffer.chars().count()));
}

#[gtest]
fn token_join_reverse_order() {
    let buffer = "1\r12\r\n123\n1234";
    let tokens = any().token().star().parse(buffer).unwrap().value;

    let joined = Token::join(tokens.into_iter().rev());
    let mut expected_values: Vec<char> = buffer.chars().collect();
    expected_values.reverse();
    assert_that!(joined.value, eq(&expected_values));
    assert_that!(joined.start, eq(0));
    assert_that!(joined.end, eq(buffer.chars().count()));
}

#[gtest]
#[should_panic(expected = "Token::join requires at least one token")]
fn token_join_empty() {
    let _ = Token::join(Vec::<Token<char>>::new());
}

#[gtest]
#[should_panic(expected = "Token::join requires all tokens use the same buffer")]
fn token_join_different_buffer() {
    let buffer_a: Rc<[char]> = "12".chars().collect::<Vec<_>>().into();
    let buffer_b: Rc<[char]> = "32".chars().collect::<Vec<_>>().into();
    let tokens = vec![
        Token::new('1', buffer_a, 0, 2),
        Token::new('3', buffer_b, 0, 2),
    ];
    let _ = Token::join(tokens);
}

#[gtest]
fn trim_test() {
    let p = string("hello").trim();
    assert_success!(p, "  hello  ", "hello");
}

#[gtest]
fn trim_with_custom_both_sides() {
    let p = char('a').trim_with(char('*').star(), char('*').star());
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "*a", 'a', 2);
    assert_success!(p, "*a*", 'a', 3);
    assert_success!(p, "**a**", 'a', 5);
    assert_failure!(p, "b", "expected 'a', but found 'b'", 0);
    assert_failure!(p, "*b", "expected 'a', but found 'b'", 1);
    assert_failure!(p, "**b", "expected 'a', but found 'b'", 2);
}

#[gtest]
fn trim_with_distinct_left_and_right() {
    let p = char('a').trim_with(char('*').star(), char('#').star());
    assert_success!(p, "a", 'a', 1);
    assert_success!(p, "*a", 'a', 2);
    assert_success!(p, "a#", 'a', 2);
    assert_success!(p, "*a#", 'a', 3);
    assert_success!(p, "**a##", 'a', 5);
    assert_failure!(p, "b", "expected 'a', but found 'b'", 0);
    assert_failure!(p, "*b", "expected 'a', but found 'b'", 1);
    assert_failure!(p, "**b", "expected 'a', but found 'b'", 2);
}

#[gtest]
fn input_test() {
    let p = letter().plus().input();
    assert_success!(p, "abc", "abc");
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
fn input_nested() {
    // .input() composes: an inner flattened span can feed a separated-repeat whose
    // own match is flattened again.
    let p = digit().star().input().plus_sep(char(',')).input();
    assert_success!(p, "1", "1", 1);
    assert_success!(p, "1,12", "1,12", 4);
    assert_success!(p, "1,12,123", "1,12,123", 8);
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
    let p = success(42).flat_map::<_, char>(|n| char((b'0' + *n as u8) as char));
    assert_success!(p, "Z", 'Z', 1);

    let p = failure_with_message("oops".to_string()).flat_map(|_| char('x'));
    assert_failure!(p, "x", "oops", 0);
}
