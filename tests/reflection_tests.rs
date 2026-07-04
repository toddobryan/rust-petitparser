//! Tests for `Analyzer::all_children` / `find_path` / `find_all_paths`, ported from
//! dart-petitparser's `reflection_test.dart` (`allChildren` and `findPath` groups).
//!
//! Key divergence from dart: dart's parsers are shared object references, so the same terminal
//! can appear at multiple positions in a graph and be compared by identity. Our combinators wrap
//! every argument in a fresh `Rc::new(..)`, so "the same" sub-parser passed twice becomes two
//! distinct nodes (the divergence documented by `linter_tests::parsers_raw_rc_share_does_not_dedupe`).
//! Where dart relies on a shared terminal reachable by two paths, we either navigate to the real
//! graph node via `children()` and compare with `Rc::ptr_eq`, or use a `kind()`-predicate that
//! matches several distinct nodes — noted at each such site.

use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::rc::Rc;

/// Thin data-pointer identity of a node, for order-independent set membership.
fn addr(p: &Rc<dyn HasChildren>) -> *const () {
    Rc::as_ptr(p) as *const ()
}

fn contains(nodes: &[Rc<dyn HasChildren>], target: &Rc<dyn HasChildren>) -> bool {
    nodes.iter().any(|n| Rc::ptr_eq(n, target))
}

// ─────────────────────────────────────────────────────────────────────────────
// allChildren
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn all_children_single() {
    // dart: inner = char('a'); parser = inner.plus(); allChildren(parser) == {inner}; allChildren(inner) empty.
    let parser: Rc<dyn HasChildren> = Rc::new(char('a').plus());
    let a = Analyzer::new(parser.clone());
    let inner = parser.children()[0].clone(); // the char('a') node as it lives in the graph
    let kids = a.all_children(&parser);
    assert_that!(kids.len(), eq(1));
    assert!(Rc::ptr_eq(&kids[0], &inner));
    assert_that!(a.all_children(&inner).len(), eq(0));
}

#[gtest]
fn all_children_multiple() {
    // dart: parser = inner1 & inner2; allChildren(parser) == {inner1, inner2}.
    let parser: Rc<dyn HasChildren> = Rc::new(seq2(char('a'), char('b')));
    let a = Analyzer::new(parser.clone());
    let inner1 = parser.children()[0].clone();
    let inner2 = parser.children()[1].clone();
    let kids = a.all_children(&parser);
    assert_that!(kids.len(), eq(2));
    assert!(contains(&kids, &inner1));
    assert!(contains(&kids, &inner2));
    assert_that!(a.all_children(&inner1).len(), eq(0));
    assert_that!(a.all_children(&inner2).len(), eq(0));
}

#[gtest]
fn all_children_repeated_distinct_instances_not_deduped() {
    // dart's 'repeated': `inner1 | inner2 | inner2` with inner2 the SAME object twice dedups to
    // {inner1, inner2} (2). Our combinators re-wrap each arg in Rc::new, so the two char('b')
    // arms are distinct nodes -> 3, not 2. Same re-wrap divergence as parsers_raw_rc_share.
    let parser: Rc<dyn HasChildren> = Rc::new(choice3(char('a'), char('b'), char('b')));
    let a = Analyzer::new(parser.clone());
    assert_that!(a.all_children(&parser).len(), eq(3));
}

#[gtest]
fn all_children_self_reference_terminates() {
    // dart's 'self reference': undefined set to itself; allChildren(parser) == {parser}. Ours is a
    // SettableParser whose delegate is its own weak ref; all_children must terminate on the cycle
    // (per-walk visited-set) and report the reachable node(s).
    let mut s: SettableParser<()> = SettableParser::undefined();
    let s_ref = s.borrow();
    s.set(s_ref);
    let root: Rc<dyn HasChildren> = Rc::new(s);
    let a = Analyzer::new(root.clone());
    let kids = a.all_children(&root);
    assert_that!(kids.is_empty(), is_false());
    // The cycle is closed through the weak ref, so the reachable set is finite (no infinite loop).
    assert_that!(kids.len(), le(2));
}

