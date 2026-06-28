//! Parsing throughput benchmarks for the two largest example grammars.
//!
//! These exist to measure the cost of combinator *dispatch*: the `rc-storage`
//! experiment converts every combinator's delegate field from a monomorphized
//! generic (`P: Parser<T>`, static dispatch) to `Rc<dyn Parser<T>>` (heap
//! allocation at build time + a vtable call per `parse_on`). We benchmark
//! against the `main` baseline to decide whether that cost is acceptable.
//!
//! The grammars themselves live under `tests/example-grammars/` and are pulled
//! in verbatim via `#[path]`. Their rule bodies use only the public combinator
//! API, so they compile unchanged on both branches — which is what makes the
//! before/after comparison apples-to-apples. The `#[grammar]` modules were made
//! `pub` so the generated `DartGrammar` / `SmalltalkGrammar` structs are
//! reachable from here; everything else (the `#[gtest]` test fns dragged in by
//! the include) is dead code in this binary, hence the broad `allow`.

use criterion::{Criterion, criterion_group, criterion_main};
use rust_petitparser::prelude::*;
use std::hint::black_box;

#[path = "../tests/example-grammars/dart.rs"]
#[allow(dead_code, unused_imports, unused_macros, clippy::all)]
mod dart;

#[path = "../tests/example-grammars/smalltalk/mod.rs"]
#[allow(dead_code, unused_imports, unused_macros, clippy::all)]
mod smalltalk;

use dart::DartGrammar;
use smalltalk::SmalltalkGrammar;

/// A valid Dart compilation unit assembled from grammar-tested snippets, repeated
/// `units` times. Each unit is one class plus one function whose body exercises a
/// broad cross-section of the statement/expression rules.
fn dart_source(units: usize) -> String {
    const UNIT: &str = r#"
class A extends B implements C, D {}
void unit() {
  var a = b;
  if (a) {} else if (b) {} else {}
  while (a) {}
  do {} while (b);
  for (var a = b, c = d; e; f++) {}
  for (a in b) {}
  switch (a) { case b: {} default: {}}
  try {} catch (a b, c d) {} finally {}
  a(b, c, d);
  return a + b;
}
"#;
    let mut s = String::from("library bench;\n");
    for _ in 0..units {
        s.push_str(UNIT);
    }
    s
}

/// A valid Smalltalk method whose body is `stmts` repeated statements ending in a
/// return. Built from grammar-tested constructs (assignment, binary/unary
/// messages, literals, return).
fn smalltalk_source(stmts: usize) -> String {
    let mut s = String::from("bench\n| a b c |\n");
    for _ in 0..stmts {
        s.push_str("a := 1 + 2 . b := 1 abs . c := 1 . ");
    }
    s.push_str("^ 0 - self");
    s
}

fn bench_dart(c: &mut Criterion) {
    let g = DartGrammar::new();
    let input = dart_source(40);
    let ctx = Context::new(&input, 0);
    // Fail loudly if the assembled input doesn't actually parse, rather than
    // silently benchmarking the error path.
    assert!(g.parse(&input).is_ok(), "dart bench input must parse");

    let mut group = c.benchmark_group("dart");
    group.bench_function("parse", |b| {
        b.iter(|| black_box(g.parse_on(black_box(&ctx))))
    });
    group.bench_function("build", |b| b.iter(|| black_box(DartGrammar::new())));
    group.finish();
}

fn bench_smalltalk(c: &mut Criterion) {
    let g = SmalltalkGrammar::new();
    let input = smalltalk_source(60);
    let ctx = Context::new(&input, 0);
    assert!(g.parse(&input).is_ok(), "smalltalk bench input must parse");

    let mut group = c.benchmark_group("smalltalk");
    group.bench_function("parse", |b| {
        b.iter(|| black_box(g.parse_on(black_box(&ctx))))
    });
    group.bench_function("build", |b| b.iter(|| black_box(SmalltalkGrammar::new())));
    group.finish();
}

criterion_group!(benches, bench_dart, bench_smalltalk);
criterion_main!(benches);
