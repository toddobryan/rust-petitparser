use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser::{assert_failure, assert_success};

/// The untyped structural result dart's `ExpressionBuilder<Object>` produces via literal
/// list expressions like `[left, value, right]` — `Leaf` for primitives/operators, `List`
/// for anything built by a prefix/postfix/infix/wrapper callback.
#[derive(Debug, Clone, PartialEq)]
enum Node {
    Leaf(String),
    List(Vec<Node>),
}

fn leaf(s: impl std::fmt::Display) -> Node {
    Node::Leaf(s.to_string())
}

fn list(items: Vec<Node>) -> Node {
    Node::List(items)
}

fn build_parser() -> impl Parser<Node> {
    let mut builder: ExpressionBuilder<Node> = ExpressionBuilder::new();
    builder.primitive(
        seq2(digit().plus(), seq2(char('.'), digit().plus()).opt())
            .input_with_message("number expected".to_string())
            .trim()
            .map(leaf),
    );
    {
        let group = builder.group();
        group.wrapper(char('(').trim(), char(')').trim(), |l, v, r| {
            list(vec![leaf(l), v, leaf(r)])
        });
        group.wrapper(string("sqrt(").trim(), char(')').trim(), |l, v, r| {
            list(vec![leaf(l), v, leaf(r)])
        });
    }
    builder
        .group()
        .prefix(char('-').trim(), |op, a| list(vec![leaf(*op), a.clone()]));
    {
        let group = builder.group();
        group.postfix(string("++").trim(), |a, op| list(vec![a.clone(), leaf(op)]));
        group.postfix(string("--").trim(), |a, op| list(vec![a.clone(), leaf(op)]));
    }
    builder.group().right(char('^').trim(), |a, op, b| {
        list(vec![a.clone(), leaf(*op), b.clone()])
    });
    {
        let group = builder.group();
        group.left(char('*').trim(), |a, op, b| {
            list(vec![a.clone(), leaf(*op), b.clone()])
        });
        group.left(char('/').trim(), |a, op, b| {
            list(vec![a.clone(), leaf(*op), b.clone()])
        });
    }
    {
        let group = builder.group();
        group.left(char('+').trim(), |a, op, b| {
            list(vec![a.clone(), leaf(*op), b.clone()])
        });
        group.left(char('-').trim(), |a, op, b| {
            list(vec![a.clone(), leaf(*op), b.clone()])
        });
    }
    builder.build().end()
}

fn build_evaluator() -> impl Parser<f64> {
    let mut builder: ExpressionBuilder<f64> = ExpressionBuilder::new();
    builder.primitive(
        seq2(digit().plus(), seq2(char('.'), digit().plus()).opt())
            .input_with_message("number expected".to_string())
            .trim()
            .map(|s| s.parse::<f64>().unwrap()),
    );
    {
        let group = builder.group();
        group.wrapper(char('(').trim(), char(')').trim(), |_, v, _| v);
        group.wrapper(string("sqrt(").trim(), char(')').trim(), |_, v, _| v.sqrt());
    }
    builder.group().prefix(char('-').trim(), |_, a| -*a);
    {
        let group = builder.group();
        group.postfix(string("++").trim(), |a, _| *a + 1.0);
        group.postfix(string("--").trim(), |a, _| *a - 1.0);
    }
    builder
        .group()
        .right(char('^').trim(), |a, _, b| a.powf(*b));
    {
        let group = builder.group();
        group.left(char('*').trim(), |a, _, b| *a * *b);
        group.left(char('/').trim(), |a, _, b| *a / *b);
    }
    {
        let group = builder.group();
        group.left(char('+').trim(), |a, _, b| *a + *b);
        group.left(char('-').trim(), |a, _, b| *a - *b);
    }
    builder.build().end()
}

const EPSILON: f64 = 1e-5;

