use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};
use rust_petitparser_macros::grammar;

#[grammar]
mod expr_grammar {
    fn start() -> impl Parser<f64> {
        add_expr().end()
    }

    pub fn add_expr() -> impl Parser<f64> {
        seq!(mul_expr(), seq!(one_of("+-").trim(), mul_expr()).star()).map(|x| fold_ops(&x))
    }

    fn mul_expr() -> impl Parser<f64> {
        seq!(atom(), seq!(one_of("*/").trim(), atom()).star()).map(|x| fold_ops(&x))
    }

    fn atom() -> impl Parser<f64> {
        choice!(
            number(),
            add_expr().skip(char('(').trim(), char(')').trim()),
        )
    }

    fn number() -> impl Parser<f64> {
        digit()
            .plus()
            .input()
            .map(|s: String| s.parse::<f64>().unwrap())
    }
}

fn fold_ops((first, rest): &(f64, Vec<(char, f64)>)) -> f64 {
    rest.iter().fold(*first, |acc, (op, val)| match op {
        '+' => acc + val,
        '-' => acc - val,
        '*' => acc * val,
        '/' => acc / val,
        _ => unreachable!(),
    })
}

#[gtest]
fn numbers() {
    let p = ExprGrammar::new();
    assert_success!(p, "1", 1.0, 1);
    assert_success!(p, "27", 27.0, 2);
    assert_success!(p, "599300", 599300.0, 6);
}

#[gtest]
fn add() {
    let p = ExprGrammar::new();
    assert_success!(p, "1 + 2", 3.0, 5);
    assert_success!(p, "1 + 2 + 3", 6.0, 9);
    assert_success!(p, "1 + 2 - 3", 0.0, 9);
}

#[gtest]
fn mul() {
    let p = ExprGrammar::new();
    assert_success!(p, "1 * 2", 2.0, 5);
    assert_success!(p, "1 * 2 * 3", 6.0, 9);
    assert_success!(p, "1 * 2 * 3 * 4 / 8", 3.0, 17);
}

#[gtest]
fn groups_and_precedence() {
    let p = ExprGrammar::new();
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
fn test_that_pub_parser_can_be_called() {
    let p = ExprGrammar::new();
    assert_success!(p.add_expr(), "1 + 2", 3.0, 5);
}

#[gtest]
fn failures() {
    let p = ExprGrammar::new();
    assert_failure!(p, "1 + 2 * 3 +", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 * 3 /", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 * 3 (", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 * 3 )", "Expected end of input", 9);
    assert_failure!(p, "1 + 2 3 * 4", "Expected end of input", 5);
}
