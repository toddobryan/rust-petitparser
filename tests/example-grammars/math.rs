use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::collections::HashMap;

/// Ported from dart-petitparser-examples' `lib/src/math/ast.dart`. `Application` carries its
/// own evaluation function directly (`fn(&[f64]) -> f64`) rather than dart's
/// `Function.apply(function, args)` dynamic dispatch — Rust has no runtime-variadic apply, and
/// every application here is fixed at 1 or 2 arguments anyway (see `create_binding`).
#[derive(Debug, Clone)]
pub enum Expr {
    Value(f64),
    Variable(String),
    Application(String, Vec<Expr>, fn(&[f64]) -> f64),
}

impl Expr {
    pub fn eval(&self, variables: &HashMap<String, f64>) -> f64 {
        match self {
            Expr::Value(v) => *v,
            Expr::Variable(name) => *variables
                .get(name)
                .unwrap_or_else(|| panic!("Unknown variable: {}", name)),
            Expr::Application(_, args, f) => {
                let values: Vec<f64> = args.iter().map(|a| a.eval(variables)).collect();
                f(&values)
            }
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Value(v) => write!(f, "Value{{{}}}", v),
            Expr::Variable(name) => write!(f, "Variable{{{}}}", name),
            Expr::Application(name, ..) => write!(f, "Application{{{}}}", name),
        }
    }
}

// --- common.dart ---

fn constant(name: &str) -> Option<f64> {
    match name {
        "e" => Some(std::f64::consts::E),
        "pi" => Some(std::f64::consts::PI),
        _ => None,
    }
}

fn function1(name: &str) -> Option<fn(&[f64]) -> f64> {
    match name {
        "acos" => Some(|a| a[0].acos()),
        "asin" => Some(|a| a[0].asin()),
        "atan" => Some(|a| a[0].atan()),
        "cos" => Some(|a| a[0].cos()),
        "exp" => Some(|a| a[0].exp()),
        "log" => Some(|a| a[0].ln()),
        "sin" => Some(|a| a[0].sin()),
        "sqrt" => Some(|a| a[0].sqrt()),
        "tan" => Some(|a| a[0].tan()),
        "abs" => Some(|a| a[0].abs()),
        "ceil" => Some(|a| a[0].ceil()),
        "floor" => Some(|a| a[0].floor()),
        "round" => Some(|a| a[0].round()),
        "sign" => Some(|a| a[0].signum()),
        "truncate" => Some(|a| a[0].trunc()),
        _ => None,
    }
}

fn function2(name: &str) -> Option<fn(&[f64]) -> f64> {
    match name {
        "atan2" => Some(|a| a[0].atan2(a[1])),
        "max" => Some(|a| a[0].max(a[1])),
        "min" => Some(|a| a[0].min(a[1])),
        "pow" => Some(|a| a[0].powf(a[1])),
        _ => None,
    }
}

fn throw_unknown(name: &str) -> ! {
    panic!("Unknown function: {}", name);
}

fn create_binding(name: String, args: Vec<Expr>) -> Expr {
    match args.len() {
        0 => match constant(&name) {
            Some(v) => Expr::Value(v),
            None => Expr::Variable(name),
        },
        1 => {
            let f = function1(&name).unwrap_or_else(|| throw_unknown(&name));
            Expr::Application(name, args, f)
        }
        2 => {
            let f = function2(&name).unwrap_or_else(|| throw_unknown(&name));
            Expr::Application(name, args, f)
        }
        _ => throw_unknown(&name),
    }
}

// --- parser.dart ---

pub fn math_parser() -> impl Parser<Expr> {
    let mut builder: ExpressionBuilder<Expr> = ExpressionBuilder::new();

    builder.primitive(
        seq3(
            digit().plus(),
            seq2(char('.'), digit().plus()).opt(),
            seq3(pattern("eE"), pattern("+-").opt(), digit().plus()).opt(),
        )
        .input_with_message("number expected".to_string())
        .trim()
        .map(|s: String| Expr::Value(s.parse::<f64>().unwrap())),
    );
    let recursive = builder.loopback.borrow();
    builder.primitive(
        seq2(
            seq2(letter(), word().star())
                .input_with_message("name expected".to_string())
                .trim(),
            seq3(
                char('(').trim(),
                recursive.star_sep(char(',').trim(), Trailing::Disallowed),
                char(')').trim(),
            )
            .map3(|_, args, _| args)
            .opt_with(Vec::new()),
        )
        .map2(create_binding),
    );

    builder
        .group()
        .wrapper(char('(').trim(), char(')').trim(), |_, value, _| value);

    {
        let group = builder.group();
        group.prefix(char('+').trim(), |_, a: &Expr| a.clone());
        group.prefix(char('-').trim(), |_, a: &Expr| {
            Expr::Application("-".to_string(), vec![a.clone()], |args| -args[0])
        });
    }

    builder
        .group()
        .right(char('^').trim(), |a: &Expr, _, b: &Expr| {
            Expr::Application("^".to_string(), vec![a.clone(), b.clone()], |args| {
                args[0].powf(args[1])
            })
        });

    {
        let group = builder.group();
        group.left(char('*').trim(), |a: &Expr, _, b: &Expr| {
            Expr::Application("*".to_string(), vec![a.clone(), b.clone()], |args| {
                args[0] * args[1]
            })
        });
        group.left(char('/').trim(), |a: &Expr, _, b: &Expr| {
            Expr::Application("/".to_string(), vec![a.clone(), b.clone()], |args| {
                args[0] / args[1]
            })
        });
    }

    {
        let group = builder.group();
        group.left(char('+').trim(), |a: &Expr, _, b: &Expr| {
            Expr::Application("+".to_string(), vec![a.clone(), b.clone()], |args| {
                args[0] + args[1]
            })
        });
        group.left(char('-').trim(), |a: &Expr, _, b: &Expr| {
            Expr::Application("-".to_string(), vec![a.clone(), b.clone()], |args| {
                args[0] - args[1]
            })
        });
    }

    builder.build().end()
}

