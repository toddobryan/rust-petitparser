use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::prelude::*;
use rust_petitparser_macros::grammar;
use std::fmt::Debug;
use std::rc::Rc;

fn token_parser<T, S, P>(p: impl Parser<T>, spacer: P) -> impl Parser<()>
where
    T: Debug,
    S: Debug,
    P: Parser<S> + Clone,
{
    p.trim_with(spacer.clone().star(), spacer.star())
        .map(|_| ())
}

fn token_str<S, P>(literal: &'static str, spacer: P) -> impl Parser<()>
where
    S: Debug,
    P: Parser<S> + Clone,
{
    token_parser(string(literal), spacer)
}

#[grammar]
mod dart_grammar {
    // -----------------------------------------------------------------
    // Keyword definitions.
    // -----------------------------------------------------------------
    fn break_token() -> impl Parser<()> {
        token_str("break", hidden_stuff_whitespace())
    }
    fn case_token() -> impl Parser<()> {
        token_str("case", hidden_stuff_whitespace())
    }
    fn catch_token() -> impl Parser<()> {
        token_str("catch", hidden_stuff_whitespace())
    }
    fn const_token() -> impl Parser<()> {
        token_str("const", hidden_stuff_whitespace())
    }
    fn continue_token() -> impl Parser<()> {
        token_str("continue", hidden_stuff_whitespace())
    }
    fn default_token() -> impl Parser<()> {
        token_str("default", hidden_stuff_whitespace())
    }
    fn do_token() -> impl Parser<()> {
        token_str("do", hidden_stuff_whitespace())
    }
    fn else_token() -> impl Parser<()> {
        token_str("else", hidden_stuff_whitespace())
    }
    fn false_token() -> impl Parser<()> {
        token_str("false", hidden_stuff_whitespace())
    }
    fn final_token() -> impl Parser<()> {
        token_str("final", hidden_stuff_whitespace())
    }
    fn finally_token() -> impl Parser<()> {
        token_str("finally", hidden_stuff_whitespace())
    }
    fn for_token() -> impl Parser<()> {
        token_str("for", hidden_stuff_whitespace())
    }
    fn if_token() -> impl Parser<()> {
        token_str("if", hidden_stuff_whitespace())
    }
    fn in_token() -> impl Parser<()> {
        token_str("in", hidden_stuff_whitespace())
    }
    fn new_token() -> impl Parser<()> {
        token_str("new", hidden_stuff_whitespace())
    }
    fn null_token() -> impl Parser<()> {
        token_str("null", hidden_stuff_whitespace())
    }
    fn return_token() -> impl Parser<()> {
        token_str("return", hidden_stuff_whitespace())
    }
    fn super_token() -> impl Parser<()> {
        token_str("super", hidden_stuff_whitespace())
    }
    fn switch_token() -> impl Parser<()> {
        token_str("switch", hidden_stuff_whitespace())
    }
    fn this_token() -> impl Parser<()> {
        token_str("this", hidden_stuff_whitespace())
    }
    fn throw_token() -> impl Parser<()> {
        token_str("throw", hidden_stuff_whitespace())
    }
    fn true_token() -> impl Parser<()> {
        token_str("true", hidden_stuff_whitespace())
    }
    fn try_token() -> impl Parser<()> {
        token_str("try", hidden_stuff_whitespace())
    }
    fn var_token() -> impl Parser<()> {
        token_str("var", hidden_stuff_whitespace())
    }
    fn void_token() -> impl Parser<()> {
        token_str("void", hidden_stuff_whitespace())
    }
    fn while_token() -> impl Parser<()> {
        token_str("while", hidden_stuff_whitespace())
    }

    // Pseudo-keywords that should also be valid identifiers.
    fn abstract_token() -> impl Parser<()> {
        token_str("abstract", hidden_stuff_whitespace())
    }
    fn as_token() -> impl Parser<()> {
        token_str("as", hidden_stuff_whitespace())
    }
    fn assert_token() -> impl Parser<()> {
        token_str("assert", hidden_stuff_whitespace())
    }
    fn class_token() -> impl Parser<()> {
        token_str("class", hidden_stuff_whitespace())
    }
    fn deferred_token() -> impl Parser<()> {
        token_str("deferred", hidden_stuff_whitespace())
    }
    fn export_token() -> impl Parser<()> {
        token_str("export", hidden_stuff_whitespace())
    }
    fn extends_token() -> impl Parser<()> {
        token_str("extends", hidden_stuff_whitespace())
    }
    fn factory_token() -> impl Parser<()> {
        token_str("factory", hidden_stuff_whitespace())
    }
    fn get_token() -> impl Parser<()> {
        token_str("get", hidden_stuff_whitespace())
    }
    fn hide_token() -> impl Parser<()> {
        token_str("hide", hidden_stuff_whitespace())
    }
    fn implements_token() -> impl Parser<()> {
        token_str("implements", hidden_stuff_whitespace())
    }
    fn import_token() -> impl Parser<()> {
        token_str("import", hidden_stuff_whitespace())
    }
    fn is_token() -> impl Parser<()> {
        token_str("is", hidden_stuff_whitespace())
    }
    fn library_token() -> impl Parser<()> {
        token_str("library", hidden_stuff_whitespace())
    }
    fn native_token() -> impl Parser<()> {
        token_str("native", hidden_stuff_whitespace())
    }
    fn negate_token() -> impl Parser<()> {
        token_str("negate", hidden_stuff_whitespace())
    }
    fn of_token() -> impl Parser<()> {
        token_str("of", hidden_stuff_whitespace())
    }
    fn operator_token() -> impl Parser<()> {
        token_str("operator", hidden_stuff_whitespace())
    }
    fn part_token() -> impl Parser<()> {
        token_str("part", hidden_stuff_whitespace())
    }
    fn set_token() -> impl Parser<()> {
        token_str("set", hidden_stuff_whitespace())
    }
    fn show_token() -> impl Parser<()> {
        token_str("show", hidden_stuff_whitespace())
    }
    fn static_token() -> impl Parser<()> {
        token_str("static", hidden_stuff_whitespace())
    }
    fn typedef_token() -> impl Parser<()> {
        token_str("typedef", hidden_stuff_whitespace())
    }

    // -----------------------------------------------------------------
    // Grammar productions.
    // -----------------------------------------------------------------
    pub fn start() -> impl Parser<()> {
        compilation_unit().end()
    }

    pub fn compilation_unit() -> impl Parser<()> {
        seq4(
            hashbang_lexical_token().opt(),
            library_directive().opt(),
            import_directive().star(),
            top_level_definition().star(),
        )
        .map(|_| ())
    }

    pub fn library_directive() -> impl Parser<()> {
        choice2(
            seq3(
                library_token(),
                qualified(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq4(
                part_token(),
                of_token(),
                qualified(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn import_directive() -> impl Parser<()> {
        choice3(
            seq6(
                import_token(),
                single_line_string_lexical_token(),
                deferred_token().opt(),
                seq2(as_token(), identifier()).map(|_| ()).opt(),
                seq2(
                    choice2(show_token(), hide_token()),
                    identifier().plus_sep(
                        token_str(",", hidden_stuff_whitespace()),
                        Trailing::Disallowed,
                    ),
                )
                .map(|_| ())
                .opt(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq4(
                export_token(),
                single_line_string_lexical_token(),
                seq2(
                    choice2(show_token(), hide_token()),
                    identifier().plus_sep(
                        token_str(",", hidden_stuff_whitespace()),
                        Trailing::Disallowed,
                    ),
                )
                .map(|_| ())
                .opt(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq3(
                part_token(),
                single_line_string_lexical_token(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn top_level_definition() -> impl Parser<()> {
        choice7(
            class_definition(),
            function_type_alias(),
            seq2(function_declaration(), function_body_or_native()).map(|_| ()),
            seq5(
                return_type().opt(),
                get_or_set(),
                identifier(),
                formal_parameter_list(),
                function_body_or_native(),
            )
            .map(|_| ()),
            seq4(
                final_token(),
                dart_type().opt(),
                static_final_declaration_list(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq4(
                const_token(),
                dart_type().opt(),
                static_final_declaration_list(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq2(
                const_initialized_variable_declaration(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn class_definition() -> impl Parser<()> {
        choice2(
            seq9(
                abstract_token().opt(),
                class_token(),
                identifier(),
                type_parameters().opt(),
                superclass().opt(),
                interfaces().opt(),
                token_str("{", hidden_stuff_whitespace()),
                class_member_definition().star(),
                token_str("}", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq2(
                seq5(
                    abstract_token().opt(),
                    class_token(),
                    identifier(),
                    type_parameters().opt(),
                    interfaces().opt(),
                )
                .map(|_| ()),
                seq5(
                    native_token(),
                    token_parser(string_lexical_token(), hidden_stuff_whitespace()),
                    token_str("{", hidden_stuff_whitespace()),
                    class_member_definition().star(),
                    token_str("}", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
            )
            .map(|_| ()),
        )
    }

    pub fn type_parameter() -> impl Parser<()> {
        seq2(
            identifier(),
            seq2(extends_token(), dart_type()).map(|_| ()).opt(),
        )
        .map(|_| ())
    }

    pub fn type_parameters() -> impl Parser<()> {
        seq4(
            token_str("<", hidden_stuff_whitespace()),
            type_parameter(),
            seq2(token_str(",", hidden_stuff_whitespace()), type_parameter())
                .map(|_| ())
                .star(),
            token_str(">", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn superclass() -> impl Parser<()> {
        seq2(extends_token(), dart_type()).map(|_| ())
    }

    pub fn interfaces() -> impl Parser<()> {
        seq2(implements_token(), type_list()).map(|_| ())
    }

    // This rule is organized in a way that may not be most readable, but
    // gives the best error messages.
    pub fn class_member_definition() -> impl Parser<()> {
        choice4(
            seq2(declaration(), token_str(";", hidden_stuff_whitespace())).map(|_| ()),
            seq2(
                constructor_declaration(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq2(method_declaration(), function_body_or_native()).map(|_| ()),
            seq3(
                const_token(),
                factory_constructor_declaration(),
                function_native(),
            )
            .map(|_| ()),
        )
    }

    pub fn function_body_or_native() -> impl Parser<()> {
        choice3(
            seq2(native_token(), function_body()).map(|_| ()),
            function_native(),
            function_body(),
        )
    }

    pub fn function_native() -> impl Parser<()> {
        seq3(
            native_token(),
            token_parser(string_lexical_token(), hidden_stuff_whitespace()).opt(),
            token_str(";", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    // A method, operator, or constructor (which all should be followed by
    // a block of code).
    pub fn method_declaration() -> impl Parser<()> {
        choice5(
            factory_constructor_declaration(),
            seq2(static_token(), function_declaration()).map(|_| ()),
            special_signature_definition(),
            seq2(function_declaration(), initializers().opt()).map(|_| ()),
            seq2(named_constructor_declaration(), initializers().opt()).map(|_| ()),
        )
    }

    // An abstract method/operator, a field, or const constructor (which
    // all should be followed by a semicolon).
    pub fn declaration() -> impl Parser<()> {
        choice6(
            seq2(function_declaration(), redirection()).map(|_| ()),
            seq2(named_constructor_declaration(), redirection()).map(|_| ()),
            seq2(abstract_token(), special_signature_definition()).map(|_| ()),
            seq2(abstract_token(), function_declaration()).map(|_| ()),
            seq4(
                static_token(),
                final_token(),
                dart_type().opt(),
                static_final_declaration_list(),
            )
            .map(|_| ()),
            seq2(
                static_token().opt(),
                const_initialized_variable_declaration(),
            )
            .map(|_| ()),
        )
    }

    pub fn initializers() -> impl Parser<()> {
        seq3(
            token_str(":", hidden_stuff_whitespace()),
            super_call_or_field_initializer(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                super_call_or_field_initializer(),
            )
            .map(|_| ())
            .star(),
        )
        .map(|_| ())
    }

    pub fn redirection() -> impl Parser<()> {
        seq4(
            token_str(":", hidden_stuff_whitespace()),
            this_token(),
            seq2(token_str(".", hidden_stuff_whitespace()), identifier())
                .map(|_| ())
                .opt(),
            arguments(),
        )
        .map(|_| ())
    }

    pub fn field_initializer() -> impl Parser<()> {
        seq4(
            seq2(this_token(), token_str(".", hidden_stuff_whitespace()))
                .map(|_| ())
                .opt(),
            identifier(),
            token_str("=", hidden_stuff_whitespace()),
            conditional_expression(),
        )
        .map(|_| ())
    }

    pub fn super_call_or_field_initializer() -> impl Parser<()> {
        choice3(
            seq2(super_token(), arguments()).map(|_| ()),
            seq4(
                super_token(),
                token_str(".", hidden_stuff_whitespace()),
                identifier(),
                arguments(),
            )
            .map(|_| ()),
            field_initializer(),
        )
    }

    pub fn static_final_declaration_list() -> impl Parser<()> {
        seq2(
            static_final_declaration(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                static_final_declaration(),
            )
            .map(|_| ())
            .star(),
        )
        .map(|_| ())
    }

    pub fn static_final_declaration() -> impl Parser<()> {
        seq3(
            identifier(),
            token_str("=", hidden_stuff_whitespace()),
            constant_expression(),
        )
        .map(|_| ())
    }

    pub fn function_type_alias() -> impl Parser<()> {
        seq5(
            typedef_token(),
            function_prefix(),
            type_parameters().opt(),
            formal_parameter_list(),
            token_str(";", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn factory_constructor_declaration() -> impl Parser<()> {
        seq5(
            factory_token(),
            qualified(),
            type_parameters().opt(),
            seq2(token_str(".", hidden_stuff_whitespace()), identifier())
                .map(|_| ())
                .opt(),
            formal_parameter_list(),
        )
        .map(|_| ())
    }

    pub fn constructor_declaration() -> impl Parser<()> {
        choice2(
            seq4(
                const_token().opt(),
                identifier(),
                formal_parameter_list(),
                choice2(redirection(), initializers()).opt(),
            )
            .map(|_| ()),
            seq3(
                const_token().opt(),
                named_constructor_declaration(),
                choice2(redirection(), initializers()).opt(),
            )
            .map(|_| ()),
        )
    }

    pub fn named_constructor_declaration() -> impl Parser<()> {
        seq4(
            identifier(),
            token_str(".", hidden_stuff_whitespace()),
            identifier(),
            formal_parameter_list(),
        )
        .map(|_| ())
    }

    pub fn special_signature_definition() -> impl Parser<()> {
        choice2(
            seq5(
                static_token().opt(),
                return_type().opt(),
                get_or_set(),
                identifier(),
                formal_parameter_list(),
            )
            .map(|_| ()),
            seq4(
                return_type().opt(),
                operator_token(),
                user_definable_operator(),
                formal_parameter_list(),
            )
            .map(|_| ()),
        )
    }

    pub fn get_or_set() -> impl Parser<()> {
        choice2(get_token(), set_token())
    }

    pub fn user_definable_operator() -> impl Parser<()> {
        choice10(
            multiplicative_operator(),
            additive_operator(),
            shift_operator(),
            relational_operator(),
            bitwise_operator(),
            token_str("==", hidden_stuff_whitespace()),
            token_str("~", hidden_stuff_whitespace()),
            negate_token(),
            seq2(
                token_str("[", hidden_stuff_whitespace()),
                token_str("]", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq3(
                token_str("[", hidden_stuff_whitespace()),
                token_str("]", hidden_stuff_whitespace()),
                token_str("=", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn prefix_operator() -> impl Parser<()> {
        choice2(additive_operator(), negate_operator())
    }

    pub fn postfix_operator() -> impl Parser<()> {
        increment_operator()
    }

    pub fn negate_operator() -> impl Parser<()> {
        choice2(
            token_str("!", hidden_stuff_whitespace()),
            token_str("~", hidden_stuff_whitespace()),
        )
    }

    pub fn multiplicative_operator() -> impl Parser<()> {
        choice4(
            token_str("*", hidden_stuff_whitespace()),
            token_str("/", hidden_stuff_whitespace()),
            token_str("%", hidden_stuff_whitespace()),
            token_str("~/", hidden_stuff_whitespace()),
        )
    }

    pub fn assignment_operator() -> impl Parser<()> {
        choice2(
            choice7(
                token_str("=", hidden_stuff_whitespace()),
                token_str("*=", hidden_stuff_whitespace()),
                token_str("/=", hidden_stuff_whitespace()),
                token_str("~/=", hidden_stuff_whitespace()),
                token_str("%=", hidden_stuff_whitespace()),
                token_str("+=", hidden_stuff_whitespace()),
                token_str("-=", hidden_stuff_whitespace()),
            ),
            choice6(
                token_str("<<=", hidden_stuff_whitespace()),
                token_str(">>>=", hidden_stuff_whitespace()),
                token_str(">>=", hidden_stuff_whitespace()),
                token_str("&=", hidden_stuff_whitespace()),
                token_str("^=", hidden_stuff_whitespace()),
                token_str("|=", hidden_stuff_whitespace()),
            ),
        )
    }

    pub fn additive_operator() -> impl Parser<()> {
        choice2(
            token_str("+", hidden_stuff_whitespace()),
            token_str("-", hidden_stuff_whitespace()),
        )
    }

    pub fn increment_operator() -> impl Parser<()> {
        choice2(
            token_str("++", hidden_stuff_whitespace()),
            token_str("--", hidden_stuff_whitespace()),
        )
    }

    pub fn shift_operator() -> impl Parser<()> {
        choice3(
            token_str("<<", hidden_stuff_whitespace()),
            token_str(">>>", hidden_stuff_whitespace()),
            token_str(">>", hidden_stuff_whitespace()),
        )
    }

    pub fn relational_operator() -> impl Parser<()> {
        choice4(
            token_str(">=", hidden_stuff_whitespace()),
            token_str(">", hidden_stuff_whitespace()),
            token_str("<=", hidden_stuff_whitespace()),
            token_str("<", hidden_stuff_whitespace()),
        )
    }

    pub fn equality_operator() -> impl Parser<()> {
        choice4(
            token_str("===", hidden_stuff_whitespace()),
            token_str("!==", hidden_stuff_whitespace()),
            token_str("==", hidden_stuff_whitespace()),
            token_str("!=", hidden_stuff_whitespace()),
        )
    }

    pub fn bitwise_operator() -> impl Parser<()> {
        choice3(
            token_str("&", hidden_stuff_whitespace()),
            token_str("^", hidden_stuff_whitespace()),
            token_str("|", hidden_stuff_whitespace()),
        )
    }

    pub fn formal_parameter_list() -> impl Parser<()> {
        choice3(
            seq3(
                token_str("(", hidden_stuff_whitespace()),
                optional_formal_parameters().opt(),
                token_str(")", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq3(
                token_str("(", hidden_stuff_whitespace()),
                named_formal_parameters().opt(),
                token_str(")", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq4(
                token_str("(", hidden_stuff_whitespace()),
                normal_formal_parameter(),
                normal_formal_parameter_tail().opt(),
                token_str(")", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn normal_formal_parameter_tail() -> impl Parser<()> {
        choice3(
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                optional_formal_parameters(),
            )
            .map(|_| ()),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                named_formal_parameters(),
            )
            .map(|_| ()),
            seq3(
                token_str(",", hidden_stuff_whitespace()),
                normal_formal_parameter(),
                normal_formal_parameter_tail().opt(),
            )
            .map(|_| ()),
        )
    }

    pub fn normal_formal_parameter() -> impl Parser<()> {
        choice3(
            field_formal_parameter(),
            function_declaration(),
            simple_formal_parameter(),
        )
    }

    pub fn simple_formal_parameter() -> impl Parser<()> {
        choice2(declared_identifier(), identifier())
    }

    pub fn field_formal_parameter() -> impl Parser<()> {
        seq3(
            this_token(),
            token_str(".", hidden_stuff_whitespace()),
            identifier(),
        )
        .map(|_| ())
    }

    pub fn optional_formal_parameters() -> impl Parser<()> {
        seq4(
            token_str("[", hidden_stuff_whitespace()),
            default_formal_parameter(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                default_formal_parameter(),
            )
            .map(|_| ())
            .star(),
            token_str("]", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn named_formal_parameters() -> impl Parser<()> {
        seq4(
            token_str("{", hidden_stuff_whitespace()),
            named_format_parameter(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                named_format_parameter(),
            )
            .map(|_| ())
            .star(),
            token_str("}", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn named_format_parameter() -> impl Parser<()> {
        seq2(
            normal_formal_parameter(),
            seq2(
                token_str(":", hidden_stuff_whitespace()),
                constant_expression(),
            )
            .map(|_| ())
            .opt(),
        )
        .map(|_| ())
    }

    pub fn default_formal_parameter() -> impl Parser<()> {
        seq2(
            normal_formal_parameter(),
            seq2(
                token_str("=", hidden_stuff_whitespace()),
                constant_expression(),
            )
            .map(|_| ())
            .opt(),
        )
        .map(|_| ())
    }

    pub fn return_type() -> impl Parser<()> {
        choice2(void_token(), dart_type())
    }

    // We have to introduce a separate rule for 'declared' identifiers to
    // allow ANTLR to decide if the first identifier we encounter after
    // final is a type or an identifier. Before this change, we used the
    // production 'finalVarOrType identifier' in numerous places.
    pub fn declared_identifier() -> impl Parser<()> {
        choice3(
            seq3(final_token(), dart_type().opt(), identifier()).map(|_| ()),
            seq2(var_token(), identifier()).map(|_| ()),
            seq2(dart_type(), identifier()).map(|_| ()),
        )
    }

    pub fn identifier() -> impl Parser<()> {
        token_parser(identifier_lexical_token(), hidden_stuff_whitespace())
    }

    pub fn qualified() -> impl Parser<()> {
        seq2(
            identifier(),
            seq2(token_str(".", hidden_stuff_whitespace()), identifier())
                .map(|_| ())
                .star(),
        )
        .map(|_| ())
    }

    // Dart names this rule `type`, but `type` is a Rust keyword.
    pub fn dart_type() -> impl Parser<()> {
        seq2(qualified(), type_arguments().opt()).map(|_| ())
    }

    pub fn type_arguments() -> impl Parser<()> {
        seq3(
            token_str("<", hidden_stuff_whitespace()),
            type_list(),
            token_str(">", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn type_list() -> impl Parser<()> {
        seq2(
            dart_type(),
            seq2(token_str(",", hidden_stuff_whitespace()), dart_type())
                .map(|_| ())
                .star(),
        )
        .map(|_| ())
    }

    pub fn block() -> impl Parser<()> {
        seq3(
            token_str("{", hidden_stuff_whitespace()),
            statements(),
            token_str("}", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn statements() -> impl Parser<()> {
        statement().star().map(|_| ())
    }

    pub fn statement() -> impl Parser<()> {
        seq2(label().star(), non_labelled_statement()).map(|_| ())
    }

    pub fn non_labelled_statement() -> impl Parser<()> {
        choice2(
            choice6(
                block(),
                seq2(
                    initialized_variable_declaration(),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
                iteration_statement(),
                selection_statement(),
                try_statement(),
                seq3(
                    break_token(),
                    identifier().opt(),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
            ),
            choice6(
                seq3(
                    continue_token(),
                    identifier().opt(),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
                seq3(
                    return_token(),
                    expression().opt(),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
                seq3(
                    throw_token(),
                    expression().opt(),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
                seq2(
                    expression().opt(),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
                seq5(
                    assert_token(),
                    token_str("(", hidden_stuff_whitespace()),
                    conditional_expression(),
                    token_str(")", hidden_stuff_whitespace()),
                    token_str(";", hidden_stuff_whitespace()),
                )
                .map(|_| ()),
                seq2(function_declaration(), function_body()).map(|_| ()),
            ),
        )
    }

    pub fn label() -> impl Parser<()> {
        seq2(identifier(), token_str(":", hidden_stuff_whitespace())).map(|_| ())
    }

    pub fn iteration_statement() -> impl Parser<()> {
        choice3(
            seq5(
                while_token(),
                token_str("(", hidden_stuff_whitespace()),
                expression(),
                token_str(")", hidden_stuff_whitespace()),
                statement(),
            )
            .map(|_| ()),
            seq7(
                do_token(),
                statement(),
                while_token(),
                token_str("(", hidden_stuff_whitespace()),
                expression(),
                token_str(")", hidden_stuff_whitespace()),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq5(
                for_token(),
                token_str("(", hidden_stuff_whitespace()),
                for_loop_parts(),
                token_str(")", hidden_stuff_whitespace()),
                statement(),
            )
            .map(|_| ()),
        )
    }

    pub fn for_loop_parts() -> impl Parser<()> {
        choice3(
            seq4(
                for_initializer_statement(),
                expression().opt(),
                token_str(";", hidden_stuff_whitespace()),
                expression_list().opt(),
            )
            .map(|_| ()),
            seq3(declared_identifier(), in_token(), expression()).map(|_| ()),
            seq3(identifier(), in_token(), expression()).map(|_| ()),
        )
    }

    pub fn for_initializer_statement() -> impl Parser<()> {
        choice2(
            seq2(
                initialized_variable_declaration(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq2(
                expression().opt(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn selection_statement() -> impl Parser<()> {
        choice2(
            seq6(
                if_token(),
                token_str("(", hidden_stuff_whitespace()),
                expression(),
                token_str(")", hidden_stuff_whitespace()),
                statement(),
                seq2(else_token(), statement()).map(|_| ()).opt(),
            )
            .map(|_| ()),
            seq8(
                switch_token(),
                token_str("(", hidden_stuff_whitespace()),
                expression(),
                token_str(")", hidden_stuff_whitespace()),
                token_str("{", hidden_stuff_whitespace()),
                switch_case().star(),
                default_case().opt(),
                token_str("}", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
        )
    }

    pub fn switch_case() -> impl Parser<()> {
        seq3(
            label().opt(),
            seq3(
                case_token(),
                expression(),
                token_str(":", hidden_stuff_whitespace()),
            )
            .map(|_| ())
            .plus(),
            statements(),
        )
        .map(|_| ())
    }

    pub fn default_case() -> impl Parser<()> {
        seq4(
            label().opt(),
            default_token(),
            token_str(":", hidden_stuff_whitespace()),
            statements(),
        )
        .map(|_| ())
    }

    pub fn try_statement() -> impl Parser<()> {
        seq3(
            try_token(),
            block(),
            choice2(
                seq2(catch_part().plus(), finally_part().opt()).map(|_| ()),
                finally_part(),
            ),
        )
        .map(|_| ())
    }

    pub fn catch_part() -> impl Parser<()> {
        seq6(
            catch_token(),
            token_str("(", hidden_stuff_whitespace()),
            declared_identifier(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                declared_identifier(),
            )
            .map(|_| ())
            .opt(),
            token_str(")", hidden_stuff_whitespace()),
            block(),
        )
        .map(|_| ())
    }

    pub fn finally_part() -> impl Parser<()> {
        seq2(finally_token(), block()).map(|_| ())
    }

    pub fn initialized_variable_declaration() -> impl Parser<()> {
        seq3(
            declared_identifier(),
            seq2(token_str("=", hidden_stuff_whitespace()), expression())
                .map(|_| ())
                .opt(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                initialized_identifier(),
            )
            .map(|_| ())
            .star(),
        )
        .map(|_| ())
    }

    pub fn initialized_identifier() -> impl Parser<()> {
        seq2(
            identifier(),
            seq2(token_str("=", hidden_stuff_whitespace()), expression())
                .map(|_| ())
                .opt(),
        )
        .map(|_| ())
    }

    pub fn const_initialized_variable_declaration() -> impl Parser<()> {
        seq3(
            declared_identifier(),
            seq2(
                token_str("=", hidden_stuff_whitespace()),
                constant_expression(),
            )
            .map(|_| ())
            .opt(),
            seq2(
                token_str(",", hidden_stuff_whitespace()),
                const_initialized_identifier(),
            )
            .map(|_| ())
            .star(),
        )
        .map(|_| ())
    }

    pub fn const_initialized_identifier() -> impl Parser<()> {
        seq2(
            identifier(),
            seq2(
                token_str("=", hidden_stuff_whitespace()),
                constant_expression(),
            )
            .map(|_| ())
            .opt(),
        )
        .map(|_| ())
    }

    // The constant expression production is used to mark certain expressions
    // as only being allowed to hold a compile-time constant. The grammar cannot
    // express these restrictions (yet), so this will have to be enforced by a
    // separate analysis phase.
    pub fn constant_expression() -> impl Parser<()> {
        expression()
    }

    pub fn expression() -> impl Parser<()> {
        choice2(
            seq3(assignable_expression(), assignment_operator(), expression()).map(|_| ()),
            conditional_expression(),
        )
    }

    pub fn expression_list() -> impl Parser<()> {
        expression()
            .plus_sep(
                token_str(",", hidden_stuff_whitespace()),
                Trailing::Disallowed,
            )
            .map(|_| ())
    }

    pub fn arguments() -> impl Parser<()> {
        seq5(
            token_str("(", hidden_stuff_whitespace()),
            argument_list().opt(),
            token_str(",", hidden_stuff_whitespace()).opt(),
            token_str(")", hidden_stuff_whitespace()),
            null_safety_annotations().opt(),
        )
        .map(|_| ())
    }

    pub fn argument_list() -> impl Parser<()> {
        argument_element()
            .plus_sep(
                token_str(",", hidden_stuff_whitespace()),
                Trailing::Disallowed,
            )
            .map(|_| ())
    }

    pub fn argument_element() -> impl Parser<()> {
        choice2(seq2(label(), expression()).map(|_| ()), expression())
    }

    pub fn assignable_expression() -> impl Parser<()> {
        choice3(
            seq2(
                primary(),
                seq2(arguments().star(), assignable_selector())
                    .map(|_| ())
                    .plus(),
            )
            .map(|_| ()),
            seq2(super_token(), assignable_selector()).map(|_| ()),
            identifier(),
        )
    }

    pub fn conditional_expression() -> impl Parser<()> {
        seq2(
            logical_or_expression(),
            seq4(
                token_str("?", hidden_stuff_whitespace()),
                expression(),
                token_str(":", hidden_stuff_whitespace()),
                expression(),
            )
            .map(|_| ())
            .opt(),
        )
        .map(|_| ())
    }

    pub fn logical_or_expression() -> impl Parser<()> {
        seq2(
            logical_and_expression(),
            seq2(
                token_str("||", hidden_stuff_whitespace()),
                logical_and_expression(),
            )
            .map(|_| ())
            .star(),
        )
        .map(|_| ())
    }

    pub fn logical_and_expression() -> impl Parser<()> {
        seq2(
            bitwise_or_expression(),
            seq2(
                token_str("&&", hidden_stuff_whitespace()),
                bitwise_or_expression(),
            )
            .map(|_| ())
            .star(),
        )
        .map(|_| ())
    }

    pub fn bitwise_or_expression() -> impl Parser<()> {
        choice2(
            seq2(
                bitwise_xor_expression(),
                seq2(
                    token_str("|", hidden_stuff_whitespace()),
                    bitwise_xor_expression(),
                )
                .map(|_| ())
                .star(),
            )
            .map(|_| ()),
            seq2(
                super_token(),
                seq2(
                    token_str("|", hidden_stuff_whitespace()),
                    bitwise_xor_expression(),
                )
                .map(|_| ())
                .plus(),
            )
            .map(|_| ()),
        )
    }

    pub fn bitwise_xor_expression() -> impl Parser<()> {
        choice2(
            seq2(
                bitwise_and_expression(),
                seq2(
                    token_str("^", hidden_stuff_whitespace()),
                    bitwise_and_expression(),
                )
                .map(|_| ())
                .star(),
            )
            .map(|_| ()),
            seq2(
                super_token(),
                seq2(
                    token_str("^", hidden_stuff_whitespace()),
                    bitwise_and_expression(),
                )
                .map(|_| ())
                .plus(),
            )
            .map(|_| ()),
        )
    }

    pub fn bitwise_and_expression() -> impl Parser<()> {
        choice2(
            seq2(
                equality_expression(),
                seq2(
                    token_str("&", hidden_stuff_whitespace()),
                    equality_expression(),
                )
                .map(|_| ())
                .star(),
            )
            .map(|_| ()),
            seq2(
                super_token(),
                seq2(
                    token_str("&", hidden_stuff_whitespace()),
                    equality_expression(),
                )
                .map(|_| ())
                .plus(),
            )
            .map(|_| ()),
        )
    }

    pub fn equality_expression() -> impl Parser<()> {
        choice2(
            seq2(
                relational_expression(),
                seq2(equality_operator(), relational_expression())
                    .map(|_| ())
                    .opt(),
            )
            .map(|_| ()),
            seq3(super_token(), equality_operator(), relational_expression()).map(|_| ()),
        )
    }

    pub fn relational_expression() -> impl Parser<()> {
        choice2(
            seq2(
                shift_expression(),
                choice2(
                    seq2(is_operator(), dart_type()).map(|_| ()),
                    seq2(relational_operator(), shift_expression()).map(|_| ()),
                )
                .opt(),
            )
            .map(|_| ()),
            seq3(super_token(), relational_operator(), shift_expression()).map(|_| ()),
        )
    }

    pub fn is_operator() -> impl Parser<()> {
        seq2(is_token(), token_str("!", hidden_stuff_whitespace()).opt()).map(|_| ())
    }

    pub fn shift_expression() -> impl Parser<()> {
        choice2(
            seq2(
                additive_expression(),
                seq2(shift_operator(), additive_expression())
                    .map(|_| ())
                    .star(),
            )
            .map(|_| ()),
            seq2(
                super_token(),
                seq2(shift_operator(), additive_expression())
                    .map(|_| ())
                    .plus(),
            )
            .map(|_| ()),
        )
    }

    pub fn additive_expression() -> impl Parser<()> {
        choice2(
            seq2(
                multiplicative_expression(),
                seq2(additive_operator(), multiplicative_expression())
                    .map(|_| ())
                    .star(),
            )
            .map(|_| ()),
            seq2(
                super_token(),
                seq2(additive_operator(), multiplicative_expression())
                    .map(|_| ())
                    .plus(),
            )
            .map(|_| ()),
        )
    }

    pub fn multiplicative_expression() -> impl Parser<()> {
        choice2(
            seq2(
                unary_expression(),
                seq2(multiplicative_operator(), unary_expression())
                    .map(|_| ())
                    .star(),
            )
            .map(|_| ()),
            seq2(
                super_token(),
                seq2(multiplicative_operator(), unary_expression())
                    .map(|_| ())
                    .plus(),
            )
            .map(|_| ()),
        )
    }

    pub fn unary_expression() -> impl Parser<()> {
        choice5(
            postfix_expression(),
            seq2(prefix_operator(), unary_expression()).map(|_| ()),
            seq2(negate_operator(), super_token()).map(|_| ()),
            seq2(token_str("-", hidden_stuff_whitespace()), super_token()).map(|_| ()),
            seq2(increment_operator(), assignable_expression()).map(|_| ()),
        )
    }

    pub fn postfix_expression() -> impl Parser<()> {
        choice2(
            seq2(assignable_expression(), postfix_operator()).map(|_| ()),
            seq2(primary(), selector().star()).map(|_| ()),
        )
    }

    pub fn selector() -> impl Parser<()> {
        choice2(
            seq2(null_safety_annotations().opt(), assignable_selector()).map(|_| ()),
            arguments(),
        )
    }

    pub fn null_safety_annotations() -> impl Parser<()> {
        choice2(
            token_str("?", hidden_stuff_whitespace()),
            token_str("!", hidden_stuff_whitespace()),
        )
    }

    pub fn assignable_selector() -> impl Parser<()> {
        choice2(
            seq3(
                token_str("[", hidden_stuff_whitespace()),
                expression(),
                token_str("]", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            seq3(
                token_str(".", hidden_stuff_whitespace()),
                identifier(),
                null_safety_annotations().opt(),
            )
            .map(|_| ()),
        )
    }

    pub fn primary() -> impl Parser<()> {
        choice8(
            this_token(),
            seq2(super_token(), assignable_selector()).map(|_| ()),
            seq3(
                const_token().opt(),
                type_arguments().opt(),
                compound_literal(),
            )
            .map(|_| ()),
            seq4(
                choice2(new_token(), const_token()),
                dart_type(),
                seq2(token_str(".", hidden_stuff_whitespace()), identifier())
                    .map(|_| ())
                    .opt(),
                arguments(),
            )
            .map(|_| ()),
            function_expression(),
            expression_in_parentheses(),
            literal(),
            identifier(),
        )
    }

    pub fn expression_in_parentheses() -> impl Parser<()> {
        seq3(
            token_str("(", hidden_stuff_whitespace()),
            expression(),
            token_str(")", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn literal() -> impl Parser<()> {
        token_parser(
            choice6(
                null_token(),
                true_token(),
                false_token(),
                hex_number_lexical_token(),
                number_lexical_token(),
                string_lexical_token(),
            ),
            hidden_stuff_whitespace(),
        )
    }

    pub fn compound_literal() -> impl Parser<()> {
        choice2(list_literal(), map_literal())
    }

    pub fn list_literal() -> impl Parser<()> {
        seq3(
            token_str("[", hidden_stuff_whitespace()),
            seq2(
                expression_list(),
                token_str(",", hidden_stuff_whitespace()).opt(),
            )
            .map(|_| ())
            .opt(),
            token_str("]", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn map_literal() -> impl Parser<()> {
        seq3(
            token_str("{", hidden_stuff_whitespace()),
            seq3(
                map_literal_entry(),
                seq2(
                    token_str(",", hidden_stuff_whitespace()),
                    map_literal_entry(),
                )
                .map(|_| ())
                .star(),
                token_str(",", hidden_stuff_whitespace()).opt(),
            )
            .map(|_| ())
            .opt(),
            token_str("}", hidden_stuff_whitespace()),
        )
        .map(|_| ())
    }

    pub fn map_literal_entry() -> impl Parser<()> {
        seq3(
            token_parser(string_lexical_token(), hidden_stuff_whitespace()),
            token_str(":", hidden_stuff_whitespace()),
            expression(),
        )
        .map(|_| ())
    }

    pub fn function_expression() -> impl Parser<()> {
        seq4(
            return_type().opt(),
            identifier().opt(),
            formal_parameter_list(),
            function_expression_body(),
        )
        .map(|_| ())
    }

    pub fn function_declaration() -> impl Parser<()> {
        choice2(
            seq3(return_type(), identifier(), formal_parameter_list()).map(|_| ()),
            seq2(identifier(), formal_parameter_list()).map(|_| ()),
        )
    }

    pub fn function_prefix() -> impl Parser<()> {
        seq2(return_type().opt(), identifier()).map(|_| ())
    }

    pub fn function_body() -> impl Parser<()> {
        choice2(
            seq3(
                token_str("=>", hidden_stuff_whitespace()),
                expression(),
                token_str(";", hidden_stuff_whitespace()),
            )
            .map(|_| ()),
            block(),
        )
    }

    pub fn function_expression_body() -> impl Parser<()> {
        choice2(
            seq2(token_str("=>", hidden_stuff_whitespace()), expression()).map(|_| ()),
            block(),
        )
    }

    // -----------------------------------------------------------------
    // Lexical tokens.
    // -----------------------------------------------------------------
    pub fn identifier_lexical_token() -> impl Parser<()> {
        seq2(
            identifier_start_lexical_token(),
            identifier_part_lexical_token().star(),
        )
        .map(|_| ())
    }

    pub fn hex_number_lexical_token() -> impl Parser<()> {
        choice2(
            seq2(string("0x").map(|_| ()), hex_digit_lexical_token().plus()).map(|_| ()),
            seq2(string("0X").map(|_| ()), hex_digit_lexical_token().plus()).map(|_| ()),
        )
    }

    pub fn number_lexical_token() -> impl Parser<()> {
        choice2(
            seq4(
                digit_lexical_token().plus(),
                number_opt_fractional_part_lexical_token(),
                exponent_lexical_token().opt(),
                number_opt_illegal_end_lexical_token(),
            )
            .map(|_| ()),
            seq4(
                char('.').map(|_| ()),
                digit_lexical_token().plus(),
                exponent_lexical_token().opt(),
                number_opt_illegal_end_lexical_token(),
            )
            .map(|_| ()),
        )
    }

    pub fn number_opt_fractional_part_lexical_token() -> impl Parser<()> {
        choice2(
            seq2(char('.').map(|_| ()), digit_lexical_token().plus()).map(|_| ()),
            epsilon(),
        )
    }

    pub fn number_opt_illegal_end_lexical_token() -> impl Parser<()> {
        epsilon()
    }

    pub fn hex_digit_lexical_token() -> impl Parser<()> {
        pattern("0-9a-fA-F").map(|_| ())
    }

    pub fn identifier_start_lexical_token() -> impl Parser<()> {
        choice2(
            identifier_start_no_dollar_lexical_token(),
            char('$').map(|_| ()),
        )
    }

    pub fn identifier_start_no_dollar_lexical_token() -> impl Parser<()> {
        choice2(letter_lexical_token(), char('_').map(|_| ()))
    }

    pub fn identifier_part_lexical_token() -> impl Parser<()> {
        choice2(identifier_start_lexical_token(), digit_lexical_token())
    }

    pub fn letter_lexical_token() -> impl Parser<()> {
        letter().map(|_| ())
    }

    pub fn digit_lexical_token() -> impl Parser<()> {
        digit().map(|_| ())
    }

    pub fn exponent_lexical_token() -> impl Parser<()> {
        seq3(
            pattern("eE").map(|_| ()),
            pattern("+-").opt(),
            digit_lexical_token().plus(),
        )
        .map(|_| ())
    }

    pub fn string_lexical_token() -> impl Parser<()> {
        choice2(
            seq2(char('@').opt(), multi_line_string_lexical_token()).map(|_| ()),
            single_line_string_lexical_token(),
        )
    }

    pub fn multi_line_string_lexical_token() -> impl Parser<()> {
        choice2(
            seq3(
                string("\"\"\"").map(|_| ()),
                any().star_lazy(string("\"\"\"")).map(|_| ()),
                string("\"\"\"").map(|_| ()),
            )
            .map(|_| ()),
            seq3(
                string("'''").map(|_| ()),
                any().star_lazy(string("'''")).map(|_| ()),
                string("'''").map(|_| ()),
            )
            .map(|_| ()),
        )
    }

    pub fn single_line_string_lexical_token() -> impl Parser<()> {
        choice4(
            seq3(
                char('"').map(|_| ()),
                string_content_double_quoted_lexical_token().star(),
                char('"').map(|_| ()),
            )
            .map(|_| ()),
            seq3(
                char('\'').map(|_| ()),
                string_content_single_quoted_lexical_token().star(),
                char('\'').map(|_| ()),
            )
            .map(|_| ()),
            seq3(
                string("@\"").map(|_| ()),
                pattern("^\"\n\r").star(),
                char('"').map(|_| ()),
            )
            .map(|_| ()),
            seq3(
                string("@'").map(|_| ()),
                pattern("^'\n\r").star(),
                char('\'').map(|_| ()),
            )
            .map(|_| ()),
        )
    }

    pub fn string_content_double_quoted_lexical_token() -> impl Parser<()> {
        choice2(
            pattern("^\\\"\n\r").map(|_| ()),
            seq2(char('\\').map(|_| ()), pattern("\n\r").map(|_| ())).map(|_| ()),
        )
    }

    pub fn string_content_single_quoted_lexical_token() -> impl Parser<()> {
        choice2(
            pattern("^\\'\n\r").map(|_| ()),
            seq2(char('\\').map(|_| ()), pattern("\n\r").map(|_| ())).map(|_| ()),
        )
    }

    pub fn newline_lexical_token() -> impl Parser<()> {
        pattern("\n\r").map(|_| ())
    }

    pub fn hashbang_lexical_token() -> impl Parser<()> {
        seq3(
            string("#!").map(|_| ()),
            pattern("^\n\r").star(),
            newline_lexical_token().opt(),
        )
        .map(|_| ())
    }

    // -----------------------------------------------------------------
    // Whitespace and comments.
    // -----------------------------------------------------------------
    pub fn hidden_whitespace() -> impl Parser<()> {
        hidden_stuff_whitespace().plus().map(|_| ())
    }

    pub fn hidden_stuff_whitespace() -> impl Parser<()> {
        choice3(
            visible_whitespace(),
            single_line_comment(),
            multi_line_comment(),
        )
    }

    pub fn visible_whitespace() -> impl Parser<()> {
        whitespace().map(|_| ())
    }

    pub fn single_line_comment() -> impl Parser<()> {
        seq3(
            string("//").map(|_| ()),
            newline_lexical_token().neg().star(),
            newline_lexical_token().opt(),
        )
        .map(|_| ())
    }

    pub fn multi_line_comment() -> impl Parser<()> {
        seq3(
            string("/*").map(|_| ()),
            choice2(multi_line_comment(), string("*/").neg().map(|_| ())).star(),
            string("*/").map(|_| ()),
        )
        .map(|_| ())
    }
}

fn assert_failure(p: impl Parser<()>, input: &str) {
    assert!(p.parse(input).is_err(), "expected failure for {input:?}");
}

#[gtest]
fn directives_hashbang() {
    let g = DartGrammar::new();
    assert_success!(g, "#!/bin/dart\n", ());
}

#[gtest]
fn directives_library() {
    let g = DartGrammar::new();
    assert_success!(g, "library a;", ());
    assert_success!(g, "library a.b;", ());
    assert_success!(g, "library a.b.c_d;", ());
}

#[gtest]
fn directives_part_of() {
    let g = DartGrammar::new();
    assert_success!(g, "part of a;", ());
    assert_success!(g, "part of a.b;", ());
    assert_success!(g, "part of a.b.c_d;", ());
}

#[gtest]
fn directives_part() {
    let g = DartGrammar::new();
    assert_success!(g, "part \"abc\";", ());
}

#[gtest]
fn directives_import() {
    let g = DartGrammar::new();
    assert_success!(g, "import \"abc\";", ());
    assert_success!(g, "import \"abc\" deferred;", ());
    assert_success!(g, "import \"abc\" as a;", ());
    assert_success!(g, "import \"abc\" deferred as a;", ());
    assert_success!(g, "import \"abc\" show a;", ());
    assert_success!(g, "import \"abc\" deferred show a, b;", ());
    assert_success!(g, "import \"abc\" hide a;", ());
    assert_success!(g, "import \"abc\" deferred hide a, b;", ());
}

#[gtest]
fn directives_export() {
    let g = DartGrammar::new();
    assert_success!(g, "export \"abc\";", ());
    assert_success!(g, "export \"abc\" show a;", ());
    assert_success!(g, "export \"abc\" show a, b;", ());
    assert_success!(g, "export \"abc\" hide a;", ());
    assert_success!(g, "export \"abc\" hide a, b;", ());
}

#[gtest]
fn directives_full() {
    let g = DartGrammar::new();
    assert_success!(g, "library test;", ());
    assert_success!(g, "library test; void main() { }", ());
    assert_success!(g, "library test; void main() { print(2 + 3); }", ());
}

#[gtest]
fn expression_literal_numbers() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "1", ());
    assert_success!(p, "1.2", ());
    assert_success!(p, "1.2e3", ());
    assert_success!(p, "1.2e-3", ());
    assert_success!(p, "-1.2e3", ());
    assert_success!(p, "-1.2e-3", ());
    assert_success!(p, "-1.2E-3", ());
}

#[gtest]
fn expression_literal_objects() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "true", ());
    assert_success!(p, "false", ());
    assert_success!(p, "null", ());
}

#[gtest]
fn expression_literal_array() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "[]", ());
    assert_success!(p, "[a]", ());
    assert_success!(p, "[a, b]", ());
    assert_success!(p, "[a, b, c]", ());
}

#[gtest]
fn expression_literal_map() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "{}", ());
    assert_success!(p, "{\"a\": b}", ());
    assert_success!(p, "{\"a\": b, \"c\": d}", ());
    assert_success!(p, "{\"a\": b, \"c\": d, \"e\": f}", ());
}

#[gtest]
fn expression_literal_nested() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "[1, true, [1], {\"a\": b}]", ());
    assert_success!(
        p,
        "{\"a\": 1, \"b\": true, \"c\": [1], \"d\": {\"a\": b}}",
        ()
    );
}

#[gtest]
fn expression_conditional() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a ? b : c", ());
    assert_success!(p, "a ? b ? c : d : c", ());
}

#[gtest]
fn expression_relational() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a is b", ());
    assert_success!(p, "a is !b", ());
}

#[gtest]
fn expression_unary_increment_decrement() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "++a", ());
    assert_success!(p, "--a", ());
    assert_success!(p, "a++", ());
    assert_success!(p, "a--", ());
}

#[gtest]
fn expression_unary_operators() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "+a", ());
    assert_success!(p, "-a", ());
    assert_success!(p, "!a", ());
    assert_success!(p, "~a", ());
}

#[gtest]
fn expression_binary_arithmetic_operators() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a + b", ());
    assert_success!(p, "a - b", ());
    assert_success!(p, "a * b", ());
    assert_success!(p, "a / b", ());
    assert_success!(p, "a ~/ b", ());
    assert_success!(p, "a % b", ());
}

#[gtest]
fn expression_binary_logical_operators() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a & b", ());
    assert_success!(p, "a | b", ());
    assert_success!(p, "a ^ b", ());
    assert_success!(p, "a && b", ());
    assert_success!(p, "a || b", ());
}

#[gtest]
fn expression_binary_conditional_operators() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a > b", ());
    assert_success!(p, "a >= b", ());
    assert_success!(p, "a < b", ());
    assert_success!(p, "a <= b", ());
    assert_success!(p, "a == b", ());
    assert_success!(p, "a != b", ());
    assert_success!(p, "a === b", ());
    assert_success!(p, "a !== b", ());
}

#[gtest]
fn expression_binary_shift_operators() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a << b", ());
    assert_success!(p, "a >>> b", ());
    assert_success!(p, "a >> b", ());
}

#[gtest]
fn expression_parenthesis() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "(a + b)", ());
    assert_success!(p, "a * (b + c)", ());
    assert_success!(p, "(a * b) + c", ());
}

#[gtest]
fn expression_access() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a.b", ());
    assert_success!(p, "a.b.c", ());
    assert_success!(p, "a?.b", ());
    assert_success!(p, "a.b?", ());
    assert_success!(p, "a.b!", ());
    assert_failure(g.expression().end(), "?.a.b");
    assert_failure(g.expression().end(), "?a.b");
    assert_failure(g.expression().end(), "a?b");
    assert_failure(g.expression().end(), "a!b");
    assert_failure(g.expression().end(), "a?.?b");
}

#[gtest]
fn expression_invoke() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a()", ());
    assert_success!(p, "a(b)", ());
    assert_success!(p, "a(b, c)", ());
    assert_success!(p, "a(b: c)", ());
    assert_success!(p, "a(b: c!.d)", ());
    assert_success!(p, "a(b: c?.d)", ());
    assert_success!(p, "a(b: c, d: e)", ());
    assert_success!(p, "a(b: c, d: e,)", ());
    assert_success!(p, "b()!", ());
    assert_success!(p, "a.b()?", ());
    assert_success!(p, "a?.b()", ());
    assert_failure(g.expression().end(), "a?()");
}

#[gtest]
fn expression_invoke_double() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a()()", ());
    assert_success!(p, "a(b)(b)", ());
    assert_success!(p, "a(b, c)(b, c)", ());
    assert_success!(p, "a(b: c)(b: c)", ());
    assert_success!(p, "a(b: c, d: e)(b: c, d: e)", ());
    assert_success!(p, "a(b: c, d: e,)(b: c, d: e,)", ());
}

#[gtest]
fn expression_constructor() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "new a()", ());
    assert_success!(p, "const a()", ());
    assert_success!(p, "new a<b>()", ());
    assert_success!(p, "const a<b>()", ());
    assert_success!(p, "new a.b()", ());
    assert_success!(p, "const a.b()", ());
}

#[gtest]
fn expression_function_expression() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "() => a", ());
    assert_success!(p, "a() => b", ());
    assert_success!(p, "a () => b", ());
    assert_success!(p, "a b() => c", ());
    assert_success!(p, "a (b) => c", ());
    assert_success!(p, "a b(c) => d", ());
}

#[gtest]
fn expression_function_block() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "() {}", ());
    assert_success!(p, "a() {}", ());
    assert_success!(p, "a () {}", ());
    assert_success!(p, "a b() {}", ());
    assert_success!(p, "a (b) {}", ());
    assert_success!(p, "a b(c) {}", ());
}

#[gtest]
fn expression_assignment() {
    let g = DartGrammar::new();
    let p = g.expression().end();
    assert_success!(p, "a = b", ());
    assert_success!(p, "a += b", ());
    assert_success!(p, "a -= b", ());
    assert_success!(p, "a *= b", ());
    assert_success!(p, "a /= b", ());
    assert_success!(p, "a %= b", ());
    assert_success!(p, "a ~/= b", ());
    assert_success!(p, "a <<= b", ());
    assert_success!(p, "a >>>= b", ());
    assert_success!(p, "a >>= b", ());
    assert_success!(p, "a &= b", ());
    assert_success!(p, "a ^= b", ());
    assert_success!(p, "a |= b", ());
}

#[gtest]
fn statement_label() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "a: {}", ());
    assert_success!(p, "a: b: {}", ());
    assert_success!(p, "a: b: c: {}", ());
}

#[gtest]
fn statement_block() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "{}", ());
    assert_success!(p, "{{}}", ());
}

#[gtest]
fn statement_declaration() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "var a;", ());
    assert_success!(p, "final a;", ());
}

#[gtest]
fn statement_declaration_initialized() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "var a = b;", ());
    assert_success!(p, "final a = b;", ());
}

#[gtest]
fn statement_declaration_typed() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "a b;", ());
    assert_success!(p, "final a b;", ());
}

#[gtest]
fn statement_declaration_typed_initialized() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "a b = c;", ());
    assert_success!(p, "final a b = c;", ());
}

#[gtest]
fn statement_while() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "while (a) {}", ());
}

#[gtest]
fn statement_do() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "do {} while (b);", ());
}

#[gtest]
fn statement_for() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "for (;;) {}", ());
    assert_success!(p, "for (var a = b; c; d++) {}", ());
    assert_success!(p, "for (var a = b, c = d; e; f++) {}", ());
    assert_success!(p, "for (a in b) {}", ());
}

#[gtest]
fn statement_if() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "if (a) {}", ());
    assert_success!(p, "if (a) {} else {}", ());
    assert_success!(p, "if (a) {} else if (b) {}", ());
    assert_success!(p, "if (a) {} else if (b) {} else {}", ());
}

#[gtest]
fn statement_switch() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "switch (a) {}", ());
    assert_success!(p, "switch (a) { case b: {} }", ());
    assert_success!(p, "switch (a) { case b: {} case d: {}}", ());
    assert_success!(p, "switch (a) { case b: {} default: {}}", ());
}

#[gtest]
fn statement_try() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "try {} finally {}", ());
    assert_success!(p, "try {} catch (a b) {}", ());
    assert_success!(p, "try {} catch (a b, c d) {}", ());
    assert_success!(p, "try {} catch (a b) {} finally {}", ());
    assert_success!(p, "try {} catch (a b, c d) {} finally {}", ());
    assert_success!(p, "try {} catch (a b) {} catch (c d) {}", ());
    assert_success!(p, "try {} catch (a b) {} catch (c d) {} finally {}", ());
}

#[gtest]
fn statement_break() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "break;", ());
    assert_success!(p, "break a;", ());
}

#[gtest]
fn statement_continue() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "continue;", ());
    assert_success!(p, "continue a;", ());
}

#[gtest]
fn statement_return() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "return;", ());
    assert_success!(p, "return b;", ());
}

#[gtest]
fn statement_throw() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "throw;", ());
    assert_success!(p, "throw b;", ());
}

#[gtest]
fn statement_expression() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "a;", ());
    assert_success!(p, "a + b;", ());
}

#[gtest]
fn statement_assert() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "assert(a);", ());
}

#[gtest]
fn statement_invocation() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "a();", ());
    assert_success!(p, "a(b);", ());
    assert_success!(p, "a(b, c);", ());
    assert_success!(p, "a(b, c, d);", ());
}

#[gtest]
fn statement_invocation_named() {
    let g = DartGrammar::new();
    let p = g.statement().end();
    assert_success!(p, "a(b: c);", ());
    assert_success!(p, "a(b: c, d: e);", ());
    assert_success!(p, "a(b: c, d: e, f: g);", ());
}

#[gtest]
fn member_function() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a() {}", ());
    assert_success!(p, "a b() {}", ());
}

#[gtest]
fn member_function_abstract() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "abstract a();", ());
    assert_success!(p, "abstract a b();", ());
}

#[gtest]
fn member_function_static() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "static a() {}", ());
    assert_success!(p, "static a b() {}", ());
}

#[gtest]
fn member_function_expression() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a() => b;", ());
    assert_success!(p, "a b() => c;", ());
}

#[gtest]
fn member_function_arguments_plain() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a() {}", ());
    assert_success!(p, "a(b) {}", ());
    assert_success!(p, "a(b, c) {}", ());
    assert_success!(p, "a(b, c, d) {}", ());
}

#[gtest]
fn member_function_arguments_optional() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a([b]) {}", ());
    assert_success!(p, "a([b, c]) {}", ());
    assert_success!(p, "a(b, [c, d]) {}", ());
    assert_success!(p, "a(b, c, [d, e]) {}", ());
}

#[gtest]
fn member_function_arguments_optional_defaults() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a([b = c]) {}", ());
    assert_success!(p, "a([b = c, d = e]) {}", ());
    assert_success!(p, "a(b, [c = d, e = f]) {}", ());
    assert_success!(p, "a(b, c, [d = e, f = g]) {}", ());
}

#[gtest]
fn member_function_arguments_named() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a({b}) {}", ());
    assert_success!(p, "a({b, c}) {}", ());
    assert_success!(p, "a(b, {c, d}) {}", ());
    assert_success!(p, "a(b, c, {d, e}) {}", ());
}

#[gtest]
fn member_function_arguments_named_defaults() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "a({b: c}) {}", ());
    assert_success!(p, "a({b: c, d: e}) {}", ());
    assert_success!(p, "a(b, {c: d, e: f}) {}", ());
    assert_success!(p, "a(b, c, {d: e, f: g}) {}", ());
}

#[gtest]
fn member_constructor() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "A();", ());
    assert_success!(p, "A() {}", ());
    assert_success!(p, "A() : super();", ());
    assert_success!(p, "A() : super() {}", ());
    assert_success!(p, "A() : super(), a = b;", ());
    assert_success!(p, "A() : super(), a = b {}", ());
    assert_success!(p, "A() : super(), a = b, c = d;", ());
    assert_success!(p, "A() : super(), a = b, c = d {}", ());
}

#[gtest]
fn member_constructor_field() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "A(this.a);", ());
    assert_success!(p, "A(this.a) {}", ());
    assert_success!(p, "A(this.a, this.b);", ());
    assert_success!(p, "A(this.a, this.b) {}", ());
}

