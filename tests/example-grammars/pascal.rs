use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::prelude::*;
use rust_petitparser_macros::grammar;
use std::fmt::Debug;
use std::rc::Rc;

const KEYWORDS: &[&str] = &[
    "and",
    "array",
    "begin",
    "case",
    "const",
    "div",
    "do",
    "downto",
    "else",
    "end",
    "exit",
    "file",
    "for",
    "function",
    "goto",
    "if",
    "in",
    "label",
    "mod",
    "nil",
    "not",
    "of",
    "or",
    "packed",
    "procedure",
    "program",
    "record",
    "repeat",
    "set",
    "then",
    "to",
    "type",
    "until",
    "var",
    "while",
    "with",
];

fn is_keyword(literal: &str) -> bool {
    KEYWORDS.contains(&literal)
}

fn token_parser<T, S, P>(p: impl Parser<T>, spacer: P) -> impl Parser<T>
where
    T: Debug,
    S: Debug,
    P: Parser<S> + Clone,
{
    // `spacer` itself requires one-or-more whitespace/comment units (mirroring dart's
    // `spacer()`); trimming must allow *zero* occurrences, so wrap it in `.star()` here
    // rather than passing it directly.
    p.trim_with(spacer.clone().star(), spacer.star())
}

fn token_str<S, P>(literal: &'static str, spacer: P) -> Rc<dyn Parser<String>>
where
    S: Debug + 'static,
    P: Parser<S> + Clone + 'static,
{
    if is_keyword(literal) {
        Rc::new(token_parser(
            string_ignore_case(literal).skip_right(word().not()),
            spacer,
        ))
    } else {
        Rc::new(token_parser(string(literal), spacer))
    }
}

#[grammar]
mod pascal_grammar {
    fn start() -> impl Parser<()> {
        program().end()
    }

    pub fn program() -> impl Parser<()> {
        seq!(
            token_str("program", spacer()),
            identifier(),
            seq!(
                token_str("(", spacer()),
                identifier().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(")", spacer()),
            )
            .opt(),
            token_str(";", spacer()),
            block(),
            token_str(".", spacer()),
        )
        .constant(())
    }

    pub fn statement() -> impl Parser<()> {
        seq!(
            statement_label().opt(),
            choice11(
                statement_assign(),
                statement_call(),
                statement_block(),
                statement_if(),
                statement_repeat(),
                statement_while(),
                statement_for(),
                statement_case(),
                statement_with(),
                statement_goto(),
                statement_exit(),
            ),
        )
        .constant(())
        .opt()
        .constant(())
    }

    pub fn statement_label() -> impl Parser<()> {
        seq!(unsigned_integer(), token_str(":", spacer())).constant(())
    }

    pub fn statement_assign() -> impl Parser<()> {
        seq!(variable(), token_str(":=", spacer()), expression()).constant(())
    }

    pub fn statement_call() -> impl Parser<()> {
        seq!(
            identifier(),
            seq!(
                token_str("(", spacer()),
                expression().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(")", spacer()),
            )
            .opt(),
        )
        .constant(())
    }

    pub fn statement_block() -> impl Parser<()> {
        seq!(
            token_str("begin", spacer()),
            statement().plus_sep(token_str(";", spacer()), Trailing::Disallowed),
            token_str("end", spacer()),
        )
        .constant(())
    }

    pub fn statement_if() -> impl Parser<()> {
        seq!(
            token_str("if", spacer()),
            expression(),
            token_str("then", spacer()),
            statement(),
            seq!(token_str("else", spacer()), statement()).opt(),
        )
        .constant(())
    }

    pub fn statement_repeat() -> impl Parser<()> {
        seq!(
            token_str("repeat", spacer()),
            statement().plus_sep(token_str(";", spacer()), Trailing::Disallowed),
            token_str("until", spacer()),
            expression(),
        )
        .constant(())
    }

    pub fn statement_while() -> impl Parser<()> {
        seq!(
            token_str("while", spacer()),
            expression(),
            token_str("do", spacer()),
            statement(),
        )
        .constant(())
    }

    pub fn statement_for() -> impl Parser<()> {
        seq!(
            token_str("for", spacer()),
            identifier(),
            token_str(":=", spacer()),
            expression(),
            choice!(token_str("to", spacer()), token_str("downto", spacer())),
            expression(),
            token_str("do", spacer()),
            statement(),
        )
        .constant(())
    }

    pub fn statement_case() -> impl Parser<()> {
        seq!(
            token_str("case", spacer()),
            expression(),
            token_str("of", spacer()),
            seq!(
                pascal_constant().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(":", spacer()),
                statement(),
            )
            .constant(())
            .plus_sep(token_str(";", spacer()), Trailing::Disallowed),
            token_str("end", spacer()),
        )
        .constant(())
    }

    pub fn statement_with() -> impl Parser<()> {
        seq!(
            token_str("with", spacer()),
            variable().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
            token_str("do", spacer()),
            statement(),
        )
        .constant(())
    }

    pub fn statement_goto() -> impl Parser<()> {
        seq!(token_str("goto", spacer()), unsigned_integer()).constant(())
    }

    pub fn statement_exit() -> impl Parser<()> {
        seq!(
            token_str("exit", spacer()),
            token_str("(", spacer()),
            choice!(token_str("program", spacer()), identifier()),
            token_str(")", spacer()),
        )
        .constant(())
    }

    pub fn block() -> impl Parser<()> {
        seq!(
            block_label().opt(),
            block_const().opt(),
            block_type().opt(),
            block_var().opt(),
            choice!(block_procedure(), block_function()).star(),
            block_statement(),
        )
        .constant(())
    }

    pub fn block_label() -> impl Parser<()> {
        seq!(
            token_str("label", spacer()),
            unsigned_integer().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
            token_str(";", spacer()),
        )
        .constant(())
    }

    pub fn block_const() -> impl Parser<()> {
        seq!(
            token_str("const", spacer()),
            seq!(
                identifier(),
                token_str("=", spacer()),
                pascal_constant(),
                token_str(";", spacer()),
            )
            .constant(())
            .plus(),
        )
        .constant(())
    }

    pub fn block_type() -> impl Parser<()> {
        seq!(
            token_str("type", spacer()),
            seq!(
                identifier(),
                token_str("=", spacer()),
                pascal_type(),
                token_str(";", spacer()),
            )
            .constant(())
            .plus(),
        )
        .constant(())
    }

    pub fn block_var() -> impl Parser<()> {
        seq!(
            token_str("var", spacer()),
            seq!(
                identifier().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(":", spacer()),
                pascal_type(),
                token_str(";", spacer()),
            )
            .constant(())
            .plus(),
        )
        .constant(())
    }

    pub fn block_procedure() -> impl Parser<()> {
        seq!(
            token_str("procedure", spacer()),
            identifier(),
            parameter_list(),
            token_str(";", spacer()),
            block(),
            token_str(";", spacer()),
        )
        .constant(())
    }

    pub fn block_function() -> impl Parser<()> {
        seq!(
            token_str("function", spacer()),
            identifier(),
            parameter_list(),
            token_str(":", spacer()),
            identifier(),
            token_str(";", spacer()),
            block(),
            token_str(";", spacer()),
        )
        .constant(())
    }

    pub fn block_statement() -> impl Parser<()> {
        seq!(
            token_str("begin", spacer()),
            statement().plus_sep(token_str(";", spacer()), Trailing::Disallowed),
            token_str("end", spacer()),
        )
        .constant(())
    }

    // Dart names this rule `type`, but `type` is a Rust keyword.
    pub fn pascal_type() -> impl Parser<()> {
        choice!(
            simple_type(),
            type_pointer(),
            seq!(
                token_str("packed", spacer()).opt(),
                choice!(type_set(), type_array(), type_record(), type_file()),
            )
            .constant(()),
        )
    }

    pub fn type_pointer() -> impl Parser<()> {
        seq!(token_str("^", spacer()), identifier()).constant(())
    }

    pub fn type_set() -> impl Parser<()> {
        seq!(
            token_str("set", spacer()),
            token_str("of", spacer()),
            simple_type(),
        )
        .constant(())
    }

    pub fn type_array() -> impl Parser<()> {
        seq!(
            token_str("array", spacer()),
            token_str("[", spacer()),
            simple_type().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
            token_str("]", spacer()),
            token_str("of", spacer()),
            pascal_type(),
        )
        .constant(())
    }

    pub fn type_record() -> impl Parser<()> {
        seq!(
            token_str("record", spacer()),
            field_list(),
            token_str("end", spacer()),
        )
        .constant(())
    }

    pub fn type_file() -> impl Parser<()> {
        seq!(
            token_str("file", spacer()),
            seq!(token_str("of", spacer()), pascal_type()).opt(),
        )
        .constant(())
    }

    pub fn identifier() -> impl Parser<String> {
        token_parser(
            seq!(letter(), word().star()).input_with_message("identifier expected".to_string()),
            spacer(),
        )
        .only_if_with_message(
            |s: &String| !is_keyword(s),
            "identifier expected".to_string(),
        )
    }

    pub fn variable() -> impl Parser<()> {
        seq!(
            identifier(),
            choice!(
                seq!(
                    token_str("[", spacer()),
                    expression().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                    token_str("]", spacer()),
                )
                .constant(()),
                seq!(token_str(".", spacer()), identifier()).constant(()),
                token_str("^", spacer()).constant(()),
            )
            .star(),
        )
        .constant(())
    }

    pub fn unsigned_number() -> impl Parser<f64> {
        token_parser(
            seq!(
                digit().plus(),
                seq!(char('.'), digit().plus()).opt(),
                seq!(one_of("eE"), one_of("+-").opt(), digit().plus()).opt(),
            )
            .input_with_message("unsigned number expected".to_string())
            .map(|s: String| s.parse::<f64>().unwrap()),
            spacer(),
        )
    }

    pub fn string_literal() -> impl Parser<String> {
        token_parser(
            seq!(char('\''), pattern("^'").star(), char('\''))
                .input_with_message("string expected".to_string()),
            spacer(),
        )
    }

    pub fn expression() -> impl Parser<()> {
        seq!(
            simple_expression(),
            seq!(
                choice!(
                    token_str("<", spacer()),
                    token_str("<=", spacer()),
                    token_str("=", spacer()),
                    token_str("<>", spacer()),
                    token_str(">=", spacer()),
                    token_str(">", spacer()),
                    token_str("in", spacer()),
                ),
                simple_expression(),
            )
            .opt(),
        )
        .constant(())
    }

    pub fn simple_expression() -> impl Parser<()> {
        seq!(
            choice!(token_str("+", spacer()), token_str("-", spacer())).opt(),
            term().plus_sep(token_str("or", spacer()), Trailing::Disallowed),
        )
        .plus()
        .constant(())
    }

    pub fn term() -> impl Parser<()> {
        factor()
            .plus_sep(
                choice!(
                    token_str("*", spacer()),
                    token_str("/", spacer()),
                    token_str("div", spacer()),
                    token_str("mod", spacer()),
                    token_str("and", spacer()),
                ),
                Trailing::Disallowed,
            )
            .constant(())
    }

    pub fn factor() -> impl Parser<()> {
        choice!(
            seq!(
                token_str("(", spacer()),
                expression(),
                token_str(")", spacer()),
            )
            .constant(()),
            seq!(token_str("not", spacer()), factor()).constant(()),
            seq!(
                token_str("[", spacer()),
                seq!(
                    expression(),
                    seq!(token_str("..", spacer()), expression()).opt(),
                )
                .constant(())
                .star_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str("]", spacer()),
            )
            .constant(()),
            seq!(
                identifier(),
                seq!(
                    token_str("(", spacer()),
                    expression().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                    token_str(")", spacer()),
                )
                .opt(),
            )
            .constant(()),
            unsigned_constant(),
            variable(),
        )
    }

    pub fn unsigned_constant() -> impl Parser<()> {
        choice!(
            token_str("nil", spacer()).constant(()),
            string_literal().constant(()),
            unsigned_number().constant(()),
            identifier().constant(()),
        )
    }

    pub fn parameter_list() -> impl Parser<()> {
        seq!(
            token_str("(", spacer()),
            seq!(
                token_str("var", spacer()).opt(),
                identifier().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(":", spacer()),
                identifier(),
            )
            .constant(())
            .plus_sep(token_str(";", spacer()), Trailing::Disallowed),
            token_str(")", spacer()),
        )
        .constant(())
        .opt()
        .constant(())
    }

    pub fn unsigned_integer() -> impl Parser<i64> {
        token_parser(
            digit()
                .plus()
                .input_with_message("unsigned integer expected".to_string())
                .map(|s: String| s.parse::<i64>().unwrap()),
            spacer(),
        )
    }

    pub fn pascal_constant() -> impl Parser<()> {
        choice!(
            seq!(
                one_of("+-"),
                choice!(identifier().constant(()), unsigned_number().constant(())),
            )
            .constant(()),
            unsigned_constant(),
        )
    }

    pub fn simple_type() -> impl Parser<()> {
        choice!(
            seq!(
                token_str("(", spacer()),
                identifier().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(")", spacer()),
            )
            .constant(()),
            seq!(
                pascal_constant(),
                token_str("..", spacer()),
                pascal_constant(),
            )
            .constant(()),
            identifier().constant(()),
        )
    }

    pub fn field_list() -> impl Parser<()> {
        choice!(
            seq!(field_list_base(), field_list_case().opt()).constant(()),
            field_list_case(),
        )
    }

    pub fn field_list_base() -> impl Parser<()> {
        seq!(
            identifier().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
            token_str(":", spacer()),
            pascal_type(),
        )
        .constant(())
        .plus_sep(token_str(";", spacer()), Trailing::Disallowed)
        .constant(())
    }

    pub fn field_list_case() -> impl Parser<()> {
        seq!(
            token_str("case", spacer()),
            seq!(identifier(), token_str(":", spacer())).opt(),
            identifier(),
            token_str("of", spacer()),
            seq!(
                pascal_constant().plus_sep(token_str(",", spacer()), Trailing::Disallowed),
                token_str(":", spacer()),
                token_str("(", spacer()),
                field_list(),
                token_str(")", spacer()),
            )
            .constant(())
            .plus_sep(token_str(";", spacer()), Trailing::Disallowed),
        )
        .constant(())
    }

    pub fn spacer() -> impl Parser<()> {
        choice!(whitespace().constant(()), comment())
            .plus()
            .constant(())
    }

    pub fn comment() -> impl Parser<()> {
        seq!(
            string("(*"),
            choice!(comment(), any().constant(())).star_lazy(string("*)")),
            string("*)"),
        )
        .constant(())
    }
}

#[gtest]
fn program_production() {
    let g = PascalGrammar::new();
    let p = g.program().end();
    assert_success!(p, "program foo; begin end.", ());
    assert_success!(p, "program foo(a); begin end.", ());
    assert_success!(p, "program foo(a, b); begin end.", ());
}

#[gtest]
fn statement_production() {
    let g = PascalGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "foo", ());
    assert_success!(p, "foo(1)", ());
    assert_success!(p, "123: a := 1", ());
    assert_success!(p, "123: a(1, 2)", ());
}

