use googletest::prelude::*;
use rust_petitparser::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// continuation
#[gtest]
fn delegation() {
    let p = digit().call_cc(|cont, ctx| cont.resume(ctx));
    assert_success!(p, "1", '1', 1);
    assert_failure!(p, "a", "expected digit, but found 'a'", 0);
}

#[gtest]
fn diversion() {
    let p = digit().call_cc(|_, ctx| letter().parse_on(ctx));
    assert_success!(p, "a", 'a', 1);
    assert_failure!(p, "1", "expected letter, but found '1'", 0);
}

#[gtest]
fn resume() {
    // Shared, interior-mutable collections: the handler is `Fn` (called via &self) and is
    // `move`, so it captures clones of these handles; the originals stay readable out here.
    let continuations: Rc<RefCell<Vec<Continuation<char>>>> = Rc::new(RefCell::new(Vec::new()));
    let contexts: Rc<RefCell<Vec<Context>>> = Rc::new(RefCell::new(Vec::new()));

    let conts_for_handler = continuations.clone();
    let ctxs_for_handler = contexts.clone();
    let p = digit().call_cc(move |cont, ctx| {
        conts_for_handler.borrow_mut().push(cont);
        ctxs_for_handler.borrow_mut().push(ctx.clone());
        // we have to return something for now
        ctx.failure::<char>("Abort")
    });

    let _failure = p.parse("1").unwrap_err();
    let _failure = p.parse("a").unwrap_err();

    // later we can execute the captured continuations
    let conts = continuations.borrow();
    let ctxs = contexts.borrow();

    assert_that!(conts[0].resume(&ctxs[0]).is_ok(), eq(true));
    assert_that!(conts[1].resume(&ctxs[1]).is_ok(), eq(false));

    // of course the continuations can be resumed multiple times
    assert_that!(conts[0].resume(&ctxs[0]).is_ok(), eq(true));
    assert_that!(conts[1].resume(&ctxs[1]).is_ok(), eq(false));
}

#[gtest]
fn success() {
    let p = digit()
        .call_cc(|_, ctx| ctx.success("success"));
    assert_success!(p, "1", "success", 0);
    assert_success!(p, "a", "success", 0);
}

#[gtest]
fn failure() {
    let p = digit()
        .call_cc(|_, ctx| ctx.failure::<String>("failure"));
    assert_failure!(p, "1", "failure", 0);
    assert_failure!(p, "a", "failure", 0);
}