fn close_to(actual: f64, expected: f64) {
    assert_that!(
        (actual - expected).abs(),
        le(EPSILON),
        "expected {} to be close to {}",
        actual,
        expected
    );
}

macro_rules! assert_evaluates_to {
    ($evaluator:expr, $input:expr, $expected:expr) => {
        let result = $evaluator.parse($input);
        let success = result.unwrap_or_else(|f| panic!("expected success, got {:?}", f));
        close_to(success.value, $expected);
    };
}

// --- add ---

#[gtest]
fn add_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "1 + 2",
        &list(vec![leaf("1"), leaf("+"), leaf("2")])
    );
    assert_success!(
        parser,
        "1 + 2 + 3",
        &list(vec![
            list(vec![leaf("1"), leaf("+"), leaf("2")]),
            leaf("+"),
            leaf("3"),
        ])
    );
}

#[gtest]
fn add_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1 + 2", 3.0);
    assert_evaluates_to!(evaluator, "2 + 1", 3.0);
    assert_evaluates_to!(evaluator, "1 + 2.3", 3.3);
    assert_evaluates_to!(evaluator, "2.3 + 1", 3.3);
    assert_evaluates_to!(evaluator, "1 + -2", -1.0);
    assert_evaluates_to!(evaluator, "-2 + 1", -1.0);
}

#[gtest]
fn add_evaluator_many() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1", 1.0);
    assert_evaluates_to!(evaluator, "1 + 2", 3.0);
    assert_evaluates_to!(evaluator, "1 + 2 + 3", 6.0);
    assert_evaluates_to!(evaluator, "1 + 2 + 3 + 4", 10.0);
    assert_evaluates_to!(evaluator, "1 + 2 + 3 + 4 + 5", 15.0);
}

#[gtest]
fn add_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "1 +", "Expected end of input", 2);
    assert_failure!(evaluator, "1 + 2 +", "Expected end of input", 6);
}

// --- sub ---

#[gtest]
fn sub_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "1 - 2",
        &list(vec![leaf("1"), leaf("-"), leaf("2")])
    );
    assert_success!(
        parser,
        "1 - 2 - 3",
        &list(vec![
            list(vec![leaf("1"), leaf("-"), leaf("2")]),
            leaf("-"),
            leaf("3"),
        ])
    );
}

#[gtest]
fn sub_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1 - 2", -1.0);
    assert_evaluates_to!(evaluator, "1.2 - 1.2", 0.0);
    assert_evaluates_to!(evaluator, "1 - -2", 3.0);
    assert_evaluates_to!(evaluator, "-1 - -2", 1.0);
}

#[gtest]
fn sub_evaluator_many() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1", 1.0);
    assert_evaluates_to!(evaluator, "1 - 2", -1.0);
    assert_evaluates_to!(evaluator, "1 - 2 - 3", -4.0);
    assert_evaluates_to!(evaluator, "1 - 2 - 3 - 4", -8.0);
    assert_evaluates_to!(evaluator, "1 - 2 - 3 - 4 - 5", -13.0);
}

#[gtest]
fn sub_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "1 -", "Expected end of input", 2);
    assert_failure!(evaluator, "1 - 2 -", "Expected end of input", 6);
}

// --- mul ---

#[gtest]
fn mul_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "1 * 2",
        &list(vec![leaf("1"), leaf("*"), leaf("2")])
    );
    assert_success!(
        parser,
        "1 * 2 * 3",
        &list(vec![
            list(vec![leaf("1"), leaf("*"), leaf("2")]),
            leaf("*"),
            leaf("3"),
        ])
    );
}

#[gtest]
fn mul_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "2 * 3", 6.0);
    assert_evaluates_to!(evaluator, "2 * -4", -8.0);
}

#[gtest]
fn mul_evaluator_many() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1 * 2", 2.0);
    assert_evaluates_to!(evaluator, "1 * 2 * 3", 6.0);
    assert_evaluates_to!(evaluator, "1 * 2 * 3 * 4", 24.0);
    assert_evaluates_to!(evaluator, "1 * 2 * 3 * 4 * 5", 120.0);
}