#[gtest]
fn statement_assign_production() {
    let g = PascalGrammar::new();
    let p = g.statement_assign().end();
    assert_success!(p, "a := 1", ());
    assert_success!(p, "a := b", ());
    assert_success!(p, "a := b + 1", ());
}

#[gtest]
fn statement_call_production() {
    let g = PascalGrammar::new();
    let p = g.statement_call().end();
    assert_success!(p, "a", ());
    assert_success!(p, "a(1)", ());
    assert_success!(p, "a(1, 2)", ());
}

#[gtest]
fn statement_block_production() {
    let g = PascalGrammar::new();
    let p = g.statement_block().end();
    assert_success!(p, "begin foo end", ());
    assert_success!(p, "begin foo; bar end", ());
}

#[gtest]
fn statement_if_production() {
    let g = PascalGrammar::new();
    let p = g.statement_if().end();
    assert_success!(p, "if a then foo", ());
    assert_success!(p, "if a then foo else bar", ());
}

#[gtest]
fn statement_repeat_production() {
    let g = PascalGrammar::new();
    let p = g.statement_repeat().end();
    assert_success!(p, "repeat foo until a", ());
    assert_success!(p, "repeat foo; bar until a", ());
}

#[gtest]
fn statement_while_production() {
    let g = PascalGrammar::new();
    let p = g.statement_while().end();
    assert_success!(p, "while a do foo", ());
}