#[gtest]
fn member_constructor_const() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "const A();", ());
    assert_success!(p, "const A._();", ());
}

#[gtest]
fn member_constructor_named() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "A._() {}", ());
    assert_success!(p, "A._() : super();", ());
    assert_success!(p, "A._() : super() {}", ());
    assert_success!(p, "A._() : super(), a = b;", ());
    assert_success!(p, "A._() : super(), a = b {}", ());
    assert_success!(p, "A._() : super(), a = b, c = d;", ());
    assert_success!(p, "A._() : super(), a = b, c = d {}", ());
}

#[gtest]
fn member_constructor_factory() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "factory A() {}", ());
}

#[gtest]
fn member_constructor_factory_named() {
    let g = DartGrammar::new();
    let p = g.class_member_definition().end();
    assert_success!(p, "factory A._() {}", ());
}

#[gtest]
fn definition_class() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "class A {}", ());
    assert_success!(p, "class A extends B {}", ());
    assert_success!(p, "class A implements B {}", ());
    assert_success!(p, "class A implements B, C {}", ());
    assert_success!(p, "class A extends B implements C {}", ());
    assert_success!(p, "class A extends B implements C, D {}", ());
}

#[gtest]
fn definition_class_typed() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "class A<T> {}", ());
    assert_success!(p, "class A<T> extends B<T> {}", ());
    assert_success!(p, "class A<T> implements B<T> {}", ());
    assert_success!(p, "class A<T> implements B<T>, C<T> {}", ());
    assert_success!(p, "class A<T> extends B<T> implements C<T> {}", ());
    assert_success!(p, "class A<T> extends B<T> implements C<T>, D<T> {}", ());
}