// --- math_test.dart ---

fn verify_with(input: &str, expected: f64, variables: &HashMap<String, f64>) {
    let parser = math_parser();
    let result = parser
        .parse(input)
        .unwrap_or_else(|f| panic!("expected success parsing {:?}, got {:?}", input, f));
    let actual = result.value.eval(variables);
    assert_that!(
        (actual - expected).abs(),
        le(1e-5),
        "evaluating {:?}: expected {} but got {}",
        input,
        expected,
        actual
    );
    assert_that!(result.value.to_string().is_empty(), eq(false));
}

fn verify(input: &str, expected: f64) {
    verify_with(input, expected, &HashMap::new());
}

#[gtest]
#[allow(clippy::approx_constant)]
fn number() {
    verify("0", 0.0);
    verify("42", 42.0);
    verify("3.141", 3.141);
    verify("1.2e5", 1.2e5);
    verify("3.4e-1", 3.4e-1);
}

#[gtest]
fn variable() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 42.0);
    verify_with("x", 42.0, &vars);

    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 1.0);
    vars.insert("y".to_string(), 2.0);
    verify_with("x / y", 0.5, &vars);
}

#[gtest]
#[should_panic(expected = "Unknown variable: x")]
fn variable_unknown_panics() {
    verify("x", f64::NAN);
}

#[gtest]
fn constants() {
    verify("pi", std::f64::consts::PI);
    verify("e", std::f64::consts::E);
}

#[gtest]
fn functions_one_arg() {
    verify("acos(0.5)", 0.5_f64.acos());
    verify("asin(0.5)", 0.5_f64.asin());
    verify("atan(0.5)", 0.5_f64.atan());
    verify("cos(7)", 7.0_f64.cos());
    verify("exp(7)", 7.0_f64.exp());
    verify("log(7)", 7.0_f64.ln());
    verify("sin(7)", 7.0_f64.sin());
    verify("sqrt(2)", 2.0_f64.sqrt());
    verify("tan(7)", 7.0_f64.tan());
    verify("abs(-1)", 1.0);
    verify("ceil(1.2)", 2.0);
    verify("floor(1.2)", 1.0);
    verify("round(1.6)", 2.0);
    verify("sign(-2)", -1.0);
    verify("truncate(-1.2)", -1.0);
}

#[gtest]
fn functions_two_args() {
    verify("atan2(2, 3)", 2.0_f64.atan2(3.0));
    verify("max(2, 3)", 3.0);
    verify("min(2, 3)", 2.0);
    verify("pow(2, 3)", 8.0);
}

#[gtest]
fn prefix() {
    verify("+2", 2.0);
    verify("-pi", -std::f64::consts::PI);
}

#[gtest]
fn power() {
    verify("2 ^ 3", 8.0);
    verify("2 ^ -3", 0.125);
    verify("-2 ^ 3", -8.0);
    verify("-2 ^ -3", -0.125);
}

#[gtest]
fn multiply() {
    verify("2 * 3", 6.0);
    verify("2 * 3 * 4", 24.0);
    verify("6 / 3", 2.0);
    verify("6 / 3 / 2", 1.0);
}

#[gtest]
fn addition() {
    verify("2 + 3", 5.0);
    verify("2 + 3 + 4", 9.0);
    verify("5 - 3", 2.0);
    verify("5 - 3 - 2", 0.0);
}

#[gtest]
fn priority() {
    verify("1 + 2 * 3", 7.0);
    verify("1 + (2 * 3)", 7.0);
    verify("(1 + 2) * 3", 9.0);
}