#[gtest]
fn statement_for_production() {
    let g = PascalGrammar::new();
    let p = g.statement_for().end();
    assert_success!(p, "for i := a to b do foo", ());
    assert_success!(p, "for i := a downto b do foo", ());
}

#[gtest]
fn statement_case_production() {
    let g = PascalGrammar::new();
    let p = g.statement_case().end();
    assert_success!(p, "case a of 1: foo end", ());
    assert_success!(p, "case a of 1, 2: foo end", ());
    assert_success!(p, "case a of 1: foo; 2: bar end", ());
}

#[gtest]
fn statement_with_production() {
    let g = PascalGrammar::new();
    let p = g.statement_with().end();
    assert_success!(p, "with a do a := 1", ());
    assert_success!(p, "with a, b do a := 1", ());
}

#[gtest]
fn statement_goto_production() {
    let g = PascalGrammar::new();
    let p = g.statement_goto().end();
    assert_success!(p, "goto 1", ());
}

#[gtest]
fn statement_exit_production() {
    let g = PascalGrammar::new();
    let p = g.statement_exit().end();
    assert_success!(p, "exit(program)", ());
    assert_success!(p, "exit(foo)", ());
}

#[gtest]
fn block_production() {
    let g = PascalGrammar::new();
    let p = g.block().end();
    assert_success!(p, "begin end", ());
    assert_success!(p, "label 1; begin end", ());
    assert_success!(p, "const a = 1; begin end", ());
    assert_success!(p, "type a = b; begin end", ());
    assert_success!(p, "var a: b; begin end", ());
    assert_success!(p, "procedure foo; begin end; begin end", ());
    assert_success!(p, "function foo: a; begin end; begin end", ());
}