#[gtest]
fn mul_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "1 *", "Expected end of input", 2);
    assert_failure!(evaluator, "1 * 2 *", "Expected end of input", 6);
}

// --- div ---

#[gtest]
fn div_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "1 / 2",
        &list(vec![leaf("1"), leaf("/"), leaf("2")])
    );
    assert_success!(
        parser,
        "1 / 2 / 3",
        &list(vec![
            list(vec![leaf("1"), leaf("/"), leaf("2")]),
            leaf("/"),
            leaf("3"),
        ])
    );
}

#[gtest]
fn div_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "12 / 3", 4.0);
    assert_evaluates_to!(evaluator, "-16 / -4", 4.0);
}

#[gtest]
fn div_evaluator_many() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "100 / 2", 50.0);
    assert_evaluates_to!(evaluator, "100 / 2 / 2", 25.0);
    assert_evaluates_to!(evaluator, "100 / 2 / 2 / 5", 5.0);
    assert_evaluates_to!(evaluator, "100 / 2 / 2 / 5 / 5", 1.0);
}

#[gtest]
fn div_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "1 /", "Expected end of input", 2);
    assert_failure!(evaluator, "1 / 2 /", "Expected end of input", 6);
}

// --- pow ---

#[gtest]
fn pow_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "1 ^ 2",
        &list(vec![leaf("1"), leaf("^"), leaf("2")])
    );
    assert_success!(
        parser,
        "1 ^ 2 ^ 3",
        &list(vec![
            leaf("1"),
            leaf("^"),
            list(vec![leaf("2"), leaf("^"), leaf("3")]),
        ])
    );
}

#[gtest]
fn pow_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "2 ^ 3", 8.0);
    assert_evaluates_to!(evaluator, "-2 ^ 3", -8.0);
    assert_evaluates_to!(evaluator, "-2 ^ -3", -0.125);
}

#[gtest]
fn pow_evaluator_many() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "4 ^ 3", 64.0);
    assert_evaluates_to!(evaluator, "4 ^ 3 ^ 2", 262144.0);
    assert_evaluates_to!(evaluator, "4 ^ 3 ^ 2 ^ 1", 262144.0);
    assert_evaluates_to!(evaluator, "4 ^ 3 ^ 2 ^ 1 ^ 0", 262144.0);
}

#[gtest]
fn pow_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "1 ^", "Expected end of input", 2);
    assert_failure!(evaluator, "1 ^ 2 ^", "Expected end of input", 6);
}

// --- parens ---

#[gtest]
fn parens_parser() {
    let parser = build_parser();
    assert_success!(parser, "(1)", &list(vec![leaf("("), leaf("1"), leaf(")")]));
    assert_success!(
        parser,
        "(1 + 2)",
        &list(vec![
            leaf("("),
            list(vec![leaf("1"), leaf("+"), leaf("2")]),
            leaf(")"),
        ])
    );
    assert_success!(
        parser,
        "((1))",
        &list(vec![
            leaf("("),
            list(vec![leaf("("), leaf("1"), leaf(")")]),
            leaf(")"),
        ])
    );
    assert_success!(
        parser,
        "((1 + 2))",
        &list(vec![
            leaf("("),
            list(vec![
                leaf("("),
                list(vec![leaf("1"), leaf("+"), leaf("2")]),
                leaf(")"),
            ]),
            leaf(")"),
        ])
    );
    assert_success!(
        parser,
        "2 * (3 + 4)",
        &list(vec![
            leaf("2"),
            leaf("*"),
            list(vec![
                leaf("("),
                list(vec![leaf("3"), leaf("+"), leaf("4")]),
                leaf(")"),
            ]),
        ])
    );
    assert_success!(
        parser,
        "(2 + 3) * 4",
        &list(vec![
            list(vec![
                leaf("("),
                list(vec![leaf("2"), leaf("+"), leaf("3")]),
                leaf(")"),
            ]),
            leaf("*"),
            leaf("4"),
        ])
    );
    assert_success!(
        parser,
        "6 / (2 + 4)",
        &list(vec![
            leaf("6"),
            leaf("/"),
            list(vec![
                leaf("("),
                list(vec![leaf("2"), leaf("+"), leaf("4")]),
                leaf(")"),
            ]),
        ])
    );
    assert_success!(
        parser,
        "(2 + 6) / 2",
        &list(vec![
            list(vec![
                leaf("("),
                list(vec![leaf("2"), leaf("+"), leaf("6")]),
                leaf(")"),
            ]),
            leaf("/"),
            leaf("2"),
        ])
    );
}

