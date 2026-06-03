use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::rc::Rc;

struct TestVals {
    identifier: Rc<dyn Parser<String>>,
    number: Rc<dyn Parser<String>>,
    quoted: Rc<dyn Parser<String>>,
    keyword: Rc<dyn Parser<String>>,
    javadoc: Rc<dyn Parser<String>>,
    multiline: Rc<dyn Parser<String>>,
}

impl TestVals {
    fn new() -> Self {
        let identifier = Rc::new(seq2(letter(), word().star()).input());
        let number = Rc::new(
            seq3(
                char('-').opt(),
                digit().plus(),
                seq2(char('.'), digit().plus()).opt(),
            )
            .input(),
        );
        let quoted = Rc::new(seq3(char('"'), char('"').neg().star(), char('"')).input());
        let keyword = Rc::new(
            seq3(
                string("return"),
                whitespace().plus().input(),
                choice3(identifier.clone(), number.clone(), quoted.clone()),
            )
            .map(|(_, _, val)| val),
        );
        let javadoc = Rc::new(seq3(string("/**"), string("*/").neg().star(), string("*/")).input());
        let multiline = Rc::new(
            seq3(
                string("\"\"\""),
                choice2(string("\\\"\"\""), any().input())
                    .star_lazy(string("\"\"\""))
                    .input(),
                string("\"\"\""),
            )
            .map(|(_, val, _)| val),
        );
        Self {
            identifier,
            number,
            quoted,
            keyword,
            javadoc,
            multiline,
        }
    }
}

#[gtest]
fn valid_identifier() {
    let tv = TestVals::new();
    assert_success!(tv.identifier, "a", "a", 1);
    assert_success!(tv.identifier, "a1", "a1", 2);
    assert_success!(tv.identifier, "a12", "a12", 3);
    assert_success!(tv.identifier, "ab", "ab", 2);
    assert_success!(tv.identifier, "a1b", "a1b", 3);
}

#[gtest]
fn incomplete_identifier() {
    let tv = TestVals::new();
    assert_success!(tv.identifier, "a=", "a", 1);
    assert_success!(tv.identifier, "a1-", "a1", 2);
    assert_success!(tv.identifier, "a12+", "a12", 3);
    assert_success!(tv.identifier, "ab ", "ab", 2);
}

#[gtest]
fn invalid_identifier() {
    let tv = TestVals::new();
    assert_failure!(
        tv.identifier,
        "",
        "expected letter, but reached end of input",
        0
    );
    assert_failure!(tv.identifier, "1", "expected letter, but found '1'", 0);
    assert_failure!(tv.identifier, "1a", "expected letter, but found '1'", 0);
}

#[gtest]
fn positive_number() {
    let tv = TestVals::new();
    assert_success!(tv.number, "1", "1", 1);
    assert_success!(tv.number, "12", "12", 2);
    assert_success!(tv.number, "12.3", "12.3", 4);
    assert_success!(tv.number, "12.34", "12.34", 5);
}

#[gtest]
fn negative_number() {
    let tv = TestVals::new();
    assert_success!(tv.number, "-1", "-1", 2);
    assert_success!(tv.number, "-12", "-12", 3);
    assert_success!(tv.number, "-12.3", "-12.3", 5);
    assert_success!(tv.number, "-12.34", "-12.34", 6);
}

#[gtest]
fn incomplete_number() {
    let tv = TestVals::new();
    assert_success!(tv.number, "1..", "1", 1);
    assert_success!(tv.number, "12-", "12", 2);
    assert_success!(tv.number, "12.3.", "12.3", 4);
    assert_success!(tv.number, "12.34.", "12.34", 5);
}

#[gtest]
fn invalid_number() {
    let tv = TestVals::new();
    assert_failure!(tv.number, "", "expected digit, but reached end of input", 0);
    assert_failure!(
        tv.number,
        "-",
        "expected digit, but reached end of input",
        1
    );
    assert_failure!(tv.number, "-x", "expected digit, but found 'x'", 1);
    assert_failure!(tv.number, ".", "expected digit, but found '.'", 0);
    assert_failure!(tv.number, ".1", "expected digit, but found '.'", 0);
}

#[gtest]
fn valid_string() {
    let tv = TestVals::new();
    assert_success!(tv.quoted, "\"\"", "\"\"", 2);
    assert_success!(tv.quoted, "\"a\"", "\"a\"", 3);
    assert_success!(tv.quoted, "\"ab\"", "\"ab\"", 4);
    assert_success!(tv.quoted, "\"abc\"", "\"abc\"", 5);
}

#[gtest]
fn incomplete_string() {
    let tv = TestVals::new();
    assert_success!(tv.quoted, "\"\"x", "\"\"", 2);
    assert_success!(tv.quoted, "\"a\"x", "\"a\"", 3);
    assert_success!(tv.quoted, "\"ab\"x", "\"ab\"", 4);
    assert_success!(tv.quoted, "\"abc\"x", "\"abc\"", 5);
}

#[gtest]
fn invalid_string() {
    let tv = TestVals::new();
    assert_failure!(
        tv.quoted,
        "\"",
        "expected '\"', but reached end of input",
        1
    );
    assert_failure!(
        tv.quoted,
        "\"a",
        "expected '\"', but reached end of input",
        2
    );
    assert_failure!(
        tv.quoted,
        "\"ab",
        "expected '\"', but reached end of input",
        3
    );
    assert_failure!(tv.quoted, "a\"", "expected '\"', but found 'a'", 0);
    assert_failure!(tv.quoted, "ab\"", "expected '\"', but found 'a'", 0);
}

#[gtest]
fn return_statement() {
    let tv = TestVals::new();
    assert_success!(tv.keyword, "return f", "f", 8);
    assert_success!(tv.keyword, "return  f", "f", 9);
    assert_success!(tv.keyword, "return foo", "foo", 10);
    assert_success!(tv.keyword, "return    foo", "foo", 13);
    assert_success!(tv.keyword, "return 1", "1", 8);
    assert_success!(tv.keyword, "return  1", "1", 9);
    assert_success!(tv.keyword, "return -2.3", "-2.3", 11);
    assert_success!(tv.keyword, "return    -2.3", "-2.3", 14);
    assert_success!(tv.keyword, "return \"a\"", "\"a\"", 10);
    assert_success!(tv.keyword, "return  \"a\"", "\"a\"", 11);
}

#[gtest]
fn invalid_statement() {
    let tv = TestVals::new();
    assert_failure!(tv.keyword, "retur f", "Expected string: \"return\"", 0);
    assert_failure!(
        tv.keyword,
        "return1",
        "expected whitespace, but found '1'",
        6
    );
    assert_failure!(tv.keyword, "return  _", "expected '\"', but found '_'", 8);
}

#[gtest]
fn javadoc() {
    let tv = TestVals::new();
    assert_success!(tv.javadoc, "/** foo */", "/** foo */", 10);
    assert_success!(tv.javadoc, "/** * * */", "/** * * */", 10);
}

#[gtest]
fn multiline() {
    let tv = TestVals::new();
    assert_success!(tv.multiline, "\"\"\"abc\"\"\"", "abc", 9);
    assert_success!(tv.multiline, "\"\"\"abc\\n\"\"\"", "abc\\n", 11);
    assert_success!(
        tv.multiline,
        "\"\"\"abc\\\"\"\"def\"\"\"",
        "abc\\\"\"\"def",
        16
    );
}
