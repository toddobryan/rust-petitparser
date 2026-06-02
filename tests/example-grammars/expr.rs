use googletest::prelude::*;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{char, digit, one_of};
use rust_petitparser::parser::combinator::choice::choice2;
use rust_petitparser::parser::combinator::sequence::seq2;
use rust_petitparser::parser::combinator::settable::SettableParser;
use rust_petitparser::parser::ext::ParserExt;
use rust_petitparser::{assert_failure, assert_success};

fn expr_parser() -> impl Parser<f64> {
    let number = digit()
        .plus()
        .input()
        .map(|s: String| s.parse::<f64>().unwrap());
    let mut atom: SettableParser<f64> = SettableParser::undefined();
    let mut add_expr: SettableParser<f64> = SettableParser::undefined();
    let mut mul_expr: SettableParser<f64> = SettableParser::undefined();

    atom.set(choice2(
        number,
        add_expr.clone().skip(char('(').trim(), char(')').trim()),
    ));

    add_expr.set(
        seq2(
            mul_expr.clone(),
            seq2(one_of("+-").trim(), mul_expr.clone()).star(),
        )
        .map(|x| fold_ops(&x)),
    );

    mul_expr.set(
        seq2(atom.clone(), seq2(one_of("*/").trim(), atom.clone()).star()).map(|x| fold_ops(&x)),
    );

    fn fold_ops((first, rest): &(f64, Vec<(char, f64)>)) -> f64 {
        rest.iter().fold(*first, |acc, (op, val)| match op {
            '+' => acc + val,
            '-' => acc - val,
            '*' => acc * val,
            '/' => acc / val,
            _ => unreachable!(),
        })
    }

    add_expr.end()
}

#[gtest]
fn numbers() {
    let p = expr_parser();
    assert_success!(p, "1", 1.0, 1);
    assert_success!(p, "27", 27.0, 2);
    assert_success!(p, "599300", 599300.0, 6);
}

#[gtest]
fn add() {
    let p = expr_parser();
    assert_success!(p, "1 + 2", 3.0, 5);
    assert_success!(p, "1 + 2 + 3", 6.0, 9);
    assert_success!(p, "1 + 2 - 3", 0.0, 9);
}

#[gtest]
fn mul() {
    let p = expr_parser();
    assert_success!(p, "1 * 2", 2.0, 5);
    assert_success!(p, "1 * 2 * 3", 6.0, 9);
    assert_success!(p, "1 * 2 * 3 * 4 / 8", 3.0, 17);
}

#[gtest]
fn groups_and_precedence() {
    let p = expr_parser();
    assert_success!(p, "(1 + 2) * 3", 9.0, 11);
    assert_success!(p, "(1 + 2) * (3 + 4)", 21.0, 17);
    assert_success!(p, "5 + 2 * (3 + 4) * 5 + 6", 81.0, 23);
    assert_success!(p, "2 + 3 * 4", 14.0, 9); // not 20.0
    assert_success!(p, "10 - 2 * 3", 4.0, 10);
    assert_success!(p, "10 - 2 - 3", 5.0, 10);
    assert_success!(p, "8 / 4 / 2", 1.0, 9);
    assert_success!(p, "1 + ((2 * 3))", 7.0, 13);
}

#[gtest]
fn failures() {
    let p = expr_parser();
    assert_failure!(p, "1 + 2 * 3 +", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 * 3 /", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 * 3 (", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 * 3 )", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 3 * 4", "Expected end of input", 5);
}