#[gtest]
fn block_label_production() {
    let g = PascalGrammar::new();
    let p = g.block_label().end();
    assert_success!(p, "label 1;", ());
    assert_success!(p, "label 1, 2;", ());
}

#[gtest]
fn block_const_production() {
    let g = PascalGrammar::new();
    let p = g.block_const().end();
    assert_success!(p, "const a = 1;", ());
    assert_success!(p, "const a = 1; b = 2;", ());
}

#[gtest]
fn block_type_production() {
    let g = PascalGrammar::new();
    let p = g.block_type().end();
    assert_success!(p, "type a = b;", ());
    assert_success!(p, "type a = b; c = d;", ());
}

#[gtest]
fn block_var_production() {
    let g = PascalGrammar::new();
    let p = g.block_var().end();
    assert_success!(p, "var a: b;", ());
    assert_success!(p, "var a, b: c;", ());
    assert_success!(p, "var a: b; c: d;", ());
}

#[gtest]
fn block_procedure_production() {
    let g = PascalGrammar::new();
    let p = g.block_procedure().end();
    assert_success!(p, "procedure foo; begin end;", ());
    assert_success!(p, "procedure foo(a: b); begin end;", ());
    assert_success!(p, "procedure foo(a: b); var a: b; begin end;", ());
}