#[gtest]
fn all_children_recursive_terminates_over_cycle() {
    // dart's 'recursive': inner2 = undefined; parser = [inner1, inner2].toChoiceParser();
    // inner2.set(parser); allChildren(parser) contains {inner1, inner2, parser} — parser itself
    // included because it's reachable from its own children via the cycle.
    //
    // Divergence: `cap.set(x)` internally does `Rc::new(x)`, so our cycle closes through a *fresh
    // wrapper* of the choice rather than `root` itself. Hence `root` is NOT in its own
    // all_children (we see the re-wrapped copy instead), but the traversal still goes all the way
    // around the cycle — reaching the char leaf beyond the constant — and terminates.
    let mut cap: SettableParser<()> = SettableParser::undefined();
    let parser: Rc<dyn Parser<()>> = Rc::new(choice2(char('a').constant(()), cap.borrow()));
    cap.set(parser.clone());
    let root: Rc<dyn HasChildren> = parser.clone();
    let a = Analyzer::new(root.clone());
    let kids = a.all_children(&root);

    // Terminated with a non-empty, all-distinct set.
    assert_that!(kids.is_empty(), is_false());
    let mut addrs: Vec<*const ()> = kids.iter().map(addr).collect();
    addrs.sort();
    addrs.dedup();
    assert_that!(addrs.len(), eq(kids.len()));

    // The traversal reached around the cycle far enough to find the char('a') leaf.
    assert!(
        kids.iter()
            .any(|k| matches!(k.kind(), ParserKind::Char { .. }))
    );

    // Re-wrap divergence: root itself is not in its own transitive children (the cycle closes
    // through a fresh Rc wrapper of the choice, not `root`).
    assert!(!contains(&kids, &root));
}

// ─────────────────────────────────────────────────────────────────────────────
// findPath / findAllPaths
// ─────────────────────────────────────────────────────────────────────────────

#[gtest]
fn find_path_simple() {
    // dart: parser = char('a'); findPathTo(parser, parser) -> [parser], indexes empty.
    let root: Rc<dyn HasChildren> = Rc::new(char('a'));
    let a = Analyzer::new(root.clone());
    let is_root = |p: &ParserPath| Rc::ptr_eq(&p.target(), &root);

    let path = a.find_path(&root, &is_root).unwrap();
    assert!(Rc::ptr_eq(&path.source(), &root));
    assert!(Rc::ptr_eq(&path.target(), &root));
    assert_that!(path.parsers.len(), eq(1));
    assert_that!(path.indexes.len(), eq(0));

    let paths = a.find_all_paths(&root, &is_root);
    assert_that!(paths.len(), eq(1));
    assert_that!(paths[0].parsers.len(), eq(1));
    assert_that!(paths[0].indexes.len(), eq(0));
}

#[gtest]
fn find_path_choice() {
    // dart: parser = terminal | terminal (shared terminal) -> 2 paths to the one terminal. Our
    // arms are distinct, so: a path to a *specific* arm is unique ([parser, arm], index [0]); and
    // a kind()-predicate matching any char finds *both* arms -> 2 paths.
    let parser: Rc<dyn HasChildren> = Rc::new(choice2(char('a'), char('b')));
    let a = Analyzer::new(parser.clone());
    let arm0 = parser.children()[0].clone();

    let path = a
        .find_path(&parser, &|p| Rc::ptr_eq(&p.target(), &arm0))
        .unwrap();
    assert!(Rc::ptr_eq(&path.source(), &parser));
    assert!(Rc::ptr_eq(&path.target(), &arm0));
    assert_eq!(path.parsers.len(), 2);
    assert_eq!(path.indexes, vec![0]);

    let is_char = |p: &ParserPath| matches!(p.target().kind(), ParserKind::Char { .. });
    let paths = a.find_all_paths(&parser, &is_char);
    assert_that!(paths.len(), eq(2));
    assert_eq!(paths[0].indexes, vec![0]);
    assert_eq!(paths[1].indexes, vec![1]);
}

#[gtest]
fn find_path_finds_shortest() {
    // dart's 'length': a shared terminal reachable directly (len 2) and via a repeater (len 3);
    // findPath returns the shortest. Reproduced with a kind()-predicate over distinct chars:
    // seq2(char('a'), char('b').star()) has a char at depth 1 (first arm) and depth 2 (star child).
    let parser: Rc<dyn HasChildren> = Rc::new(seq2(char('a'), char('b').star()));
    let a = Analyzer::new(parser.clone());
    let is_char = |p: &ParserPath| matches!(p.target().kind(), ParserKind::Char { .. });

    let all = a.find_all_paths(&parser, &is_char);
    assert_that!(all.len(), eq(2)); // [parser, char('a')] and [parser, star, char('b')]

    let shortest = a.find_path(&parser, &is_char).unwrap();
    assert_that!(shortest.parsers.len(), eq(2)); // the direct arm, not the one through the star
}

#[gtest]
fn find_all_paths_false_predicate_is_empty_and_terminates() {
    // dart's 'recursive grammar' / 'self reference': findAllPaths with a never-matching predicate
    // must fully explore a cyclic graph and return empty without looping (per-path cycle guard).
    let mut s: SettableParser<()> = SettableParser::undefined();
    let s_ref = s.borrow();
    s.set(s_ref);
    let root: Rc<dyn HasChildren> = Rc::new(s);
    let a = Analyzer::new(root.clone());
    let paths = a.find_all_paths(&root, &|_| false);
    assert_that!(paths.is_empty(), is_true());
}