#[gtest]
fn parens_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "(1)", 1.0);
    assert_evaluates_to!(evaluator, "(1 + 2)", 3.0);
    assert_evaluates_to!(evaluator, "((1))", 1.0);
    assert_evaluates_to!(evaluator, "((1 + 2))", 3.0);
    assert_evaluates_to!(evaluator, "2 * (3 + 4)", 14.0);
    assert_evaluates_to!(evaluator, "(2 + 3) * 4", 20.0);
    assert_evaluates_to!(evaluator, "6 / (2 + 4)", 1.0);
    assert_evaluates_to!(evaluator, "(2 + 6) / 2", 4.0);
}

#[gtest]
fn parens_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "(", "number expected", 0);
    assert_failure!(evaluator, "()", "number expected", 0);
    assert_failure!(evaluator, "(1", "number expected", 0);
    assert_failure!(evaluator, "((", "number expected", 0);
    assert_failure!(evaluator, "((2", "number expected", 0);
    assert_failure!(evaluator, "((2)", "number expected", 0);
}

// --- sqrt ---

#[gtest]
fn sqrt_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "sqrt(4)",
        &list(vec![leaf("sqrt("), leaf("4"), leaf(")")])
    );
    assert_success!(
        parser,
        "sqrt(1 + 3)",
        &list(vec![
            leaf("sqrt("),
            list(vec![leaf("1"), leaf("+"), leaf("3")]),
            leaf(")"),
        ])
    );
    assert_success!(
        parser,
        "1 + sqrt(16)",
        &list(vec![
            leaf("1"),
            leaf("+"),
            list(vec![leaf("sqrt("), leaf("16"), leaf(")")]),
        ])
    );
    assert_success!(
        parser,
        "sqrt(sqrt(16))",
        &list(vec![
            leaf("sqrt("),
            list(vec![leaf("sqrt("), leaf("16"), leaf(")")]),
            leaf(")"),
        ])
    );
}

#[gtest]
fn sqrt_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "sqrt(4)", 2.0);
    assert_evaluates_to!(evaluator, "sqrt(1 + 3)", 2.0);
    assert_evaluates_to!(evaluator, "1 + sqrt(16)", 5.0);
    assert_evaluates_to!(evaluator, "sqrt(sqrt(16))", 2.0);
}

#[gtest]
fn sqrt_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "sqrt(", "number expected", 0);
    assert_failure!(evaluator, "sqrt()", "number expected", 0);
    assert_failure!(evaluator, "sqrt(1", "number expected", 0);
    assert_failure!(evaluator, "sqrt(sqrt(", "number expected", 0);
    assert_failure!(evaluator, "sqrt(sqrt(1", "number expected", 0);
    assert_failure!(evaluator, "sqrt(sqrt(1)", "number expected", 0);
}

// --- postfix add ---