#[gtest]
fn block_function_production() {
    let g = PascalGrammar::new();
    let p = g.block_function().end();
    assert_success!(p, "function foo: a; begin end;", ());
    assert_success!(p, "function foo(a: b): c; begin end;", ());
    assert_success!(p, "function foo(a: b): c; var a: b; begin end;", ());
}

#[gtest]
fn block_statement_production() {
    let g = PascalGrammar::new();
    let p = g.block_statement().end();
    assert_success!(p, "begin end", ());
    assert_success!(p, "begin foo end", ());
    assert_success!(p, "begin foo; bar end", ());
}

#[gtest]
fn type_production() {
    let g = PascalGrammar::new();
    let p = g.pascal_type().end();
    assert_success!(p, "a", ());
    assert_success!(p, "^a", ());
    assert_success!(p, "packed set of a", ());
    assert_success!(p, "packed array [a] of b", ());
    assert_success!(p, "packed record a: b end", ());
    assert_success!(p, "packed file", ());
}

#[gtest]
fn type_pointer_production() {
    let g = PascalGrammar::new();
    let p = g.type_pointer().end();
    assert_success!(p, "^a", ());
}

#[gtest]
fn type_set_production() {
    let g = PascalGrammar::new();
    let p = g.type_set().end();
    assert_success!(p, "set of a", ());
}

