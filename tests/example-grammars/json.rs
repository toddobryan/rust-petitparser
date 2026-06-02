use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::core::parser::Parser;
use rust_petitparser::parser::character::character::{
    char, char_ci, digit, digit_with_radix, none_of, one_of,
};
use rust_petitparser::parser::combinator::choice::{choice2, choice6, choice8};
use rust_petitparser::parser::combinator::sequence::{seq2, seq3, seq4};
use rust_petitparser::parser::combinator::settable::SettableParser;
use rust_petitparser::parser::ext::ParserExt;
use rust_petitparser::parser::predicate::predicate::string;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

fn raw_string_parser() -> impl Parser<String> {
    let unicode_escape = seq2(char('u'), digit_with_radix(16).times(4).input())
        .map(|(_, s)| char::from_u32(u32::from_str_radix(&s, 16).unwrap()).unwrap());

    let escape_char = choice8(
        char('"').map(|_| '"'),
        char('b').map(|_| '\x08'),
        char('f').map(|_| '\x0C'),
        char('n').map(|_| '\n'),
        char('r').map(|_| '\r'),
        char('t').map(|_| '\t'),
        char('\\').map(|_| '\\'),
        unicode_escape,
    );

    let escape_seq = seq2(char('\\'), escape_char).map(|(_, c)| c);
    let string_char = choice2(none_of("\"\\"), escape_seq);

    seq3(char('"'), string_char.star(), char('"')).map(|(_, chars, _)| chars.into_iter().collect())
}

pub fn json_parser() -> impl Parser<Json> {
    let mut json_value: SettableParser<Json> = SettableParser::undefined();

    let null = string("null").map(|_| Json::Null);

    let boolean = choice2(
        string("true").map(|_| Json::Bool(true)),
        string("false").map(|_| Json::Bool(false)),
    );

    let fraction = seq2(char('.'), digit().plus());
    let exponent = seq3(char_ci('e'), one_of("+-").opt(), digit().plus());

    let num = seq4(
        char('-').opt(),
        choice2(
            char('0').map(|c| c.to_string()),
            seq2(char('0').not(), digit().plus()).map(|(_, ds)| ds.into_iter().collect()),
        )
        .opt(),
        fraction.opt(),
        exponent.opt(),
    )
    .input()
    .only_if(|s| !s.is_empty())
    .map(|s| Json::Num(s.parse::<f64>().unwrap()));

    let json_string = raw_string_parser().map(Json::Str);

    let array = seq3(
        char('['),
        json_value.borrow().trim().star_sep(char(',').trim()),
        char(']'),
    )
    .map(|(_, v, _)| Json::Array(v));

    let member = seq3(
        raw_string_parser().trim(),
        char(':').trim(),
        json_value.borrow().trim(),
    )
    .map(|(s, _, v)| (s, v));

    let object = seq3(char('{'), member.star_sep(char(',').trim()), char('}'))
        .map(|(_, ms, _)| Json::Object(ms));

    json_value.set(choice6(null, boolean, num, json_string, array, object).trim());

    json_value
}

#[gtest]
fn json_object_test() {
    let json_str = r#"{"name": "John", "age": 30, "city": "New York"}"#;
    assert_success!(
        json_parser(),
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
    let json_str = r#"[true, 2, "x", {"array": ["a", false]}, ["b", 23]]"#;
    assert_success!(
        json_parser(),
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
