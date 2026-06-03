use std::rc::Rc;

use googletest::prelude::*;
use rust_petitparser::prelude::*;

struct TestVals {
    buffer: Rc<[char]>,
    context: Context,
}

impl TestVals {
    fn new() -> Self {
        let buffer = Rc::new(['a', '\n', 'c']);
        Self {
            buffer: buffer.clone(),
            context: Context {
                buffer: buffer.clone(),
                position: 0,
            },
        }
    }
}

#[gtest]
fn context() {
    let tv = TestVals::new();
    assert_that!(tv.context.buffer, eq(&tv.buffer));
    assert_that!(tv.context.position, eq(0));
    assert_that!(tv.context, displays_as(eq("Context[0=1:1]")));
}

// group success
#[gtest]
fn success_default() {
    let tv = TestVals::new();
    let success = tv.context.success("result").unwrap();
    assert_that!(success.buffer(), eq(&tv.buffer));
    assert_that!(success.position(), eq(0));
    assert_that!(success.value, eq("result"));
    assert_that!(success, displays_as(eq("Success[0=1:1]: \"result\"")));
}

#[gtest]
fn success_with_position() {
    let tv = TestVals::new();
    let success = tv.context.success_with_position("result", 2).unwrap();
    assert_that!(success.buffer(), eq(&tv.buffer));
    assert_that!(success.position(), eq(2));
    assert_that!(success.value, eq("result"));
    assert_that!(success, displays_as(eq("Success[2=2:1]: \"result\"")));
}

// group failure
#[gtest]
fn failure_default() {
    let tv = TestVals::new();
    let failure = tv.context.failure::<String>("error").unwrap_err();
    assert_that!(failure.buffer(), eq(&tv.buffer));
    assert_that!(failure.position(), eq(0));
    assert_that!(failure.message, eq("error"));
    assert_that!(failure, displays_as(eq("Failure[0=1:1]: error")));
}

#[gtest]
fn failure_with_position() {
    let tv = TestVals::new();
    let failure = tv
        .context
        .failure_with_position::<String>("error", 2)
        .unwrap_err();
    assert_that!(failure.buffer(), eq(&tv.buffer));
    assert_that!(failure.position(), eq(2));
    assert_that!(failure.message, eq("error"));
    assert_that!(failure, displays_as(eq("Failure[2=2:1]: error")));
}