#[gtest]
fn type_array_production() {
    let g = PascalGrammar::new();
    let p = g.type_array().end();
    assert_success!(p, "array [a] of b", ());
    assert_success!(p, "array [a, b] of c", ());
}

#[gtest]
fn type_record_production() {
    let g = PascalGrammar::new();
    let p = g.type_record().end();
    assert_success!(p, "record a: b end", ());
    assert_success!(p, "record a, b: c end", ());
    assert_success!(p, "record case a of 1: (b: c) end", ());
    assert_success!(p, "record case a: b of 1: (a: b) end", ());
    assert_success!(p, "record case a of 1, 2: (a: b) end", ());
    assert_success!(p, "record case a of 1: (b: c); 2: (d: e) end", ());
}

#[gtest]
fn type_file_production() {
    let g = PascalGrammar::new();
    let p = g.type_file().end();
    assert_success!(p, "file", ());
    assert_success!(p, "file of a", ());
}

#[gtest]
fn identifier_production() {
    let g = PascalGrammar::new();
    let p = g.identifier().end();
    assert_success!(p, "a", &"a".to_string());
    assert_success!(p, "abc", &"abc".to_string());
    assert_success!(p, "a123", &"a123".to_string());
}

#[gtest]
fn variable_production() {
    let g = PascalGrammar::new();
    let p = g.variable().end();
    assert_success!(p, "a", ());
    assert_success!(p, "a[1]", ());
    assert_success!(p, "a[1,2]", ());
    assert_success!(p, "a[1][2]", ());
    assert_success!(p, "a.b", ());
    assert_success!(p, "a.b.c", ());
    assert_success!(p, "a^", ());
    assert_success!(p, "a^^", ());
}

#[gtest]
fn unsigned_number_production() {
    let g = PascalGrammar::new();
    let p = g.unsigned_number().end();
    assert_success!(p, "0", 0.0);
    assert_success!(p, "123", 123.0);
    assert_success!(p, "123.456", 123.456);
    assert_success!(p, "123.456e7", 123.456e7);
    assert_success!(p, "123.456e+7", 123.456e+7);
    assert_success!(p, "123e-4", 123e-4);
}

#[gtest]
fn string_literal_production() {
    let g = PascalGrammar::new();
    let p = g.string_literal().end();
    assert_success!(p, "''", &"''".to_string());
    assert_success!(p, "'whatever'", &"'whatever'".to_string());
}

#[gtest]
fn expression_production() {
    let g = PascalGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a", ());
    assert_success!(p, "a = b", ());
    assert_success!(p, "1 in b", ());
}

#[gtest]
fn simple_expression_production() {
    let g = PascalGrammar::new();
    let p = g.simple_expression().end();
    assert_success!(p, "a", ());
    assert_success!(p, "+ a", ());
    assert_success!(p, "- a", ());
    assert_success!(p, "a + b", ());
    assert_success!(p, "a - b - c", ());
    assert_success!(p, "a or b", ());
    assert_success!(p, "a or b or c", ());
}

#[gtest]
fn term_production() {
    let g = PascalGrammar::new();
    let p = g.term().end();
    assert_success!(p, "a", ());
    assert_success!(p, "a * b", ());
    assert_success!(p, "a mod b", ());
    assert_success!(p, "a * b / c", ());
    assert_success!(p, "a and b and c", ());
}