#[gtest]
fn postfix_add_parser() {
    let parser = build_parser();
    assert_success!(parser, "0++", &list(vec![leaf("0"), leaf("++")]));
    assert_success!(
        parser,
        "0++++",
        &list(vec![list(vec![leaf("0"), leaf("++")]), leaf("++")])
    );
    assert_success!(
        parser,
        "0++++++",
        &list(vec![
            list(vec![list(vec![leaf("0"), leaf("++")]), leaf("++")]),
            leaf("++"),
        ])
    );
    assert_success!(
        parser,
        "0+++1",
        &list(vec![
            list(vec![leaf("0"), leaf("++")]),
            leaf("+"),
            leaf("1"),
        ])
    );
    assert_success!(
        parser,
        "0+++++1",
        &list(vec![
            list(vec![list(vec![leaf("0"), leaf("++")]), leaf("++")]),
            leaf("+"),
            leaf("1"),
        ])
    );
    assert_success!(
        parser,
        "0+++++++1",
        &list(vec![
            list(vec![
                list(vec![list(vec![leaf("0"), leaf("++")]), leaf("++")]),
                leaf("++"),
            ]),
            leaf("+"),
            leaf("1"),
        ])
    );
}

#[gtest]
fn postfix_add_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "0++", 1.0);
    assert_evaluates_to!(evaluator, "0++++", 2.0);
    assert_evaluates_to!(evaluator, "0++++++", 3.0);
    assert_evaluates_to!(evaluator, "0+++1", 2.0);
    assert_evaluates_to!(evaluator, "0+++++1", 3.0);
    assert_evaluates_to!(evaluator, "0+++++++1", 4.0);
}

#[gtest]
fn postfix_add_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "++", "number expected", 0);
    assert_failure!(evaluator, "0+++", "Expected end of input", 3);
}

// --- postfix sub ---

#[gtest]
fn postfix_sub_parser() {
    let parser = build_parser();
    assert_success!(parser, "0--", &list(vec![leaf("0"), leaf("--")]));
    assert_success!(
        parser,
        "0----",
        &list(vec![list(vec![leaf("0"), leaf("--")]), leaf("--")])
    );
    assert_success!(
        parser,
        "0------",
        &list(vec![
            list(vec![list(vec![leaf("0"), leaf("--")]), leaf("--")]),
            leaf("--"),
        ])
    );
    assert_success!(
        parser,
        "0---1",
        &list(vec![
            list(vec![leaf("0"), leaf("--")]),
            leaf("-"),
            leaf("1"),
        ])
    );
    assert_success!(
        parser,
        "0-----1",
        &list(vec![
            list(vec![list(vec![leaf("0"), leaf("--")]), leaf("--")]),
            leaf("-"),
            leaf("1"),
        ])
    );
    assert_success!(
        parser,
        "0-------1",
        &list(vec![
            list(vec![
                list(vec![list(vec![leaf("0"), leaf("--")]), leaf("--")]),
                leaf("--"),
            ]),
            leaf("-"),
            leaf("1"),
        ])
    );
}

#[gtest]
fn postfix_sub_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1--", 0.0);
    assert_evaluates_to!(evaluator, "2----", 0.0);
    assert_evaluates_to!(evaluator, "3------", 0.0);
    assert_evaluates_to!(evaluator, "2---1", 0.0);
    assert_evaluates_to!(evaluator, "3-----1", 0.0);
    assert_evaluates_to!(evaluator, "4-------1", 0.0);
}

#[gtest]
fn postfix_sub_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "--", "number expected", 2);
    assert_failure!(evaluator, "0---", "Expected end of input", 3);
}

// --- negate ---

#[gtest]
fn negate_parser() {
    let parser = build_parser();
    assert_success!(parser, "1", &leaf("1"));
    assert_success!(parser, "-1", &list(vec![leaf("-"), leaf("1")]));
    assert_success!(
        parser,
        "--1",
        &list(vec![leaf("-"), list(vec![leaf("-"), leaf("1")])])
    );
    assert_success!(
        parser,
        "---1",
        &list(vec![
            leaf("-"),
            list(vec![leaf("-"), list(vec![leaf("-"), leaf("1")])]),
        ])
    );
}

