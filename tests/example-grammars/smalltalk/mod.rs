// Ported from dart-petitparser-examples' lib/src/smalltalk/{grammar,parser}.dart.
//
// dart keeps a separate "grammar" definition (pure recognizer) and "parser"
// definition (subclass overriding each rule to build an AST). Since every
// rule that builds a real AST node also recognizes exactly the same language
// (the AST-building `.map()` never changes success/failure, only the
// returned value), we build a single grammar here that produces real AST
// values directly — smalltalk_test.dart's "grammar" sub-test (which only
// checks that parsing doesn't throw) is folded into the "parser" sub-test's
// success check in our port, since a second, value-erased copy of ~50 rules
// would carry no additional test signal.
//
// dart's `token(source, message)` dispatches at runtime on whether `source`
// is a `String` or a `Parser` (throwing `ArgumentError` otherwise — ported
// nowhere, since Rust's static typing makes that dynamic check unreachable).
// We split it into two statically-typed helpers instead: `token_str` for
// literal punctuation/keywords (always erased to `()`, mirroring dart.rs's
// helper of the same name), and `token_parser` for sub-parsers whose value
// must survive (identifiers, selectors, numbers, strings) — unlike dart.rs,
// smalltalk's AST needs those values, so `token_parser` here is the
// value-preserving generalization dart.rs never needed.
//
// Numbers: dart's `buildNumber` parses a *string* built by concatenating the
// matched spans (`numberToken().value`) back into `num.parse`/`int.parse`.
// We compute the f64 value directly while parsing instead (no
// build-a-string-then-reparse round trip) — same result, idiomatic Rust.
// `exponent()` faithfully requires a literal `-` (ported as-is from dart's
// `char('-').seq(decimalInteger)`), so a positive exponent like `3e4` is not
// actually reachable through this grammar; untested upstream too, left as-is
// per this project's "port bugs faithfully" convention.

mod ast;

pub use ast::{Literal, Method, Node, Pragma, Sequence};
use ast::{build_assignment, build_cascade, build_message};
use rust_petitparser::prelude::*;
use rust_petitparser_macros::grammar;
use std::fmt::Debug;

fn token_parser<T, S, P>(p: impl Parser<T>, spacer: P) -> impl Parser<T>
where
    T: Debug + 'static,
    S: Debug + 'static,
    P: Parser<S> + Clone,
{
    p.trim_with(spacer.clone().star(), spacer.star())
}

