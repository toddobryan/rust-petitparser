use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::rc::Rc;

#[gtest]
fn all_matches_basic() {
    let buffer: Rc<[char]> = "abacada".chars().collect::<Vec<_>>().into();
    let matches: Vec<char> = char('a')
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec!['a', 'a', 'a', 'a']));
}

#[gtest]
fn all_matches_none() {
    let buffer: Rc<[char]> = "bcdef".chars().collect::<Vec<_>>().into();
    let matches: Vec<char> = char('a')
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![]));
}

#[gtest]
fn all_matches_start_position() {
    let buffer: Rc<[char]> = "aaba".chars().collect::<Vec<_>>().into();
    let matches: Vec<char> = char('a')
        .all_matches(buffer, 2, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec!['a']));
}

#[gtest]
fn all_matches_non_overlapping() {
    let buffer: Rc<[char]> = "aaaa".chars().collect::<Vec<_>>().into();
    let matches: Vec<(char, char)> = seq2(char('a'), char('a'))
        .all_matches(buffer, 0, false)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![('a', 'a'), ('a', 'a')]));
}

#[gtest]
fn all_matches_overlapping() {
    let buffer: Rc<[char]> = "aaaa".chars().collect::<Vec<_>>().into();
    let matches: Vec<(char, char)> = seq2(char('a'), char('a'))
        .all_matches(buffer, 0, true)
        .into_iter()
        .collect();
    assert_that!(matches, eq(&vec![('a', 'a'), ('a', 'a'), ('a', 'a')]));
}