#[gtest]
fn definition_class_abstract() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "abstract class A {}", ());
    assert_success!(p, "abstract class A extends B {}", ());
    assert_success!(p, "abstract class A implements B {}", ());
    assert_success!(p, "abstract class A implements B, C {}", ());
    assert_success!(p, "abstract class A extends B implements C {}", ());
    assert_success!(p, "abstract class A extends B implements C, D {}", ());
}

#[gtest]
fn definition_typedef() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "typedef a b();", ());
    assert_success!(p, "typedef a b(c);", ());
    assert_success!(p, "typedef a b(c d);", ());
}

#[gtest]
fn definition_typedef_typed() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "typedef a b<T>();", ());
    assert_success!(p, "typedef a b<T>(c);", ());
    assert_success!(p, "typedef a b<T>(c d);", ());
}

#[gtest]
fn definition_final() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "final a = 0;", ());
    assert_success!(p, "final a b = 0;", ());
}

#[gtest]
fn definition_const() {
    let g = DartGrammar::new();
    let p = g.top_level_definition().end();
    assert_success!(p, "const a = 0;", ());
    assert_success!(p, "const a b = 0;", ());
}

#[gtest]
fn whitespace_whitespace() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, " ", ());
    assert_success!(p, "\t", ());
    assert_success!(p, "\n", ());
    assert_success!(p, "\r", ());
    assert_failure(g.hidden_whitespace().end(), "a");
}