fn token_str<S, P>(literal: &'static str, spacer: P) -> impl Parser<()>
where
    S: Debug + 'static,
    P: Parser<S> + Clone,
{
    token_parser(string(literal), spacer).constant(())
}

#[grammar]
pub mod smalltalk_grammar {
    use super::*;

    pub fn start() -> impl Parser<Method> {
        method().end()
    }

    // Whitespace and comments.

    fn spacer() -> impl Parser<()> {
        choice!(whitespace().constant(()), comment())
    }

    fn comment() -> impl Parser<()> {
        seq!(char('"'), char('"').neg().star(), char('"')).constant(())
    }

    // ANSI-standard number literal.

    fn digits() -> impl Parser<String> {
        digit().plus_string()
    }

    fn decimal_integer() -> impl Parser<f64> {
        digits().map(|s| s.parse::<f64>().unwrap())
    }

    fn radix_specifier() -> impl Parser<u32> {
        digits().map(|s| s.parse::<u32>().unwrap())
    }

    fn radix_digits() -> impl Parser<String> {
        pattern("0-9A-Z").plus_string()
    }

    fn radix_integer() -> impl Parser<f64> {
        seq!(
            radix_specifier(),
            char('r'),
            radix_digits() =>
            |base, _, digits| i64::from_str_radix(&digits, base).unwrap() as f64
        )
    }

    fn integer() -> impl Parser<f64> {
        choice!(radix_integer(), decimal_integer())
    }

    fn exponent_letter() -> impl Parser<char> {
        pattern("edq")
    }

    fn exponent() -> impl Parser<f64> {
        seq!(char('-'), decimal_integer() => |_, d| -d)
    }

    fn mantissa() -> impl Parser<f64> {
        seq!(digits(), char('.'), digits() => |int_part, _, frac_part| {
            format!("{int_part}.{frac_part}").parse::<f64>().unwrap()
        })
    }

    fn float() -> impl Parser<f64> {
        seq!(
            mantissa(),
            seq!(exponent_letter(), exponent()).opt() =>
            |m, exp| match exp {
                Some((_, e)) => m * 10f64.powf(e),
                None => m,
            }
        )
    }

    fn scaled_mantissa() -> impl Parser<f64> {
        choice!(decimal_integer(), mantissa())
    }

    fn fractional_digits() -> impl Parser<f64> {
        decimal_integer()
    }

    fn scaled_decimal() -> impl Parser<f64> {
        seq!(
            scaled_mantissa(),
            char('s'),
            fractional_digits().opt() =>
            |m, _, _frac| m
        )
    }

    fn positive_number() -> impl Parser<f64> {
        choice!(scaled_decimal(), float(), integer())
    }

    fn number() -> impl Parser<f64> {
        seq!(
            char('-').opt(),
            positive_number() =>
            |neg, n| if neg.is_some() { -n } else { n }
        )
    }

    // The original smalltalk grammar.

    fn identifier() -> impl Parser<String> {
        seq!(
            pattern("a-zA-Z_"),
            word().star_string() =>
            |first, rest| format!("{first}{rest}")
        )
    }

    fn keyword() -> impl Parser<String> {
        seq!(identifier(), char(':') => |id, _| format!("{id}:"))
    }

    fn binary() -> impl Parser<String> {
        one_of("!%&*+,-/<=>?@\\|~").plus_string()
    }

    fn unary() -> impl Parser<String> {
        seq!(identifier(), char(':').not() => |id, _| id)
    }

    fn multiword() -> impl Parser<String> {
        keyword().plus().map(|parts| parts.concat())
    }

    fn character() -> impl Parser<String> {
        seq!(char('$'), any() => |_, c| c.to_string())
    }

    fn period() -> impl Parser<()> {
        char('.').constant(())
    }

    fn string_lexical() -> impl Parser<String> {
        seq!(
            char('\''),
            choice!(string("''").map(|_| '\''), none_of("'")).star(),
            char('\'') =>
            |_, chars: Vec<char>, _| chars.into_iter().collect()
        )
    }

    fn symbol() -> impl Parser<String> {
        choice!(unary(), binary(), multiword(), string_lexical())
    }

    fn identifier_token() -> impl Parser<String> {
        token_parser(identifier(), spacer())
    }

    fn unary_token() -> impl Parser<String> {
        token_parser(unary(), spacer())
    }

    fn binary_token() -> impl Parser<String> {
        token_parser(binary(), spacer())
    }

    fn keyword_token() -> impl Parser<String> {
        token_parser(keyword(), spacer())
    }

    fn period_token() -> impl Parser<()> {
        token_parser(period(), spacer())
    }

    fn character_token() -> impl Parser<String> {
        token_parser(character(), spacer())
    }

    fn number_token() -> impl Parser<f64> {
        token_parser(number(), spacer())
    }

    fn string_token() -> impl Parser<String> {
        token_parser(string_lexical(), spacer())
    }

    pub fn number_literal() -> impl Parser<Literal> {
        number_token().map(Literal::Number)
    }

    pub fn character_literal() -> impl Parser<Literal> {
        character_token().map(Literal::Str)
    }

    pub fn string_literal() -> impl Parser<Literal> {
        string_token().map(Literal::Str)
    }

    pub fn true_literal() -> impl Parser<Literal> {
        seq!(string("true"), word().not())
            .trim_with(spacer().star(), spacer().star())
            .map(|_| Literal::Bool(true))
    }

    pub fn false_literal() -> impl Parser<Literal> {
        seq!(string("false"), word().not())
            .trim_with(spacer().star(), spacer().star())
            .map(|_| Literal::Bool(false))
    }

    pub fn nil_literal() -> impl Parser<Literal> {
        seq!(string("nil"), word().not())
            .trim_with(spacer().star(), spacer().star())
            .map(|_| Literal::Nil)
    }

    fn symbol_literal_array() -> impl Parser<Literal> {
        token_parser(symbol(), spacer()).map(Literal::Str)
    }

    pub fn symbol_literal() -> impl Parser<Literal> {
        seq!(
            token_str("#", spacer()).plus(),
            token_parser(symbol(), spacer()) =>
            |_, sym| Literal::Str(sym)
        )
    }

    fn array_item() -> impl Parser<Literal> {
        choice!(
            literal(),
            symbol_literal_array(),
            array_literal_array(),
            byte_literal_array(),
        )
    }

    pub fn array_literal() -> impl Parser<Literal> {
        seq!(
            token_str("#(", spacer()),
            array_item().star(),
            token_str(")", spacer()) =>
            |_, items, _| Literal::Array(items)
        )
    }

    fn array_literal_array() -> impl Parser<Literal> {
        seq!(
            token_str("(", spacer()),
            array_item().star(),
            token_str(")", spacer()) =>
            |_, items, _| Literal::Array(items)
        )
    }

    pub fn byte_literal() -> impl Parser<Literal> {
        seq!(
            token_str("#[", spacer()),
            number_literal().star(),
            token_str("]", spacer()) =>
            |_, items, _| Literal::Array(items)
        )
    }

    fn byte_literal_array() -> impl Parser<Literal> {
        seq!(
            token_str("[", spacer()),
            number_literal().star(),
            token_str("]", spacer()) =>
            |_, items, _| Literal::Array(items)
        )
    }

    fn literal() -> impl Parser<Literal> {
        choice!(
            number_literal(),
            string_literal(),
            character_literal(),
            array_literal(),
            byte_literal(),
            symbol_literal(),
            nil_literal(),
            true_literal(),
            false_literal(),
        )
    }

    pub fn variable() -> impl Parser<Node> {
        identifier_token().map(Node::Variable)
    }

    fn assignment_token() -> impl Parser<()> {
        token_str(":=", spacer())
    }

    fn assignment() -> impl Parser<String> {
        seq!(identifier_token(), assignment_token() => |name, _| name)
    }

    fn unary_message() -> impl Parser<(String, Vec<Node>)> {
        unary_token().map(|sel| (sel, vec![]))
    }

    fn binary_message() -> impl Parser<(String, Vec<Node>)> {
        seq!(binary_token(), unary_expression() => |sel, arg| (
            sel,
            vec![arg]
        ))
    }

    fn keyword_message() -> impl Parser<(String, Vec<Node>)> {
        seq!(keyword_token(), binary_expression())
            .plus()
            .map(|pairs| {
                let selector = pairs.iter().map(|(k, _)| k.clone()).collect::<String>();
                let arguments = pairs.into_iter().map(|(_, a)| a).collect();
                (selector, arguments)
            })
    }

    fn message() -> impl Parser<(String, Vec<Node>)> {
        choice!(keyword_message(), binary_message(), unary_message())
    }

    pub fn primary() -> impl Parser<Node> {
        choice!(
            literal().map(Node::Literal),
            variable(),
            block(),
            parens(),
            array(),
        )
    }

    fn unary_expression() -> impl Parser<Node> {
        seq!(primary(), unary_message().star(), => build_message)
    }

    fn binary_expression() -> impl Parser<Node> {
        seq!(unary_expression(), binary_message().star(), => build_message)
    }

    fn keyword_expression() -> impl Parser<Node> {
        seq!(
            binary_expression(),
            keyword_message().opt() =>
            |receiver, part| match part {
                Some(part) => build_message(receiver, vec![part]),
                None => receiver,
            }
        )
    }

    fn cascade_message() -> impl Parser<(String, Vec<Node>)> {
        seq!(token_str(";", spacer()), message() => |_, part| part)
    }

    fn cascade_expression() -> impl Parser<Node> {
        seq!(keyword_expression(), cascade_message().star(), => build_cascade)
    }

    fn expression_return() -> impl Parser<Node> {
        seq!(token_str("^", spacer()), expression() => |_, value| {
            Node::Return(Box::new(value))
        })
    }

    pub fn expression() -> impl Parser<Node> {
        seq!(assignment().star(), cascade_expression() => |vars, value| {
            build_assignment(value, vars)
        })
    }

    fn parens() -> impl Parser<Node> {
        seq!(
            token_str("(", spacer()),
            expression(),
            token_str(")", spacer()) =>
            |_, e, _| e
        )
    }

    fn statements() -> impl Parser<Vec<Node>> {
        choice!(expression_return(), expression())
            .star_sep(period_token().plus(), Trailing::Allowed)
    }

    fn temporaries() -> impl Parser<Vec<String>> {
        seq!(
            token_str("|", spacer()),
            identifier_token().star(),
            token_str("|", spacer()) =>
            |_, vars, _| vars
        )
        .opt()
        .map(|vars| vars.unwrap_or_default())
    }

    pub fn sequence() -> impl Parser<Sequence> {
        seq!(
            temporaries(),
            period_token().star(),
            statements() =>
            |temporaries, _, statements| Sequence {
                temporaries,
                statements,
            },
        )
    }

    pub fn array() -> impl Parser<Node> {
        seq!(
            token_str("{", spacer()),
            expression().star_sep(period_token().plus(), Trailing::Allowed),
            token_str("}", spacer()) =>
            |_, items, _| Node::Array(items)
        )
    }

    fn block_argument() -> impl Parser<String> {
        seq!(token_str(":", spacer()), identifier_token() => |_, name| name)
    }

    fn block_arguments_with() -> impl Parser<Vec<String>> {
        seq!(
            block_argument().plus(),
            choice!(token_str("|", spacer()), token_str("]", spacer()).and()) =>
            |args, _| args
        )
    }

    fn block_arguments_without() -> impl Parser<Vec<String>> {
        epsilon_with(Vec::<String>::new())
    }

    fn block_arguments() -> impl Parser<Vec<String>> {
        choice!(block_arguments_with(), block_arguments_without())
    }

    fn block_body() -> impl Parser<(Vec<String>, Sequence)> {
        seq!(block_arguments(), sequence() => |args, body| (args, body))
    }

    pub fn block() -> impl Parser<Node> {
        seq!(
            token_str("[", spacer()),
            block_body(),
            token_str("]", spacer()) =>
            |_, (arguments, body), _| Node::Block { arguments, body }
        )
    }

    fn keyword_pragma() -> impl Parser<Pragma> {
        seq!(keyword_token(), array_item()).plus().map(|pairs| {
            let selector = pairs.iter().map(|(k, _)| k.clone()).collect::<String>();
            let arguments = pairs.into_iter().map(|(_, a)| a).collect();
            Pragma::new(selector, arguments)
        })
    }

    fn unary_pragma() -> impl Parser<Pragma> {
        identifier_token().map(|sel| Pragma::new(sel, vec![]))
    }

    fn binary_pragma() -> impl Parser<Pragma> {
        seq!(binary_token(), array_item() => |sel, arg| Pragma::new(
            sel,
            vec![arg]
        ))
    }

    fn pragma_message() -> impl Parser<Pragma> {
        choice!(keyword_pragma(), unary_pragma(), binary_pragma())
    }

    pub fn pragma() -> impl Parser<Pragma> {
        seq!(
            token_str("<", spacer()),
            pragma_message(),
            token_str(">", spacer()) =>
            |_, p, _| p
        )
    }

    fn pragmas() -> impl Parser<Vec<Pragma>> {
        pragma().star()
    }

    fn keyword_method() -> impl Parser<(String, Vec<String>)> {
        seq!(keyword_token(), identifier_token())
            .plus()
            .map(|pairs| {
                let selector = pairs.iter().map(|(k, _)| k.clone()).collect::<String>();
                let arguments = pairs.into_iter().map(|(_, a)| a).collect();
                (selector, arguments)
            })
    }

    fn unary_method() -> impl Parser<(String, Vec<String>)> {
        identifier_token().map(|sel| (sel, vec![]))
    }

    fn binary_method() -> impl Parser<(String, Vec<String>)> {
        seq!(binary_token(), identifier_token() => |sel, arg| (
            sel,
            vec![arg]
        ))
    }

    fn method_declaration() -> impl Parser<(String, Vec<String>)> {
        choice!(keyword_method(), unary_method(), binary_method())
    }

    fn method_sequence() -> impl Parser<(Vec<Pragma>, Vec<String>, Vec<Node>)> {
        seq!(
            period_token().star(),
            pragmas(),
            period_token().star(),
            temporaries(),
            period_token().star(),
            pragmas(),
            period_token().star(),
            statements() =>
            |_, before, _, temporaries, _, after, _, statements| {
                let mut pragmas: Vec<Pragma> = before;
                pragmas.extend(after);
                (pragmas, temporaries, statements)
            }
        )
    }

    pub fn method() -> impl Parser<Method> {
        seq!(method_declaration(), method_sequence() => |(
            selector,
            arguments,
        ),
                                                       (
            pragmas,
            temporaries,
            statements,
        )| {
            Method::new(
                selector,
                arguments,
                pragmas,
                Sequence {
                    temporaries,
                    statements,
                },
            )
        },)
    }
}

// Tests ported from dart-petitparser-examples' test/smalltalk_test.dart.
//
// dart's `verify(name, source, grammarProduction, parserProduction, matcher)`
// runs two sub-tests: "grammar" (just checks parsing doesn't throw) and
// "parser" (checks the built AST against a `Matcher`). Since our single
// grammar always builds the same AST a pure recognizer would merely accept
// or reject, the "grammar" half carries no information the "parser" half's
// success doesn't already imply — folded into one assertion per case. dart's
// `NodeCollector.allNodes(ast)` non-empty check is dropped for the same
// reason noted in ast.rs (trivially true for every parse). dart's `token`
// (dynamic-dispatch argument error) and `linter` (reflection) sub-tests have
// no Rust equivalent, matching the precedent set by other example-grammar
// ports in this repo.
#[cfg(test)]
mod tests {
    use super::ast::selector_type_of;
    use super::*;
    use googletest::prelude::*;
    use std::fmt::Debug;

    fn verify<T: Debug + PartialEq + 'static>(
        production: impl Parser<T>,
        source: &str,
        expected: T,
    ) {
        let result = production.end().parse(source);
        let success =
            result.unwrap_or_else(|e| panic!("expected success for {source:?}, got {e:?}"));
        assert_that!(success.value, eq(&expected));
    }

    fn num(n: f64) -> Node {
        Node::Literal(Literal::Number(n))
    }

    fn var(name: &str) -> Node {
        Node::Variable(name.to_string())
    }

    fn assign(name: &str, value: Node) -> Node {
        Node::Assignment(name.to_string(), Box::new(value))
    }

    fn msg(receiver: Node, selector: &str, arguments: Vec<Node>) -> Node {
        let selector_type = selector_type_of(selector, arguments.len());
        Node::Message {
            receiver: Box::new(receiver),
            selector: selector.to_string(),
            selector_type,
            arguments,
        }
    }

    fn cascade(messages: Vec<Node>) -> Node {
        Node::Cascade(messages)
    }

    fn arr(items: Vec<Node>) -> Node {
        Node::Array(items)
    }

    fn ret(value: Node) -> Node {
        Node::Return(Box::new(value))
    }

    fn names(values: Vec<&str>) -> Vec<String> {
        values.into_iter().map(String::from).collect()
    }

    fn seq(temporaries: Vec<&str>, statements: Vec<Node>) -> Sequence {
        Sequence {
            temporaries: names(temporaries),
            statements,
        }
    }

    fn block(arguments: Vec<&str>, temporaries: Vec<&str>, statements: Vec<Node>) -> Node {
        Node::Block {
            arguments: names(arguments),
            body: seq(temporaries, statements),
        }
    }

    fn pragma(selector: &str, arguments: Vec<Literal>) -> Pragma {
        Pragma::new(selector.to_string(), arguments)
    }

    fn method(
        selector: &str,
        arguments: Vec<&str>,
        pragmas: Vec<Pragma>,
        temporaries: Vec<&str>,
        statements: Vec<Node>,
    ) -> Method {
        Method::new(
            selector.to_string(),
            names(arguments),
            pragmas,
            seq(temporaries, statements),
        )
    }

    #[gtest]
    fn array() {
        let g = SmalltalkGrammar::new();
        verify(g.array(), "{}", arr(vec![]));
        verify(g.array(), "{1}", arr(vec![num(1.0)]));
        verify(g.array(), "{1. 2}", arr(vec![num(1.0), num(2.0)]));
        verify(g.array(), "{1. 2. }", arr(vec![num(1.0), num(2.0)]));
    }

    #[gtest]
    fn assignment() {
        let g = SmalltalkGrammar::new();
        verify(g.expression(), "1", num(1.0));
        verify(g.expression(), "a := 1", assign("a", num(1.0)));
        verify(
            g.expression(),
            "a := b := 1",
            assign("a", assign("b", num(1.0))),
        );
        verify(
            g.expression(),
            "a := (b := c)",
            assign("a", assign("b", var("c"))),
        );
    }

    #[gtest]
    fn comment() {
        let g = SmalltalkGrammar::new();
        let expected = || msg(num(1.0), "+", vec![num(2.0)]);
        verify(g.expression(), "1\"one\"+2", expected());
        verify(g.expression(), "1 \"one\" +2", expected());
        verify(g.expression(), "1\"one\"+\"two\"2", expected());
        verify(g.expression(), "1\"one\"\"two\"+2", expected());
        verify(g.expression(), "1\"one\" \"two\"+2", expected());
    }

    #[gtest]
    fn method_negated() {
        let g = SmalltalkGrammar::new();
        let expected = || {
            method(
                "negated",
                vec![],
                vec![],
                vec![],
                vec![ret(msg(num(0.0), "-", vec![var("self")]))],
            )
        };
        verify(g.method(), "negated ^ 0 - self", expected());
        verify(g.method(), "   negated ^ 0 - self", expected());
        verify(g.method(), " negated ^ 0 - self  ", expected());
    }

    #[gtest]
    fn sequence_and_statements() {
        let g = SmalltalkGrammar::new();
        verify(g.sequence(), "| a | 1", seq(vec!["a"], vec![num(1.0)]));
        verify(
            g.sequence(),
            "| a | ^ 1",
            seq(vec!["a"], vec![ret(num(1.0))]),
        );
        verify(
            g.sequence(),
            "| a | 1. ^ 2",
            seq(vec!["a"], vec![num(1.0), ret(num(2.0))]),
        );
        verify(g.sequence(), "1", seq(vec![], vec![num(1.0)]));
        verify(g.sequence(), "1 . 2", seq(vec![], vec![num(1.0), num(2.0)]));
        verify(
            g.sequence(),
            "1 . 2 . 3",
            seq(vec![], vec![num(1.0), num(2.0), num(3.0)]),
        );
        verify(
            g.sequence(),
            "1 . 2 . 3 .",
            seq(vec![], vec![num(1.0), num(2.0), num(3.0)]),
        );
        verify(
            g.sequence(),
            "1 . . 2",
            seq(vec![], vec![num(1.0), num(2.0)]),
        );
        verify(g.sequence(), "1. 2", seq(vec![], vec![num(1.0), num(2.0)]));
        verify(g.sequence(), ". 1", seq(vec![], vec![num(1.0)]));
        verify(g.sequence(), ".1", seq(vec![], vec![num(1.0)]));
        verify(
            g.sequence(),
            "a := 1. b := 2",
            seq(vec![], vec![assign("a", num(1.0)), assign("b", num(2.0))]),
        );
        verify(g.sequence(), "^ 1", seq(vec![], vec![ret(num(1.0))]));
        verify(
            g.sequence(),
            "1. ^ 2",
            seq(vec![], vec![num(1.0), ret(num(2.0))]),
        );
    }

    #[gtest]
    fn temporaries() {
        let g = SmalltalkGrammar::new();
        verify(g.sequence(), "| a |", seq(vec!["a"], vec![]));
        verify(g.sequence(), "| a b |", seq(vec!["a", "b"], vec![]));
        verify(g.sequence(), "| a b c |", seq(vec!["a", "b", "c"], vec![]));
    }

    #[gtest]
    fn variable() {
        let g = SmalltalkGrammar::new();
        verify(g.primary(), "trueBinding", var("trueBinding"));
        verify(g.primary(), "falseBinding", var("falseBinding"));
        verify(g.primary(), "nilly", var("nilly"));
        verify(g.primary(), "selfish", var("selfish"));
        verify(g.primary(), "superman", var("superman"));
        verify(g.primary(), "super_nanny", var("super_nanny"));
        verify(g.primary(), "_gen_var_123_", var("_gen_var_123_"));
    }

    #[gtest]
    fn arguments_block() {
        let g = SmalltalkGrammar::new();
        verify(g.block(), "[ :a | ]", block(vec!["a"], vec![], vec![]));
        verify(
            g.block(),
            "[ :a :b | ]",
            block(vec!["a", "b"], vec![], vec![]),
        );
        verify(
            g.block(),
            "[ :a :b :c | ]",
            block(vec!["a", "b", "c"], vec![], vec![]),
        );
    }

    #[gtest]
    fn complex_block() {
        let g = SmalltalkGrammar::new();
        let expected = || block(vec!["a"], vec!["b"], vec![var("c")]);
        verify(g.block(), "[ :a | | b | c ]", expected());
        verify(g.block(), "[:a||b|c]", expected());
    }

    #[gtest]
    fn simple_and_statement_block() {
        let g = SmalltalkGrammar::new();
        verify(g.block(), "[ ]", block(vec![], vec![], vec![]));
        verify(g.block(), "[ a ]", block(vec![], vec![], vec![var("a")]));
        verify(g.block(), "[ :a ]", block(vec!["a"], vec![], vec![]));
        verify(g.block(), "[ 1 ]", block(vec![], vec![], vec![num(1.0)]));
        verify(
            g.block(),
            "[ | a | 1 ]",
            block(vec![], vec!["a"], vec![num(1.0)]),
        );
        verify(
            g.block(),
            "[ | a b | 1 ]",
            block(vec![], vec!["a", "b"], vec![num(1.0)]),
        );
    }

    #[gtest]
    fn array_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.array_literal(), "#()", Literal::Array(vec![]));
        verify(
            g.array_literal(),
            "#(1)",
            Literal::Array(vec![Literal::Number(1.0)]),
        );
        verify(
            g.array_literal(),
            "#(1 2)",
            Literal::Array(vec![Literal::Number(1.0), Literal::Number(2.0)]),
        );
        verify(
            g.array_literal(),
            "#(true false nil)",
            Literal::Array(vec![
                Literal::Bool(true),
                Literal::Bool(false),
                Literal::Nil,
            ]),
        );
        verify(
            g.array_literal(),
            "#($a)",
            Literal::Array(vec![Literal::Str("a".to_string())]),
        );
        verify(
            g.array_literal(),
            "#(1.2)",
            Literal::Array(vec![Literal::Number(1.2)]),
        );
        verify(
            g.array_literal(),
            "#(size #at: at:put: #'==')",
            Literal::Array(vec![
                Literal::Str("size".to_string()),
                Literal::Str("at:".to_string()),
                Literal::Str("at:put:".to_string()),
                Literal::Str("==".to_string()),
            ]),
        );
        verify(
            g.array_literal(),
            "#('baz')",
            Literal::Array(vec![Literal::Str("baz".to_string())]),
        );
        verify(
            g.array_literal(),
            "#((1) 2)",
            Literal::Array(vec![
                Literal::Array(vec![Literal::Number(1.0)]),
                Literal::Number(2.0),
            ]),
        );
        verify(
            g.array_literal(),
            "#((1 2) #(1 2 3))",
            Literal::Array(vec![
                Literal::Array(vec![Literal::Number(1.0), Literal::Number(2.0)]),
                Literal::Array(vec![
                    Literal::Number(1.0),
                    Literal::Number(2.0),
                    Literal::Number(3.0),
                ]),
            ]),
        );
        verify(
            g.array_literal(),
            "#([1 2] #[1 2 3])",
            Literal::Array(vec![
                Literal::Array(vec![Literal::Number(1.0), Literal::Number(2.0)]),
                Literal::Array(vec![
                    Literal::Number(1.0),
                    Literal::Number(2.0),
                    Literal::Number(3.0),
                ]),
            ]),
        );
    }

    #[gtest]
    fn byte_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.byte_literal(), "#[]", Literal::Array(vec![]));
        verify(
            g.byte_literal(),
            "#[0]",
            Literal::Array(vec![Literal::Number(0.0)]),
        );
        verify(
            g.byte_literal(),
            "#[255]",
            Literal::Array(vec![Literal::Number(255.0)]),
        );
        verify(
            g.byte_literal(),
            "#[ 1 2 ]",
            Literal::Array(vec![Literal::Number(1.0), Literal::Number(2.0)]),
        );
        verify(
            g.byte_literal(),
            "#[ 2r1010 8r77 16rFF ]",
            Literal::Array(vec![
                Literal::Number(10.0),
                Literal::Number(63.0),
                Literal::Number(255.0),
            ]),
        );
    }

    #[gtest]
    fn character_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.character_literal(), "$a", Literal::Str("a".to_string()));
        verify(g.character_literal(), "$ ", Literal::Str(" ".to_string()));
        verify(g.character_literal(), "$$", Literal::Str("$".to_string()));
    }

    #[gtest]
    fn number_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.number_literal(), "0", Literal::Number(0.0));
        verify(g.number_literal(), "0.1", Literal::Number(0.1));
        verify(g.number_literal(), "123", Literal::Number(123.0));
        verify(g.number_literal(), "123.456", Literal::Number(123.456));
        verify(g.number_literal(), "-0", Literal::Number(0.0));
        verify(g.number_literal(), "-0.1", Literal::Number(-0.1));
        verify(g.number_literal(), "-123", Literal::Number(-123.0));
        verify(g.number_literal(), "-123.456", Literal::Number(-123.456));
        verify(g.number_literal(), "10r10", Literal::Number(10.0));
        verify(g.number_literal(), "8r777", Literal::Number(511.0));
        verify(g.number_literal(), "16rAF", Literal::Number(175.0));
    }

    #[gtest]
    fn special_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.true_literal(), "true", Literal::Bool(true));
        verify(g.false_literal(), "false", Literal::Bool(false));
        verify(g.nil_literal(), "nil", Literal::Nil);
    }

    #[gtest]
    fn string_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.string_literal(), "''", Literal::Str(String::new()));
        verify(g.string_literal(), "'ab'", Literal::Str("ab".to_string()));
        verify(
            g.string_literal(),
            "'ab''cd'",
            Literal::Str("ab'cd".to_string()),
        );
    }

    #[gtest]
    fn symbol_literal() {
        let g = SmalltalkGrammar::new();
        verify(g.symbol_literal(), "#foo", Literal::Str("foo".to_string()));
        verify(g.symbol_literal(), "#+", Literal::Str("+".to_string()));
        verify(
            g.symbol_literal(),
            "#key:",
            Literal::Str("key:".to_string()),
        );
        verify(
            g.symbol_literal(),
            "#key:value:",
            Literal::Str("key:value:".to_string()),
        );
        verify(
            g.symbol_literal(),
            "#'ing-result'",
            Literal::Str("ing-result".to_string()),
        );
        verify(
            g.symbol_literal(),
            "#_gen_binding",
            Literal::Str("_gen_binding".to_string()),
        );
        verify(g.symbol_literal(), "# foo", Literal::Str("foo".to_string()));
        verify(g.symbol_literal(), "##foo", Literal::Str("foo".to_string()));
        verify(
            g.symbol_literal(),
            "## foo",
            Literal::Str("foo".to_string()),
        );
    }

    #[gtest]
    fn binary_expression() {
        let g = SmalltalkGrammar::new();
        verify(g.expression(), "1 + 2", msg(num(1.0), "+", vec![num(2.0)]));
        verify(
            g.expression(),
            "1 + 2 + 3",
            msg(msg(num(1.0), "+", vec![num(2.0)]), "+", vec![num(3.0)]),
        );
        verify(
            g.expression(),
            "1 // 2",
            msg(num(1.0), "//", vec![num(2.0)]),
        );
        verify(
            g.expression(),
            "1 -- 2",
            msg(num(1.0), "--", vec![num(2.0)]),
        );
        verify(
            g.expression(),
            "1 ==> 2",
            msg(num(1.0), "==>", vec![num(2.0)]),
        );
    }

    #[gtest]
    fn binary_method() {
        let g = SmalltalkGrammar::new();
        verify(
            g.method(),
            "+ a",
            method("+", vec!["a"], vec![], vec![], vec![]),
        );
        verify(
            g.method(),
            "+ a | b |",
            method("+", vec!["a"], vec![], vec!["b"], vec![]),
        );
        verify(
            g.method(),
            "+ a b",
            method("+", vec!["a"], vec![], vec![], vec![var("b")]),
        );
        verify(
            g.method(),
            "+ a | b | c",
            method("+", vec!["a"], vec![], vec!["b"], vec![var("c")]),
        );
        verify(
            g.method(),
            "-- a",
            method("--", vec!["a"], vec![], vec![], vec![]),
        );
    }

    #[gtest]
    fn binary_pragma() {
        let g = SmalltalkGrammar::new();
        verify(
            g.pragma(),
            "<& true>",
            pragma("&", vec![Literal::Bool(true)]),
        );
        verify(g.pragma(), "<// nil>", pragma("//", vec![Literal::Nil]));
    }

    #[gtest]
    fn cascade_expression() {
        let g = SmalltalkGrammar::new();
        verify(
            g.expression(),
            "1 abs; negated",
            cascade(vec![
                msg(num(1.0), "abs", vec![]),
                msg(num(1.0), "negated", vec![]),
            ]),
        );
        verify(
            g.expression(),
            "1 abs negated; raisedTo: 12; rounded",
            cascade(vec![
                msg(msg(num(1.0), "abs", vec![]), "negated", vec![]),
                msg(msg(num(1.0), "abs", vec![]), "raisedTo:", vec![num(12.0)]),
                msg(msg(num(1.0), "abs", vec![]), "rounded", vec![]),
            ]),
        );
        verify(
            g.expression(),
            "1 + 2; - 3",
            cascade(vec![
                msg(num(1.0), "+", vec![num(2.0)]),
                msg(num(1.0), "-", vec![num(3.0)]),
            ]),
        );
    }

    #[gtest]
    fn keyword_expression() {
        let g = SmalltalkGrammar::new();
        verify(
            g.expression(),
            "1 to: 2",
            msg(num(1.0), "to:", vec![num(2.0)]),
        );
        verify(
            g.expression(),
            "1 to: 2 by: 3",
            msg(num(1.0), "to:by:", vec![num(2.0), num(3.0)]),
        );
        verify(
            g.expression(),
            "1 to: 2 by: 3 do: 4",
            msg(num(1.0), "to:by:do:", vec![num(2.0), num(3.0), num(4.0)]),
        );
    }

    #[gtest]
    fn keyword_method() {
        let g = SmalltalkGrammar::new();
        verify(
            g.method(),
            "to: a",
            method("to:", vec!["a"], vec![], vec![], vec![]),
        );
        verify(
            g.method(),
            "to: a do: b | c |",
            method("to:do:", vec!["a", "b"], vec![], vec!["c"], vec![]),
        );
        verify(
            g.method(),
            "to: a do: b by: c d",
            method(
                "to:do:by:",
                vec!["a", "b", "c"],
                vec![],
                vec![],
                vec![var("d")],
            ),
        );
        verify(
            g.method(),
            "to: a do: b by: c | d | e",
            method(
                "to:do:by:",
                vec!["a", "b", "c"],
                vec![],
                vec!["d"],
                vec![var("e")],
            ),
        );
    }

    #[gtest]
    fn keyword_pragma() {
        let g = SmalltalkGrammar::new();
        verify(
            g.pragma(),
            "<primitive: 42>",
            pragma("primitive:", vec![Literal::Number(42.0)]),
        );
        verify(
            g.pragma(),
            "<primitive: 'fileOpen' error: 0>",
            pragma(
                "primitive:error:",
                vec![Literal::Str("fileOpen".to_string()), Literal::Number(0.0)],
            ),
        );
    }

    #[gtest]
    fn unary_expression() {
        let g = SmalltalkGrammar::new();
        verify(g.expression(), "1 abs", msg(num(1.0), "abs", vec![]));
        verify(
            g.expression(),
            "1 abs negated",
            msg(msg(num(1.0), "abs", vec![]), "negated", vec![]),
        );
    }

    #[gtest]
    fn unary_method() {
        let g = SmalltalkGrammar::new();
        verify(
            g.method(),
            "abs",
            method("abs", vec![], vec![], vec![], vec![]),
        );
        verify(
            g.method(),
            "abs | a |",
            method("abs", vec![], vec![], vec!["a"], vec![]),
        );
        verify(
            g.method(),
            "abs a",
            method("abs", vec![], vec![], vec![], vec![var("a")]),
        );
        verify(
            g.method(),
            "abs | a | b",
            method("abs", vec![], vec![], vec!["a"], vec![var("b")]),
        );
    }

    #[gtest]
    fn unary_pragma() {
        let g = SmalltalkGrammar::new();
        verify(g.pragma(), "<menu>", pragma("menu", vec![]));
    }

    #[gtest]
    fn pragma_combinations() {
        let g = SmalltalkGrammar::new();
        verify(
            g.method(),
            "method <foo>",
            method(
                "method",
                vec![],
                vec![pragma("foo", vec![])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo> <bar>",
            method(
                "method",
                vec![],
                vec![pragma("foo", vec![]), pragma("bar", vec![])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method | a | <foo>",
            method(
                "method",
                vec![],
                vec![pragma("foo", vec![])],
                vec!["a"],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo> | a |",
            method(
                "method",
                vec![],
                vec![pragma("foo", vec![])],
                vec!["a"],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo> | a | <bar>",
            method(
                "method",
                vec![],
                vec![pragma("foo", vec![]), pragma("bar", vec![])],
                vec!["a"],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: 1>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Number(1.0)])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: 1.2>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Number(1.2)])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: 'bar'>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Str("bar".to_string())])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: #'bar'>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Str("bar".to_string())])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: bar>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Str("bar".to_string())])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: true>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Bool(true)])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: false>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Bool(false)])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: nil>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Nil])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: ()>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Array(vec![])])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method <foo: #()>",
            method(
                "method",
                vec![],
                vec![pragma("foo:", vec![Literal::Array(vec![])])],
                vec![],
                vec![],
            ),
        );
        verify(
            g.method(),
            "method < + 1 >",
            method(
                "method",
                vec![],
                vec![pragma("+", vec![Literal::Number(1.0)])],
                vec![],
                vec![],
            ),
        );
    }

    #[gtest]
    fn start_full_method() {
        let g = SmalltalkGrammar::new();
        let source = r#"exampleWithNumber: x
  "A method that illustrates every part of Smalltalk method syntax
  except primitives. It has unary, binary, and keyword messages,
  declares arguments and temporaries, accesses a global variable
  (but not and instance variable), uses literals (array, character,
  symbol, string, integer, float), uses the pseudo variables
  true false, nil, self, and super, and has sequence, assignment,
  return and cascade. It has both zero argument and one argument blocks."

  |y|
  y := true & false not & (nil isNil) ifFalse: [self halt].
  self size + super size.
  #($a #a "a" 1 1.0)
      do: [:each | Transcript show: (each class name);
                               show: ' '].
  ^ x < y"#;
        let result = g.start().parse(source);
        assert!(result.is_ok(), "expected the example method to parse");
    }
}