#[gtest]
fn factor_production() {
    let g = PascalGrammar::new();
    let p = g.factor().end();
    assert_success!(p, "1", ());
    assert_success!(p, "a", ());
    assert_success!(p, "sin(a)", ());
    assert_success!(p, "arctan(a, b)", ());
    assert_success!(p, "not a", ());
    assert_success!(p, "[]", ());
    assert_success!(p, "[1]", ());
    assert_success!(p, "[1, 2]", ());
    assert_success!(p, "[1..2]", ());
    assert_success!(p, "[1..2, 3..4]", ());
}

#[gtest]
fn unsigned_constant_production() {
    let g = PascalGrammar::new();
    let p = g.unsigned_constant().end();
    assert_success!(p, "1", ());
    assert_success!(p, "a", ());
    assert_success!(p, "''", ());
    assert_success!(p, "nil", ());
}

#[gtest]
fn parameter_list_production() {
    let g = PascalGrammar::new();
    let p = g.parameter_list().end();
    assert_success!(p, "", ());
    assert_success!(p, "(a: b)", ());
    assert_success!(p, "(a: b; c: d)", ());
    assert_success!(p, "(a, b: c)", ());
    assert_success!(p, "(var a: b)", ());
    assert_success!(p, "(var a: b; var c: d)", ());
    assert_success!(p, "(var a, b: c)", ());
}

#[gtest]
fn unsigned_integer_production() {
    let g = PascalGrammar::new();
    let p = g.unsigned_integer().end();
    assert_success!(p, "0", 0);
    assert_success!(p, "123", 123);
    assert_success!(p, "12345", 12345);
}

#[gtest]
fn constant_production() {
    let g = PascalGrammar::new();
    let p = g.pascal_constant().end();
    assert_success!(p, "a", ());
    assert_success!(p, "+b", ());
    assert_success!(p, "-c", ());
    assert_success!(p, "1", ());
    assert_success!(p, "+2", ());
    assert_success!(p, "-3", ());
    assert_success!(p, "'hello'", ());
    assert_success!(p, "nil", ());
}

#[gtest]
fn simple_type_production() {
    let g = PascalGrammar::new();
    let p = g.simple_type().end();
    assert_success!(p, "a", ());
    assert_success!(p, "(a)", ());
    assert_success!(p, "(a, b)", ());
    assert_success!(p, "a..b", ());
}

#[gtest]
fn field_list_production() {
    let g = PascalGrammar::new();
    let p = g.field_list().end();
    assert_success!(p, "a: b", ());
    assert_success!(p, "a, b: c", ());
    assert_success!(p, "case a of b : (c: d)", ());
    assert_success!(p, "case a : b of c : (d: e)", ());
    assert_success!(p, "case a of b, c : (d: e)", ());
    assert_success!(p, "case a of b : (c: d); e : (f: g)", ());
    assert_success!(p, "a: b case c of d : (e: f)", ());
}

#[gtest]
fn hello_world() {
    let g = PascalGrammar::new();
    let input = "program simple;\nbegin\n  writeln('Hello World!');\nend.";
    assert_success!(g, input, ());
}

#[gtest]
fn comparestrings() {
    let g = PascalGrammar::new();
    // Cleaned-up equivalent of dart's fixture: the original relies on adjacent dart
    // string-literal concatenation (no comma between two `join('\n')` list entries),
    // which has no Rust equivalent and isn't meaningful test content on its own.
    let input = "program comparestrings;\n\
                 var s: string;\n\
                 \tt: string;\n\
                 begin\n\
                 \ts := 'something';\n\
                 \tt := 'something bigger';\n\
                 \tif s = t then\n\
                 \t\twriteln(s, ' is equal to ', t)\n\
                 \telse\n\
                 \t\tif s > t then\n\
                 \t\t\twriteln(s, ' is greater than ', t)\n\
                 \t\telse\n\
                 \t\t\tif s < t then\n\
                 \t\t\t\twriteln(s, ' is less than ', t);\n\
                 end.";
    assert_success!(g, input, ());
}
