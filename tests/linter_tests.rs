//! Tests for the reflection / linter infrastructure.
//!
//! Covers:
//! - `Analyzer` graph traversal (parsers collection, deduplication)
//! - `Analyzer::is_nullable` (simple and composite grammars)
//! - `Analyzer::first_set` (count- and nullability-based assertions)
//! - `Analyzer::cycle_set` (acyclic → empty, cyclic → non-empty)
//! - `linter()` runner and individual rules: `NullableRepeater`, `UnresolvedSettable`

use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::rc::Rc;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn assert_no_issues(root: Rc<dyn HasChildren>, rules: &[&dyn LinterRule]) {
    let issues = linter(root, rules);
    assert!(
        issues.is_empty(),
        "expected no issues but got: {:?}",
        issues.iter().map(|i| i.title.as_str()).collect::<Vec<_>>()
    );
}

fn assert_one_issue(root: Rc<dyn HasChildren>, rules: &[&dyn LinterRule], expected_title: &str) {
    let issues = linter(root, rules);
    assert_eq!(
        issues.len(),
        1,
        "expected 1 issue ({expected_title}) but got {}: {:?}",
        issues.len(),
        issues.iter().map(|i| i.title.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(issues[0].title, expected_title);
}

// ─────────────────────────────────────────────────────────────────────────────
// Graph traversal — Analyzer::parsers
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn parsers_single_leaf() {
    let p: Rc<dyn HasChildren> = Rc::new(char('a'));
    let a = Analyzer::new(p);
    assert_that!(a.parsers.len(), eq(1));
}

#[gtest]
fn parsers_nested() {
    // star(char('a')) → 2 nodes: the star + char('a')
    let p: Rc<dyn HasChildren> = Rc::new(char('a').star());
    let a = Analyzer::new(p);
    assert_that!(a.parsers.len(), eq(2));
}

#[gtest]
fn parsers_branched() {
    // choice2(char('a'), char('b')) → 3 nodes
    let p: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')));
    let a = Analyzer::new(p);
    assert_that!(a.parsers.len(), eq(3));
}

#[gtest]
fn parsers_raw_rc_share_does_not_dedupe() {
    // seq2(a, a) where a is the same Rc. seq2's macro wraps each argument in Rc::new(),
    // so even though both args come from the same Rc clone, they end up as two distinct
    // outer allocations (each Rc<Rc<dyn Parser<char>>> is a fresh pointer). DFS sees 3
    // distinct nodes: seq2 + the two distinct outer Rcs. True structural sharing only
    // happens with SettableParser/.borrow() weak-ref cycles, not seq2/choice2 arguments.
    //
    // This is a real divergence from dart's `allParser` 'duplicated' test
    // (`parser2.seq(parser2)` -> {parser1, parser2}, 2 nodes, not 3) -- dart's Parser
    // objects are plain object references, so reusing the same reference twice is
    // structural sharing for free; our Rc<dyn Parser<T>> delegates get re-boxed by every
    // combinator constructor, which defeats it. See `parsers_grammar_shared_rule_dedupes`
    // below for the idiom that *does* dedupe (SettableParser/.borrow(), as used by
    // `#[grammar]`).
    let a: Rc<dyn Parser<char>> = Rc::new(char('a'));
    let p: Rc<dyn HasChildren> = Rc::new(seq2(a.clone(), a.clone()));
    let a = Analyzer::new(p);
    assert_that!(a.parsers.len(), eq(3));
}

#[grammar]
mod shared_rule_grammar {
    // `shared()` is called twice from `start()`'s body; the #[grammar] macro rewrites both
    // call sites to `shared.borrow()` (a SettableParserRef weak ref into the same
    // SettableParser slot), not a fresh `Rc::new()` wrap of the parser itself -- so unlike
    // `seq2(a.clone(), a.clone())` above, the *underlying* `char('a')` leaf is the same Rc
    // both times.
    pub fn start() -> impl Parser<(char, char)> {
        seq2(shared(), shared())
    }
    fn shared() -> impl Parser<char> {
        char('a')
    }
}

#[gtest]
fn parsers_grammar_shared_rule_dedupes() {
    // Mirrors dart's `allParser` 'duplicated' test intent: a rule referenced twice should
    // contribute its leaf parser to the graph exactly once, not once per call site. The two
    // `shared.borrow()` call sites still produce two distinct SettableParserRef wrapper
    // nodes (an unavoidable Weak-ref indirection cost, same as the self-referential case
    // above) -- but both resolve through the same underlying SettableParser slot, so the
    // `char('a')` leaf itself is not duplicated.
    let g = SharedRuleGrammar::new();
    let root: Rc<dyn HasChildren> = Rc::new(g);
    let analyzer = Analyzer::new(root);
    let char_leaf_count = analyzer
        .parsers
        .iter()
        .filter(|p| {
            matches!(
                p.kind(),
                ParserKind::Char { .. } | ParserKind::PredicateChar { .. }
            )
        })
        .count();
    assert_that!(char_leaf_count, eq(1));
}

#[gtest]
fn parsers_self_referential_terminates_without_infinite_loop() {
    // s.set(s.borrow()): SettableParser whose delegate is its own weak ref.
    // The seen-set stops the DFS from looping.
    // Yields 2 nodes: SettableParser (root) + SettableParserRef (the borrow).
    let mut s: SettableParser<()> = SettableParser::undefined();
    let s_ref = s.borrow();
    s.set(s_ref);
    let root: Rc<dyn HasChildren> = Rc::new(s);
    let a = Analyzer::new(root);
    assert_that!(a.parsers.len(), eq(2));
}

// ─────────────────────────────────────────────────────────────────────────────
// is_nullable — simple parsers
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn is_nullable_char_is_false() {
    let p: Rc<dyn HasChildren> = Rc::new(char('a'));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn is_nullable_epsilon_is_true() {
    let p: Rc<dyn HasChildren> = Rc::new(epsilon());
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_true());
}

#[gtest]
fn is_nullable_plus_is_false() {
    let p: Rc<dyn HasChildren> = Rc::new(char('a').plus());
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn is_nullable_star_is_true() {
    let p: Rc<dyn HasChildren> = Rc::new(char('a').star());
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_true());
}

#[gtest]
fn is_nullable_opt_is_true() {
    // opt() = rep(0, Some(1)) — min == 0 → is_directly_nullable
    let p: Rc<dyn HasChildren> = Rc::new(char('a').rep(0, Some(1)));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_true());
}

#[gtest]
fn is_nullable_choice_of_non_nullables_is_false() {
    let p: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn is_nullable_choice_with_epsilon_is_true() {
    let p: Rc<dyn HasChildren> = Rc::new(choice2(char('a').constant(()), epsilon()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_true());
}

#[gtest]
fn is_nullable_sequence_of_non_nullables_is_false() {
    let p: Rc<dyn HasChildren> = Rc::new(seq2(char('a'), char('b')).map(|_| ()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn is_nullable_sequence_nullable_first_nonnullable_second_is_false() {
    // seq2(epsilon, char('a')): first is nullable but second is not → not nullable
    let p: Rc<dyn HasChildren> =
        Rc::new(seq2(epsilon().map(|_| ()), char('a').constant(())).map(|_| ()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn is_nullable_sequence_both_nullable_is_true() {
    // seq2(epsilon, star(char('a'))): both children nullable → sequence is nullable
    let p: Rc<dyn HasChildren> =
        Rc::new(seq2(epsilon().map(|_| ()), char('a').star().map(|_| ())).map(|_| ()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.is_nullable(&p), is_true());
}

// ─────────────────────────────────────────────────────────────────────────────
// is_nullable — composite / transitive
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn is_nullable_transitive_through_choice() {
    // B = b | epsilon → nullable; A = a | B → nullable transitively via B.
    //
    // NOTE: seq2/choice2 wrap every argument in Rc::new(), so passing a
    // Rc<dyn Parser<()>> creates a new wrapper allocation. The node for B
    // that the Analyzer actually stores is that wrapper, NOT cap_b's own
    // allocation — so we find parsers by traversing analyzer.parsers rather
    // than by looking up the original Rc handles.
    let b_char = char('b').constant(());
    let e = epsilon();
    let cap_b = choice2(b_char, e); // Choice2<()>: nullable via epsilon branch
    let a_char = char('a').constant(());
    let cap_a = choice2(a_char, cap_b); // Choice2<()>: nullable via cap_b branch

    let root: Rc<dyn HasChildren> = Rc::new(cap_a);
    let analyzer = Analyzer::new(root.clone());

    // Root (cap_a) is directly in parsers[0] and can be looked up by its ptr.
    assert_that!(analyzer.is_nullable(&root), is_true());

    // Find the inner cap_b node by looking it up in analyzer.parsers: it should
    // be a Choice node (is_choice) among the non-root parsers that is nullable.
    let nullable_inner_choices: Vec<_> = analyzer
        .parsers
        .iter()
        .skip(1)
        .filter(|p| matches!(p.kind(), ParserKind::Choice { .. }) && analyzer.is_nullable(p))
        .collect();
    assert_that!(nullable_inner_choices.len(), eq(1));
}

#[gtest]
fn is_nullable_sequence_stops_propagating_at_nonnullable_child() {
    // Übersetzerbau-style: S = A B c d, A = a | B, B = b | ε.
    // A and B are nullable but c is not → S is not nullable.
    let b_char: Rc<dyn Parser<()>> = Rc::new(char('b').constant(()));
    let c_char: Rc<dyn Parser<()>> = Rc::new(char('c').constant(()));
    let d_char: Rc<dyn Parser<()>> = Rc::new(char('d').constant(()));
    let a_char: Rc<dyn Parser<()>> = Rc::new(char('a').constant(()));
    let e: Rc<dyn Parser<()>> = Rc::new(epsilon());
    let cap_b: Rc<dyn Parser<()>> = Rc::new(choice2(b_char, e));
    let cap_a: Rc<dyn Parser<()>> = Rc::new(choice2(a_char, cap_b.clone()));
    let s: Rc<dyn Parser<()>> =
        Rc::new(seq4(cap_a.clone(), cap_b.clone(), c_char, d_char).map(|_| ()));

    let s_hc: Rc<dyn HasChildren> = s.clone();
    let analyzer = Analyzer::new(s_hc.clone());
    assert_that!(analyzer.is_nullable(&s_hc), is_false());
}

// ─────────────────────────────────────────────────────────────────────────────
// first_set — count- and nullability-based assertions
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn first_set_leaf_has_one_terminal_and_is_not_nullable() {
    let p: Rc<dyn HasChildren> = Rc::new(char('a'));
    let a = Analyzer::new(p.clone());
    assert_that!(a.first_set(&p).len(), eq(1));
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn first_set_star_has_one_terminal_and_is_nullable() {
    // star(char('a')): first_set = {char('a')}, nullable because min == 0
    let p: Rc<dyn HasChildren> = Rc::new(char('a').star());
    let a = Analyzer::new(p.clone());
    assert_that!(a.first_set(&p).len(), eq(1));
    assert_that!(a.is_nullable(&p), is_true());
}

#[gtest]
fn first_set_choice_accumulates_both_branches() {
    let p: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')));
    let a = Analyzer::new(p.clone());
    assert_that!(a.first_set(&p).len(), eq(2));
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn first_set_choice_with_epsilon_is_nullable_one_terminal() {
    // choice2(char('a').constant(()), epsilon()): epsilon is a leaf, so it gets added
    // to its own first_set as a "start parser" (just like char), AND contributes the
    // nullable sentinel. When propagated into the choice, both char('a').constant(())
    // and epsilon() appear as non-sentinel terminals → 2 entries. The choice is also
    // nullable because of the epsilon branch.
    let p: Rc<dyn HasChildren> = Rc::new(choice2(char('a').constant(()), epsilon()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.first_set(&p).len(), eq(2));
    assert_that!(a.is_nullable(&p), is_true());
}

#[gtest]
fn first_set_sequence_stops_at_nonnullable_first_child() {
    // seq2(char('a'), char('b')): char('a') not nullable → first_set = {char('a')}
    let p: Rc<dyn HasChildren> = Rc::new(seq2(char('a'), char('b')).map(|_| ()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.first_set(&p).len(), eq(1));
    assert_that!(a.is_nullable(&p), is_false());
}

#[gtest]
fn first_set_sequence_looks_past_nullable_first_child() {
    // seq2(epsilon().map(|_| ()), char('a').constant(())): the first child's first_set
    // includes epsilon (as a leaf "start parser") + nullable sentinel. The sequence
    // propagates past the nullable first child and also includes char('a').constant(())
    // from the second child. So: 2 non-sentinel terminals {epsilon, char_a}, not nullable
    // (the second child char('a') is not nullable, so the whole sequence isn't).
    let p: Rc<dyn HasChildren> =
        Rc::new(seq2(epsilon().map(|_| ()), char('a').constant(())).map(|_| ()));
    let a = Analyzer::new(p.clone());
    assert_that!(a.first_set(&p).len(), eq(2));
    assert_that!(a.is_nullable(&p), is_false());
}

// ─────────────────────────────────────────────────────────────────────────────
// cycle_set
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn cycle_set_acyclic_grammar_all_empty() {
    // star(char('a')): no left-recursive cycle anywhere in the graph
    let p: Rc<dyn HasChildren> = Rc::new(char('a').star());
    let a = Analyzer::new(p);
    for parser in &a.parsers {
        assert_that!(a.cycle_set(parser).len(), eq(0));
    }
}

#[gtest]
fn cycle_set_no_false_positive_for_acyclic_choice() {
    let p: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')));
    let a = Analyzer::new(p);
    for parser in &a.parsers {
        assert_that!(a.cycle_set(parser).is_empty(), is_true());
    }
}

#[gtest]
fn cycle_set_self_reference_detected() {
    // s.set(s.borrow()): the SettableParserRef cycles back through the same slot
    let mut s: SettableParser<()> = SettableParser::undefined();
    let s_ref = s.borrow();
    s.set(s_ref);
    let root: Rc<dyn HasChildren> = Rc::new(s);
    let a = Analyzer::new(root);
    // parsers[0] = SettableParser, parsers[1] = SettableParserRef (the borrow)
    // The ref's children() re-resolves to itself → cycle
    assert_that!(a.cycle_set(&a.parsers[1]).len(), gt(0));
}

#[gtest]
fn cycle_set_mutual_recursion_detected() {
    // P = S '+' S; S = P | 'p'  — mutually left-recursive through choice
    let mut s_par: SettableParser<()> = SettableParser::undefined();
    let mut p_par: SettableParser<()> = SettableParser::undefined();
    let s_for_p1 = s_par.borrow();
    let s_for_p2 = s_par.borrow();
    let p_for_s = p_par.borrow();
    p_par.set(seq3(s_for_p1, char('+').constant(()), s_for_p2).map(|_| ()));
    s_par.set(choice2(p_for_s, char('p').constant(())));
    let root: Rc<dyn HasChildren> = Rc::new(s_par);
    // Keep p_par alive: its inner RefCell is referenced by the weak borrows above
    let _p_alive: Rc<dyn HasChildren> = Rc::new(p_par);
    let a = Analyzer::new(root);
    let has_cycle = a.parsers.iter().any(|p| !a.cycle_set(p).is_empty());
    assert_that!(has_cycle, is_true());
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: UnresolvedSettable
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn unresolved_settable_flags_undefined_settable() {
    let s: SettableParser<()> = SettableParser::undefined();
    let root: Rc<dyn HasChildren> = Rc::new(s);
    assert_one_issue(root, &[&UnresolvedSettable], "Unresolved settable");
}

#[gtest]
fn unresolved_settable_no_issue_after_set() {
    let mut s: SettableParser<()> = SettableParser::undefined();
    s.set(char('a').constant(()));
    let root: Rc<dyn HasChildren> = Rc::new(s);
    assert_no_issues(root, &[&UnresolvedSettable]);
}

#[gtest]
fn unresolved_settable_no_issue_for_non_settable_parser() {
    let root: Rc<dyn HasChildren> = Rc::new(char('a').star());
    assert_no_issues(root, &[&UnresolvedSettable]);
}

#[gtest]
fn unresolved_settable_no_issue_for_choice() {
    let root: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')));
    assert_no_issues(root, &[&UnresolvedSettable]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: NullableRepeater
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn nullable_repeater_flags_star_of_star() {
    // outer star's child (inner star) is nullable (min=0) → flagged
    let root: Rc<dyn HasChildren> = Rc::new(char('a').star().star());
    assert_one_issue(root, &[&NullableRepeater], "Nullable repeater");
}

#[gtest]
fn nullable_repeater_flags_star_of_epsilon() {
    let root: Rc<dyn HasChildren> = Rc::new(epsilon().star());
    assert_one_issue(root, &[&NullableRepeater], "Nullable repeater");
}

#[gtest]
fn nullable_repeater_flags_plus_of_epsilon() {
    let root: Rc<dyn HasChildren> = Rc::new(epsilon().plus());
    assert_one_issue(root, &[&NullableRepeater], "Nullable repeater");
}

#[gtest]
fn nullable_repeater_no_issue_for_star_of_char() {
    let root: Rc<dyn HasChildren> = Rc::new(char('a').star());
    assert_no_issues(root, &[&NullableRepeater]);
}

#[gtest]
fn nullable_repeater_no_issue_for_plus_of_char() {
    let root: Rc<dyn HasChildren> = Rc::new(char('a').plus());
    assert_no_issues(root, &[&NullableRepeater]);
}

#[gtest]
fn nullable_repeater_separated_nullable_element_nonnullable_sep_no_issue() {
    // epsilon element, any() separator (not nullable) → the separated-repeater exemption
    // applies: only flag when the *separator* is also nullable.
    let root: Rc<dyn HasChildren> = Rc::new(epsilon().star_with_sep(any(), Trailing::Disallowed));
    assert_no_issues(root, &[&NullableRepeater]);
}

#[gtest]
fn nullable_repeater_separated_nullable_element_nullable_sep_flagged() {
    // epsilon element AND epsilon separator → exemption does not apply, issue raised
    let root: Rc<dyn HasChildren> =
        Rc::new(epsilon().star_with_sep(epsilon(), Trailing::Disallowed));
    assert_one_issue(root, &[&NullableRepeater], "Nullable repeater");
}

#[gtest]
fn nullable_repeater_nonnullable_element_and_sep_no_issue() {
    let root: Rc<dyn HasChildren> = Rc::new(any().star_with_sep(any(), Trailing::Disallowed));
    assert_no_issues(root, &[&NullableRepeater]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: LeftRecursion
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn left_recursion_flags_self_reference() {
    // s.set(s.borrow()): dart's equivalent fixture (`createSelfReference`) sets a settable's
    // delegate to itself directly, since dart parsers are plain object references. We have no
    // such single-object self-reference: `.borrow()` is required to close the cycle without a
    // strong Rc cycle, so the delegate is a *distinct* SettableParserRef node, not `s` itself
    // (see cycle_set_self_reference_detected above -- cycle_set(s) is empty, only
    // cycle_set(s.borrow()) is non-empty). linter() walks every node in the graph, so this still
    // yields exactly one issue: the SettableParserRef is the only node with a non-empty cycle_set.
    let mut s: SettableParser<()> = SettableParser::undefined();
    let s_ref = s.borrow();
    s.set(s_ref);
    let root: Rc<dyn HasChildren> = Rc::new(s);
    assert_one_issue(root, &[&LeftRecursion], "Left recursion");
}

#[gtest]
fn left_recursion_no_issue_for_non_recursive_parser() {
    let root: Rc<dyn HasChildren> = Rc::new(digit());
    assert_no_issues(root, &[&LeftRecursion]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: NestedChoice
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn nested_choice_flags_choice_nested_directly_inside_another() {
    // [char('1'), [char('2'), char('3')].toChoiceParser(), char('4')].toChoiceParser()
    let root: Rc<dyn HasChildren> =
        Rc::new(choice3(char('1'), choice2(char('2'), char('3')), char('4')));
    assert_one_issue(root, &[&NestedChoice], "Nested choice");
}

#[gtest]
fn nested_choice_no_issue_when_nested_choice_is_flattened() {
    // Same shape, but the nested choice is wrapped in .input() (dart's .flatten()) first, so
    // it's no longer directly is_choice() from its parent's perspective. .constant(()) unifies
    // the three choice arms to one type -- dart's dynamically-typed toChoiceParser() doesn't
    // need this, since it freely mixes char and String results.
    let root: Rc<dyn HasChildren> = Rc::new(choice3(
        char('1').constant(()),
        choice2(char('2'), char('3')).input().constant(()),
        char('4').constant(()),
    ));
    assert_no_issues(root, &[&NestedChoice]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: UnreachableChoice
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn unreachable_choice_flags_alternative_after_nullable() {
    // [char('1'), char('2'), epsilon(), char('3')].toChoiceParser() -- epsilon() at index 2
    // means char('3') can never be reached. .constant(()) unifies char/epsilon's value types.
    let root: Rc<dyn HasChildren> = Rc::new(choice4(
        char('1').constant(()),
        char('2').constant(()),
        epsilon(),
        char('3').constant(()),
    ));
    assert_one_issue(root, &[&UnreachableChoice], "Unreachable choice");
}

#[gtest]
fn unreachable_choice_no_issue_without_nullable_alternative() {
    let root: Rc<dyn HasChildren> = Rc::new(choice3(char('1'), char('2'), char('3')));
    assert_no_issues(root, &[&UnreachableChoice]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: CharacterRepeater
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn character_repeater_flags_flattened_char_parser_star() {
    // char('a').star().flatten()
    let root: Rc<dyn HasChildren> = Rc::new(char('a').star().input());
    assert_one_issue(root, &[&CharacterRepeater], "Character repeater");
}

#[gtest]
fn character_repeater_flags_flattened_any_parser_plus() {
    // any().plus().flatten()
    let root: Rc<dyn HasChildren> = Rc::new(any().plus().input());
    assert_one_issue(root, &[&CharacterRepeater], "Character repeater");
}

#[gtest]
fn character_repeater_no_issue_for_tokenized_repeater() {
    // char('a').plus().token() -- token(), not flatten()/input(), so is_input() is false
    let root: Rc<dyn HasChildren> = Rc::new(char('a').plus().token());
    assert_no_issues(root, &[&CharacterRepeater]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: UnnecessaryInput (dart's UnnecessaryFlatten)
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn unnecessary_input_flags_char_delegate_with_no_message() {
    // any().flatten() -- any() is already char-shaped, no message to enable the fast path
    let root: Rc<dyn HasChildren> = Rc::new(any().input());
    assert_one_issue(root, &[&UnnecessaryInput], "Unnecessary input");
}

#[gtest]
fn unnecessary_input_no_issue_when_delegate_is_not_string_shaped() {
    // any().optional().flatten() -- opt() isn't one of the five string-shaped delegate kinds
    let root: Rc<dyn HasChildren> = Rc::new(any().opt().input());
    assert_no_issues(root, &[&UnnecessaryInput]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: UnoptimizedInput (dart's UnoptimizedFlatten)
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn unoptimized_input_flags_flatten_with_no_message() {
    let root: Rc<dyn HasChildren> = Rc::new(any().input());
    assert_one_issue(root, &[&UnoptimizedInput], "Unoptimized input");
}

#[gtest]
fn unoptimized_input_no_issue_when_message_provided() {
    let root: Rc<dyn HasChildren> =
        Rc::new(any().input_with_message("anything really".to_string()));
    assert_no_issues(root, &[&UnoptimizedInput]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: DuplicateParser
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn duplicate_parser_flags_two_structurally_equal_children() {
    // seq2(digit(), digit()): the two digit() leaves are structurally equal (same CharKind,
    // same message), so the first one is flagged. Only the first occurrence reports (dart's
    // `duplicates.first == parser` guard), so exactly one issue.
    let root: Rc<dyn HasChildren> = Rc::new(seq2(digit(), digit()));
    assert_one_issue(root, &[&DuplicateParser], "Duplicate parser");
}

#[gtest]
fn duplicate_parser_no_issue_when_children_differ() {
    // Distinct messages make the two digits structurally unequal (message is part of the
    // Char kind), so neither is a duplicate.
    let root: Rc<dyn HasChildren> = Rc::new(seq2(
        digit().with_message("first"),
        digit().with_message("second"),
    ));
    assert_no_issues(root, &[&DuplicateParser]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: RepeatedChoice
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn repeated_choice_flags_identical_alternatives() {
    // [1, 2, 3, 2, 4]: children[1] and children[3] are both char('2') → structurally equal,
    // so the second is unreachable. One issue on the choice.
    let root: Rc<dyn HasChildren> = Rc::new(choice5(
        char('1'),
        char('2'),
        char('3'),
        char('2'),
        char('4'),
    ));
    assert_one_issue(root, &[&RepeatedChoice], "Repeated choice");
}

#[gtest]
fn repeated_choice_no_issue_when_all_alternatives_distinct() {
    let root: Rc<dyn HasChildren> = Rc::new(choice4(char('1'), char('2'), char('3'), char('4')));
    assert_no_issues(root, &[&RepeatedChoice]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: OverlappingChoice
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn overlapping_choice_flags_alternatives_with_equal_first_sets() {
    // dart: [char('1'), char('2') & char('a'), char('2') & char('b'), char('3')].toChoiceParser()
    // children[1] and children[2] are distinct parsers, but both have first-set {char('2')} --
    // so they *overlap* even though they aren't identical (that's what sets this apart from
    // RepeatedChoice). One issue on the choice.
    //
    // .constant(()) unifies the mixed value types (char vs (char, char)) into one Parser<()>;
    // it's transparent to first-sets (first_set(x.constant(())) == first_set(x)), so the
    // overlap is detected exactly as in dart.
    let root: Rc<dyn HasChildren> = Rc::new(choice4(
        char('1').constant(()),
        seq2(char('2'), char('a')).constant(()),
        seq2(char('2'), char('b')).constant(()),
        char('3').constant(()),
    ));
    assert_one_issue(root, &[&OverlappingChoice], "Overlapping choice");
}

#[gtest]
fn overlapping_choice_no_issue_when_first_sets_disjoint() {
    // [char('1'), char('2'), char('3')] -- first-sets {1}, {2}, {3} are pairwise disjoint.
    let root: Rc<dyn HasChildren> = Rc::new(choice3(char('1'), char('2'), char('3')));
    assert_no_issues(root, &[&OverlappingChoice]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linter: UnusedResult
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn unused_result_flags_flatten_over_a_result_producing_child() {
    // dart: digit().map(int.parse).star().flatten()
    // The .map(...) is a result-producing parser buried under star().input(); the flatten
    // discards its result and returns the consumed input instead → one issue on the flatten.
    let root: Rc<dyn HasChildren> =
        Rc::new(digit().map(|c| c.to_digit(10).unwrap()).star().input());
    assert_one_issue(root, &[&UnusedResult], "Unused result");
}

#[gtest]
fn unused_result_no_issue_without_a_result_producing_child() {
    // digit().star().flatten() -- digit (char) and star (repeating) are not result-producing,
    // so the flatten discards nothing meaningful.
    let root: Rc<dyn HasChildren> = Rc::new(digit().star().input());
    assert_no_issues(root, &[&UnusedResult]);
}

// ─────────────────────────────────────────────────────────────────────────────
// ALL_LINTER_RULES smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn all_rules_clean_grammar_produces_no_issues() {
    // A well-formed grammar: no undefined settables, no nullable repeaters.
    let root: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')).star());
    let issues = linter(root, ALL_LINTER_RULES);
    assert_that!(issues.len(), eq(0));
}

#[gtest]
fn all_rules_undefined_settable_found() {
    let root: Rc<dyn HasChildren> = Rc::new(SettableParser::<()>::undefined());
    let issues = linter(root, ALL_LINTER_RULES);
    assert!(
        issues.iter().any(|i| i.title == "Unresolved settable"),
        "expected an 'Unresolved settable' issue"
    );
}
