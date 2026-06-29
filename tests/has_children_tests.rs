//! Tests for the `HasChildren` reflection foundation: every parser reports its immediate
//! sub-parsers, and a tree-walker can recurse through an acyclic parser graph.

use googletest::prelude::*;
use rust_petitparser::prelude::*;
use rust_petitparser_macros::grammar;
use std::rc::Rc;

/// Recursively counts nodes in an *acyclic* parser graph. (A recursive grammar's graph has
/// cycles via `SettableParserRef`, so a real linter would track visited nodes by `Rc`
/// identity; these tests only walk acyclic parsers.)
fn count_nodes(node: &Rc<dyn HasChildren>) -> usize {
    1 + node.children().iter().map(count_nodes).sum::<usize>()
}

#[gtest]
fn leaf_parsers_have_no_children() {
    assert_that!(digit().children().len(), eq(0));
    assert_that!(char('a').children().len(), eq(0));
    assert_that!(string("hi").children().len(), eq(0));
    assert_that!(epsilon().children().len(), eq(0));
}

#[gtest]
fn single_delegate_combinators_report_one_child() {
    assert_that!(digit().map(|c| c).children().len(), eq(1));
    assert_that!(digit().star().children().len(), eq(1));
    assert_that!(digit().token().children().len(), eq(1));
}

#[gtest]
fn seq_and_choice_report_every_branch() {
    assert_that!(
        seq3(char('a'), char('b'), char('c')).children().len(),
        eq(3)
    );
    assert_that!(choice2(char('a'), char('b')).children().len(), eq(2));
}

#[gtest]
fn skip_reports_before_delegate_and_after() {
    // skip(before, after) wraps the receiver between two delimiters: 3 children.
    let p = digit().skip(char('('), char(')'));
    assert_that!(p.children().len(), eq(3));
}

#[gtest]
fn walks_a_full_acyclic_tree() {
    // token( map( seq2(char, char) ) ): token + map + seq2 + 2 chars = 5 nodes.
    let p: Rc<dyn HasChildren> = Rc::new(seq2(char('a'), char('b')).map(|t| t).token());
    assert_that!(count_nodes(&p), eq(5));
}

#[gtest]
fn settable_reports_its_current_delegate() {
    // An undefined settable still has exactly one child: the sentinel UndefinedParser.
    let s: SettableParser<char> = SettableParser::undefined();
    assert_that!(s.children().len(), eq(1));
}

#[grammar]
mod tiny_grammar {
    pub fn start() -> impl Parser<char> {
        atom()
    }
    fn atom() -> impl Parser<char> {
        char('a')
    }
}

#[gtest]
fn grammar_root_child_is_its_start_rule() {
    let g = TinyGrammar::new();
    // The grammar's single immediate child is the start rule; walking from there reaches the
    // rest of the (here acyclic) graph.
    assert_that!(g.children().len(), eq(1));
    let root: Rc<dyn HasChildren> = Rc::new(g);
    // start (SettableParser) -> body ref (SettableParserRef) -> atom (SettableParser)
    //   -> char('a'); plus the grammar node itself.
    assert_that!(count_nodes(&root), ge(4));
}