#[gtest]
fn negate_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "1", 1.0);
    assert_evaluates_to!(evaluator, "-1", -1.0);
    assert_evaluates_to!(evaluator, "--1", 1.0);
    assert_evaluates_to!(evaluator, "---1", -1.0);
}

#[gtest]
fn negate_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "-", "number expected", 1);
    assert_failure!(evaluator, "--", "number expected", 2);
    assert_failure!(evaluator, "+2", "number expected", 0);
}

// --- number ---

#[gtest]
fn number_parser() {
    let parser = build_parser();
    assert_success!(parser, "0", &leaf("0"));
    assert_success!(parser, "0.1", &leaf("0.1"));
    assert_success!(parser, "-1", &list(vec![leaf("-"), leaf("1")]));
}

#[gtest]
fn number_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "0", 0.0);
    assert_evaluates_to!(evaluator, "0.0", 0.0);
    assert_evaluates_to!(evaluator, "1", 1.0);
    assert_evaluates_to!(evaluator, "1.2", 1.2);
    assert_evaluates_to!(evaluator, "34", 34.0);
    assert_evaluates_to!(evaluator, "34.7", 34.7);
    assert_evaluates_to!(evaluator, "56.78", 56.78);
}

#[gtest]
fn number_error() {
    let evaluator = build_evaluator();
    assert_failure!(evaluator, "", "number expected", 0);
    assert_failure!(evaluator, "-", "number expected", 1);
    assert_failure!(evaluator, "(", "number expected", 0);
    assert_failure!(evaluator, "0.", "Expected end of input", 1);
}

// --- priority ---

#[gtest]
fn priority_parser() {
    let parser = build_parser();
    assert_success!(
        parser,
        "2 * 3 + 4",
        &list(vec![
            list(vec![leaf("2"), leaf("*"), leaf("3")]),
            leaf("+"),
            leaf("4"),
        ])
    );
    assert_success!(
        parser,
        "2 + 3 * 4",
        &list(vec![
            leaf("2"),
            leaf("+"),
            list(vec![leaf("3"), leaf("*"), leaf("4")]),
        ])
    );
}

#[gtest]
fn priority_evaluator() {
    let evaluator = build_evaluator();
    assert_evaluates_to!(evaluator, "2 * 3 + 4", 10.0);
    assert_evaluates_to!(evaluator, "2 + 3 * 4", 14.0);
    assert_evaluates_to!(evaluator, "6 / 3 + 4", 6.0);
    assert_evaluates_to!(evaluator, "2 + 6 / 2", 5.0);
}

// --- builder ---

#[gtest]
#[should_panic(expected = "At least one primitive parser required")]
fn builder_empty() {
    let builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder.build();
}

#[gtest]
#[should_panic(expected = "At least one primitive parser required")]
fn builder_no_primitive() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder
        .group()
        .wrapper(char('('), char(')'), |_, v: String, _| format!("[{}]", v));
    builder.build();
}

#[gtest]
fn builder_loopback() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    let recursive = seq2(char('a'), builder.loopback.borrow()).input();
    builder.primitive(recursive);
    builder.primitive(char('b').map(|c| c.to_string()));
    let parser = builder.build();
    assert_success!(parser, "b", "b");
    assert_success!(parser, "ab", "ab");
    assert_success!(parser, "aab", "aab");
}

#[gtest]
fn builder_epsilon_primitive() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder.primitive(none_of("()").map(|c| c.to_string()));
    builder.primitive(epsilon_with("*".to_string()));
    builder
        .group()
        .wrapper(char('('), char(')'), |_, v: String, _| format!("[{}]", v));
    let parser = builder.build().end();
    assert_success!(parser, "", "*");
    assert_success!(parser, "a", "a");
    assert_success!(parser, "(a)", "[a]");
    assert_success!(parser, "((a))", "[[a]]");
    assert_success!(parser, "()", "[*]");
    assert_success!(parser, "(())", "[[*]]");
}

