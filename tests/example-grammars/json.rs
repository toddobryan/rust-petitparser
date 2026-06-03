use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::prelude::*;
use rust_petitparser_macros::grammar;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

#[grammar]
mod json_grammar {
    use rust_petitparser_macros::grammar;

    fn start() -> impl Parser<Json> {
        json_value().trim().end()
    }

    fn json_value() -> impl Parser<Json> {
        choice6(null(), boolean(), num(), json_string(), array(), object())
    }

    fn object() -> impl Parser<Json> {
        member()
            .star_sep(char(',').trim())
            .skip(char('{').trim(), char('}').trim())
            .map(Json::Object)
    }

    fn array() -> impl Parser<Json> {
        json_value()
            .star_sep(char(',').trim())
            .skip(char('[').trim(), char(']').trim())
            .map(Json::Array)
    }

    fn member() -> impl Parser<(String, Json)> {
        seq3(raw_string().trim(), char(':'), json_value().trim())
            .map(|(key, _, value)| (key, value))
    }

    fn num() -> impl Parser<Json> {
        seq4(
            char('-').opt(),
            choice2(
                char('0').map(|c| c.to_string()),
                seq2(char('0').not(), digit().plus()).input(),
            )
            .opt(),
            fraction().opt(),
            exponent().opt(),
        )
        .input()
        .only_if(|s| !s.is_empty())
        .map(|s| Json::Num(s.parse::<f64>().unwrap()))
    }

    fn fraction() -> impl Parser<()> {
        seq2(char('.'), digit().plus()).to(())
    }

    fn exponent() -> impl Parser<()> {
        seq3(char_ci('e'), one_of("+-").opt(), digit().plus()).to(())
    }

    fn boolean() -> impl Parser<Json> {
        choice2(
            string("true").to(Json::Bool(true)),
            string("false").to(Json::Bool(false)),
        )
    }

    fn null() -> impl Parser<Json> {
        string("null").to(Json::Null)
    }

    fn json_string() -> impl Parser<Json> {
        raw_string().map(Json::Str)
    }

    fn raw_string() -> impl Parser<String> {
        string_char()
            .star()
            .skip(char('"'), char('"'))
            .map(|chars| chars.into_iter().collect())
    }

    fn string_char() -> impl Parser<char> {
        choice2(none_of("\"\\"), escape_sequence())
    }

    fn escape_sequence() -> impl Parser<char> {
        escape_char().skip_left(char('\\'))
    }

    fn escape_char() -> impl Parser<char> {
        choice8(
            char('"').to('"'),
            char('b').to('\x08'),
            char('f').to('\x0C'),
            char('n').to('\n'),
            char('r').to('\r'),
            char('t').to('\t'),
            char('\\').to('\\'),
            unicode_escape(),
        )
    }

    fn unicode_escape() -> impl Parser<char> {
        digit_with_radix(16)
            .times(4)
            .skip_left(char('u'))
            .map(|hex: Vec<char>| {
                char::from_u32(
                    u32::from_str_radix(hex.into_iter().collect::<String>().as_str(), 16).unwrap(),
                )
                .unwrap()
            })
    }
}

#[gtest]
fn json_object_test() {
    let p = JsonGrammar::new();
    let json_str = r#"{"name": "John", "age": 30, "city": "New York"}"#;
    assert_success!(
        p,
        json_str,
        &Json::Object(vec![
            ("name".to_string(), Json::Str("John".to_string())),
            ("age".to_string(), Json::Num(30.0)),
            ("city".to_string(), Json::Str("New York".to_string())),
        ]),
        47,
    );
}

#[gtest]
fn array_test() {
    let p = JsonGrammar::new();
    let json_str = r#"[true, 2, "x", {"array": ["a", false]}, ["b", 23]]"#;
    assert_success!(
        p,
        json_str,
        &Json::Array(vec![
            Json::Bool(true),
            Json::Num(2.0),
            Json::Str("x".to_string()),
            Json::Object(vec![(
                "array".to_string(),
                Json::Array(vec![Json::Str("a".to_string()), Json::Bool(false),])
            ),]),
            Json::Array(vec![Json::Str("b".to_string()), Json::Num(23.0),]),
        ]),
        50,
    );
}