#[gtest]
fn whitespace_single_line_comment() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, "//", ());
    assert_success!(p, "// foo", ());
    assert_success!(p, "//\n", ());
    assert_success!(p, "// foo\n", ());
}

#[gtest]
fn whitespace_single_line_documentation() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, "///", ());
    assert_success!(p, "/// foo", ());
    assert_success!(p, "/// \n", ());
    assert_success!(p, "/// foo\n", ());
}

#[gtest]
fn whitespace_multi_line_comment() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, "/**/", ());
    assert_success!(p, "/* foo */", ());
    assert_success!(p, "/* foo \n bar */", ());
    assert_success!(p, "/* foo ** bar */", ());
    assert_success!(p, "/* foo * / bar */", ());
}

#[gtest]
fn whitespace_multi_line_documentation() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, "/***/", ());
    assert_success!(p, "/*******/", ());
    assert_success!(p, "/** foo */", ());
    assert_success!(p, "/**\n *\n *\n */", ());
}

#[gtest]
fn whitespace_multi_line_nested() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, "/* outer /* nested */ */", ());
    assert_success!(p, "/* outer /* nested /* deeply nested */ */ */", ());
    assert_failure(g.hidden_whitespace().end(), "/* outer /* not closed */");
}

#[gtest]
fn whitespace_combined() {
    let g = DartGrammar::new();
    let p = g.hidden_whitespace().end();
    assert_success!(p, "/**/", ());
    assert_success!(p, " /**/", ());
    assert_success!(p, "/**/ ", ());
    assert_success!(p, " /**/ ", ());
    assert_success!(p, "/**///", ());
    assert_success!(p, "/**/ //", ());
    assert_success!(p, " /**/ //", ());
}