#[gtest]
fn builder_epsilon_left() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder.primitive(any().map(|c| c.to_string()));
    builder
        .group()
        .left(epsilon_with(()), |a: &String, _, b: &String| {
            format!("[{}{}]", a, b)
        });
    let parser = builder.build().end();
    assert_failure!(
        parser,
        "",
        "expected any character, but reached end of input",
        0
    );
    assert_success!(parser, "a", "a");
    assert_success!(parser, "ab", "[ab]");
    assert_success!(parser, "abc", "[[ab]c]");
    assert_success!(parser, "abcd", "[[[ab]c]d]");
}

#[gtest]
fn builder_epsilon_right() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder.primitive(any().map(|c| c.to_string()));
    builder
        .group()
        .right(epsilon_with(()), |a: &String, _, b: &String| {
            format!("[{}{}]", a, b)
        });
    let parser = builder.build().end();
    assert_failure!(
        parser,
        "",
        "expected any character, but reached end of input",
        0
    );
    assert_success!(parser, "a", "a");
    assert_success!(parser, "ab", "[ab]");
    assert_success!(parser, "abc", "[a[bc]]");
    assert_success!(parser, "abcd", "[a[b[cd]]]");
}

#[gtest]
fn builder_optional_basic() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder.primitive(digit().map(|c| c.to_string()));
    {
        let group = builder.group();
        group.wrapper(char('('), char(')'), |_, v: String, _| format!("({})", v));
        group.optional("∅".to_string());
    }
    let parser = builder.build().end();
    assert_success!(parser, "", "∅");
    assert_success!(parser, "()", "(∅)");
    assert_success!(parser, "1", "1");
    assert_success!(parser, "(1)", "(1)");
}

#[gtest]
#[should_panic(expected = "At most one optional value expected")]
fn builder_optional_repeated() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    let group = builder.group();
    group.optional("foo".to_string());
    group.optional("bar".to_string());
}

// --- examples ---

#[gtest]
fn examples_regex() {
    let mut builder: ExpressionBuilder<String> = ExpressionBuilder::new();
    builder.primitive(none_of(")").map(|c| c.to_string()));
    {
        let group = builder.group();
        group.wrapper(char('('), char(')'), |_, v: String, _| format!("({})", v));
        group.prefix(char('!'), |_, v: &String| format!("!({})", v));
        group.postfix(char('?'), |v: &String, _| format!("({})?", v));
        group.left(char('|'), |l: &String, _, r: &String| {
            format!("({}|{})", l, r)
        });
        group.right(char('&'), |l: &String, _, r: &String| {
            format!("({}&{})", l, r)
        });
    }
    {
        let group = builder.group();
        group.left(epsilon_with(()), |a: &String, _, b: &String| {
            format!("[{}{}]", a, b)
        });
        group.optional("∅".to_string());
    }
    let parser = builder.build().end();
    assert_success!(parser, "", "∅");
    assert_success!(parser, "a", "a");
    assert_success!(parser, "ab", "[ab]");
    assert_success!(parser, "abc", "[[ab]c]");
    assert_success!(parser, "a&b", "(a&b)");
    assert_success!(parser, "a&b&c", "(a&(b&c))");
    assert_success!(parser, "a|b", "(a|b)");
    assert_success!(parser, "a|b|c", "((a|b)|c)");
    assert_success!(parser, "a?", "(a)?");
    assert_success!(parser, "a??", "((a)?)?");
    assert_success!(parser, "!a", "!(a)");
    assert_success!(parser, "!!a", "!(!(a))");
    assert_success!(parser, "()", "(∅)");
    assert_success!(parser, "(a)", "(a)");
    assert_success!(parser, "(ab)", "([ab])");
    assert_success!(parser, "(abc)", "([[ab]c])");
}
