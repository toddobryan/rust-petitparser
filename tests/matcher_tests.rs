use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::rc::Rc;

trait AsInput {
    fn into_input(self) -> Rc<[char]>;
}

impl AsInput for &str {
    fn into_input(self) -> Rc<[char]> {
        self.chars().collect::<Vec<_>>().into()
    }
}

#[gtest]
fn all_matches_basic() {
    let buffer: Rc<[char]> = "abacada".into_input();
    let matches: Vec<char> = char('a')
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec!['a', 'a', 'a', 'a']));
}

#[gtest]
fn all_matches_none() {
    let buffer: Rc<[char]> = "bcdef".into_input();
    let matches: Vec<char> = char('a')
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![]));
}

#[gtest]
fn all_matches_start_position() {
    let buffer: Rc<[char]> = "aaba".into_input();
    let matches: Vec<char> = char('a')
        .all_matches(buffer, 2, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec!['a']));
}

#[gtest]
fn all_matches_non_overlapping() {
    let buffer: Rc<[char]> = "aaaa".into_input();
    let matches: Vec<(char, char)> = seq2(char('a'), char('a'))
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![('a', 'a'), ('a', 'a')]));
}

#[gtest]
fn all_matches_overlapping() {
    let buffer: Rc<[char]> = "aaaa".into_input();
    let matches: Vec<(char, char)> = seq2(char('a'), char('a'))
        .all_matches(buffer, 0, true)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![('a', 'a'), ('a', 'a'), ('a', 'a')]));
}

#[gtest]
fn accept() {
    let p = &char('a');
    assert_that!(p.accept("a"), is_true());
    assert_that!(p.accept("b"), is_false());
}

#[gtest]
fn accept_with_start() {
    let p = char('b');
    assert_that!(p.accept_at("abc", 0), is_false());
    assert_that!(p.accept_at("abc", 1), is_true());
    assert_that!(p.accept_at("abc", 2), is_false());
    assert_that!(p.accept_at("abc", 3), is_false());
    assert_that!(p.accept_at("abc", 4), is_false());
}