#[gtest]
fn child_parsers_single_line_string() {
    let g = DartGrammar::new();
    let p = g.string_lexical_token().end();
    assert_success!(p, "'hi'", ());
    assert_success!(p, "\"hi\"", ());
    assert_failure(g.string_lexical_token().end(), "no quotes");
    assert_failure(g.string_lexical_token().end(), "\"missing quote");
    assert_failure(g.string_lexical_token().end(), "'missing quote");
}

#[gtest]
fn official_identifier() {
    let g = DartGrammar::new();
    let p = g.identifier().end();
    assert_success!(p, "foo", ());
    assert_success!(p, "bar9", ());
    assert_success!(p, "dollar$", ());
    assert_success!(p, "_foo", ());
    assert_success!(p, "_bar9", ());
    assert_success!(p, "_dollar$", ());
    assert_success!(p, "$", ());
    assert_success!(p, " leadingSpace", ());
    assert_failure(g.identifier().end(), "9");
    assert_failure(g.identifier().end(), "3foo");
    assert_failure(g.identifier().end(), "");
}

#[gtest]
fn official_numeric_literal() {
    let g = DartGrammar::new();
    let p = g.literal().end();
    assert_success!(p, "0", ());
    assert_success!(p, "1984", ());
    assert_success!(p, " 1984", ());
    assert_success!(p, "0xCAFE", ());
    assert_success!(p, "0XCAFE", ());
    assert_success!(p, "0xcafe", ());
    assert_success!(p, "0Xcafe", ());
    assert_success!(p, "0xCaFe", ());
    assert_success!(p, "0XCaFe", ());
    assert_success!(p, "3e4", ());
    assert_success!(p, "3e-4", ());
    assert_success!(p, "3E4", ());
    assert_success!(p, "3E-4", ());
    assert_success!(p, "3.14E4", ());
    assert_success!(p, "3.14E-4", ());
    assert_success!(p, "3.14", ());
    assert_failure(g.literal().end(), "3e--4");
    assert_failure(g.literal().end(), "5.");
    assert_failure(g.literal().end(), "CAFE");
    assert_failure(g.literal().end(), "0xGHIJ");
    assert_failure(g.literal().end(), "-");
    assert_failure(g.literal().end(), "");
}

#[gtest]
fn official_boolean_literal() {
    let g = DartGrammar::new();
    let p = g.literal().end();
    assert_success!(p, "true", ());
    assert_success!(p, "false", ());
    assert_success!(p, " true", ());
    assert_success!(p, " false", ());
}
