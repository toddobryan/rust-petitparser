# rust-petitparser

A Rust port of PetitParser (originally Dart/Java).

## Reference Checkout
The dart-petitparser reference clone at `/home/toddobryan/code/dart/dart-petitparser` is pinned
to tag `v7.0.2` (detached HEAD at `a8579a0`, the latest released version as of 2026-06-24) — use
this as the ground truth for parity checks rather than `origin/main` (which has unreleased
CI/dependency-bump churn beyond the tag). Diffed `v7.0.2` against our previous reference point
(`807dece`, 7 commits past `v7.0.1`): almost entirely doc-comment-only changes, except one new
combinator — `.constant(value)` (`lib/src/parser/action/constant.dart`) — which we already had
under the name `.to(value)`, with the identical shape (clone on the slow path, position-only skip
on the fast path); renamed ours to `.constant(value)` (`src/parser/action/constant.rs`,
`ConstantParser`) to match dart exactly. Surfaced one real naming collision while doing the
rename: Pascal's own grammar has a rule named `constant`, and since `PascalGrammar` implements
`Parser<T>` (so it picks up the blanket `ParserExt<T>` impl too), Rust's method resolution tries
the value-receiver trait method (`ParserExt::constant<V>(self, value: V)`) before the `&self`-
receiver inherent accessor the `#[grammar]` macro generates — same root cause, same fix, as the
pre-existing `type` → `pascal_type` rename (a Rust keyword collision, not a trait collision, but
the resolution mechanics that make renaming necessary are the same). Fixed by renaming the rule
to `pascal_constant` in `tests/example-grammars/pascal.rs`, mirroring `pascal_type`.

## User Preferences
- Provide hints and structural guidance, not full code — let the user implement with guidance
- Give type signatures, key gotchas, and design rationale

## Workspace Layout
The repo root **is** the `rust-petitparser` library package, and it also owns a Cargo workspace
(root `Cargo.toml` has both `[package]` and `[workspace]`). The only other member:
- `rust-petitparser-macros/` — proc-macro crate exposing `#[grammar]` (see proc macro section),
  pulled in via `rust-petitparser-macros = { path = "rust-petitparser-macros" }`

The root package is an implicit workspace member, so `[workspace].members` lists only the macros
crate. Single `Cargo.lock` at the root.

`src/lib.rs` keeps `core`/`matcher`/`parser` as `pub(crate)`; the only public surface is
`pub mod prelude` plus a re-export of `grammar`. Downstream code (and the example grammars) does
`use rust_petitparser::prelude::*;` — the prelude re-exports the `Parser` trait, `HasChildren`,
`ParserExt`, `Context`/`Success`/`Failure`/`ParseResult`, `Token`, all parser constructors, the
`assert_success!`/`assert_failure!` macros, and `grammar`.

## Module Structure
```
src/
  prelude.rs    - public re-export surface (see Workspace Layout)
  core/
    context.rs    - Context, Success, Failure, ParseResult
    parser.rs     - Parser<T> trait + HasChildren supertrait (see "Storage model" section)
    result.rs     - ParseResult type alias
    token.rs      - Token<T>, line_and_column_of, position_string
    test_helpers.rs
  parser/
    character.rs  - CharParser, PredicateCharParser, any(), char(), letter(), digit(), one_of(), etc.
    combinator/
      choice.rs     - Choice2-11 via choice_impl! macro (generic over value type T only)
      sequence.rs   - Seq2-9 via impl_seq! macro (generic over value types T1..Tn only)
      settable.rs   - SettableParser<T> / SettableParserRef<T> for recursive grammars
      lookahead.rs  - AndParser<T>, NotParser<T>
      skip.rs       - SkipParser<Aft, Bef, T> (open, content, close → content)
    action/
      map.rs      - MapParser<T, F> (delegate: Rc<dyn Parser<T>>, needs PhantomData<T>)
      token.rs    - TokenParser<T>
      input.rs    - InputParser (flatten matched chars to String)
      only_if.rs  - OnlyIfParser (predicate gate on success value)
      flat_map.rs - FlatMapParser (monadic bind: value → next parser)
      constant.rs - ConstantParser<T,V> (.constant(value) — replace matched value with a clone, V: Clone)
    repeater/
      possessive.rs  - PossessiveRepeatingParser<T> { min, max }
      separated.rs   - SeparatedRepeatingParser (rep_sep/star_sep/plus_sep)
      lazy.rs        - LazyRepeatingParser<T, TC> { delegate, limit, min, max }
    predicate.rs  - PredicateParser, string(&str), string_ignore_case(&str)
    regex.rs      - RegexParser, RegexMatch, regex(&str)
    ext.rs        - ParserExt<T> extension trait: map, flat_map, rep, times, star, plus, opt,
                    token, trim, input, input_with_message, only_if, only_if_with_message,
                    only_if_with_factory, all_matches, and, not, labeled, skip,
                    skip_left, skip_right, end, to,
                    rep_sep, star_sep, plus_sep,
                    rep_lazy, star_lazy, plus_lazy
    misc/
      newline.rs
      end.rs      - EndOfInputParser, eof(), eof_with_message()
      epsilon.rs  - EpsilonParser, epsilon(), epsilon_with(T)
      success.rs  - SuccessParser, success(T)
      failure.rs  - FailureParser, failure(), failure_with_message(String)
      label.rs    - LabeledParser, .labeled(label) via ParserExt
      position.rs - PositionParser, position()
  matcher/
    matches.rs    - MatchesIterator<T,P>, MatchesIterable<T,P> (IntoIterator)

rust-petitparser-macros/src/
  lib.rs        - #[grammar] proc-macro attribute

benches/
  parse_grammars.rs - criterion parse/build benchmarks over the Dart + Smalltalk grammars
```

## Storage model: `Rc<dyn Parser<T>>` everywhere + `HasChildren` (current design)
The combinator core was redesigned (landed on `main`, commits `363633f`/`6d73303`; an abandoned
alternative was evaluated and discarded, see below). Two intertwined changes:

**1. `Parser<T>` storage is `Rc<dyn Parser<T>>`, not a monomorphized generic.** Every combinator
that holds a sub-parser stores it as `Rc<dyn Parser<T>>` (dynamic dispatch) rather than a bare
generic `P: Parser<T>` (static dispatch). Consequences:
- The trait gained two supertraits: `pub trait Parser<T: 'static>: Debug + HasChildren + 'static`.
  The `'static` (on both `T` and `Self`) is what lets `ParserExt` methods do `Rc::new(self)` →
  `Rc<dyn Parser<T>>` *without* restating `Self: 'static` on every method — satisfying `Parser<T>`
  already implies it. `HasChildren` is the reflection hook (see below).
- Combinator structs dropped their parser-type generic: `MapParser<T, P, F>` → `MapParser<T, F>`,
  `ConstantParser<T, P, V>` → `ConstantParser<T, V>`, `LabeledParser<P, T>` → `LabeledParser<T>`,
  `SkipParser<A, Aft, B, Bef, P, T>` → `SkipParser<Aft, Bef, T>`, `PossessiveRepeatingParser<P>` →
  `<T>`, etc. The `Seq2-9` macro is generic over the value types (`Seq3<T1, T2, T3>`) and `Choice2-11`
  over the single shared `T` (`Choice3<T>`); both got manual `Clone` impls (cheap `Rc::clone`,
  bound-free) instead of `#[derive(Clone)]` (which would spuriously require `T: Clone`). `FlatMapParser`
  keeps its `P2` generic — that parser is produced on the fly by the closure, not stored.
  `ContinuationParser`/`SettableParser`/`expression/*` already stored `Rc<dyn Parser>` and were
  unchanged. Leaf parsers (char/regex/predicate/position/success/epsilon/newline/eof/failure/
  `CharacterRepeatingParser`) hold no delegate and were unchanged structurally.
- `ParserExt` methods wrap `self` (and any passed-in delegate like `before`/`sep`/`limit`) in
  `Rc::new(..)`. `impl Trait` arguments already imply `'static` via the `Parser` supertrait, so no
  extra bounds were needed there.

**The performance tradeoff (measured, deliberate).** Dynamic dispatch costs **~20% parse time on
backtracking-heavy *recognizer* grammars** (Dart: 73→86 ms / 290→352 ms / 578→694 ms at 40/160/320
units — scale-invariant, since the cost is one vtable hop + lost inlining *per `parse_on` call* and
call count scales with input), and **~0% on AST-building grammars** (Smalltalk: statistically flat).
Build (grammar construction) time is 2–3× (every layer heap-allocs an `Rc`) but in absolute µs,
amortized over all parses → irrelevant. Chosen knowingly: the win is dramatically simpler code,
smaller combinator types, faster compiles, and a near-free `HasChildren`. NOTE: the catastrophic
exponential backtracking some grammars exhibit (e.g. the Dart grammar on real-world input) is an
*orthogonal* no-memoization (non-packrat) property — `ref0`-style object sharing (our
`SettableParser`/`#[grammar]`) avoids combinatorial explosion at graph *construction*, not at
*evaluation*; the `Rc` change does not affect evaluation count at all. Bench harness:
`benches/parse_grammars.rs` (run `cargo bench`; uses criterion, a dev-dependency).

**2. `HasChildren` — required reflection supertrait (linter groundwork).**
`pub trait HasChildren: Debug { fn children(&self) -> Vec<Rc<dyn HasChildren>>; }`, a *required*
supertrait of `Parser<T>` (no opt-out, no default impl). It is deliberately **not** generic over
the value type: a parser's children may each produce a different `T`, so a tree-walker needs one
non-generic `dyn` handle to recurse through — this trait *cannot* be folded into `Parser<T>` and
cannot be eliminated, only made mandatory (which is what gives universal linter support).
- Combinator `children()` returns its stored delegate(s) `Rc<dyn Parser<Ti>>` upcast to
  `Rc<dyn HasChildren>` — a one-liner, because of the `Rc` storage. The upcast is via **trait
  upcasting** (stable Rust 1.86+, available on edition 2024); `vec![self.delegate.clone()]` coerces
  without an explicit `as`, since the fn return type propagates as the expected element type making
  the array literal a coercion site. Multi-delegate combinators list all (e.g. `SkipParser` →
  before/delegate/after; repeaters → delegate + limit/separator). Leaf parsers return `vec![]`.
- `SettableParser`/`SettableParserRef` return their current wrapped delegate; the `#[grammar]` macro
  emits `children()` for the generated grammar struct (returns the start rule — a linter walks
  transitively from there). Exported from the prelude.
- **A real linter walk must dedupe by `Rc::as_ptr` identity** — recursive grammars are cyclic via
  `SettableParserRef`, so naive recursion loops forever. `tests/has_children_tests.rs` only deep-walks
  *acyclic* parsers for this reason.
- **Custom `Parser` authors must impl `HasChildren`** (the one obligation the supertrait imposes).
  The only hand-written custom parser in the examples, `TabularDefinition` (`tabular.rs`), has a
  manual impl returning its four config sub-parsers — the template to follow.
- The **linter rules themselves are NOT built yet** — this is only the `children()` reflection
  foundation that unblocks them (dart's `linter_rules.dart`: unresolved-settable, nullable/unbounded
  repeater, left-recursion, unreachable-choice-alternative, …).

**Abandoned alternative (was branch `rc-everywhere`, commit `e3f3aaf`, since deleted).** The other
way to get `HasChildren` was to keep monomorphized generic storage and widen ~390 call sites with
`Clone + 'static` bounds (so `children()` could `Rc::new(self.delegate.clone())`). Zero runtime
cost but pervasive bound noise. Benchmarked the `Rc` approach against it, chose `Rc` for the
simpler code, and deleted both the `rc-everywhere` and `rc-storage` work branches once `rc-storage`
landed on `main` — this paragraph is the only remaining record of the alternative.

## Key Design Decisions
- `Parser<T>` is generic over `T` (not an associated type), and carries supertraits:
  `pub trait Parser<T: 'static>: Debug + HasChildren + 'static` (see "Storage model" above for the
  `'static`/`HasChildren` rationale). Structs still carry `PhantomData<T>` where `T` would otherwise
  only appear in a `where` clause — see `MapParser`.
- `ParserExt<T>: Parser<T> + Sized where T: Debug + 'static` extension trait provides method syntax. Blanket impl covers all parsers.
- `accept(input: &str)`/`accept_at(input: &str, start)` take `&str` and build the `Rc<[char]>` buffer
  internally, matching both dart's actual signature and this crate's own `Parser::parse(&str)`
  convenience wrapper — these are one-shot leaf calls, never invoked elsewhere in the codebase with a
  pre-existing buffer, so there's no reuse to preserve. `all_matches` deliberately stayed on
  `Rc<[char]>` instead of getting the same treatment: `core/token.rs`'s `line_and_column_of` calls it
  internally on a buffer it already holds (the original parse buffer, shared via `Rc`), and forcing
  `&str` there would mean collecting that buffer into a `String` just to re-split it back into an
  `Rc<[char]>` inside `all_matches` — a wasteful, lossy round trip purely to satisfy the signature.
  Same combinator, two different call patterns, two different right answers.
- Blanket `impl<T: 'static, P: Parser<T> + ?Sized> Parser<T> for Rc<P>` (in `core/parser.rs`) — lets `Rc<P>` and `Rc<dyn Parser<T>>` be used as parsers (delegates `parse_on`/`fast_parse_on`). Needed to share a sub-parser across multiple combinators via `Rc::new(..).clone()` (leaf parsers aren't generally `Clone`; combinators are now `Rc`-backed). `Rc<P>` is `Sized`, so it auto-gets `ParserExt` too. A parallel blanket `impl<P: HasChildren + ?Sized> HasChildren for Rc<P>` delegates `children()`.
- `choice_impl!` macro uses `Option<Failure>` accumulation pattern (avoids needing to separate "first" from "rest").
- `impl_seq!` macro uses `?` operator with sequential context threading.
- `MatchesIterable` implements `IntoIterator` → `MatchesIterator`; supports `overlapping` flag.
- Repetition parsers (`star`, `plus`) include an infinite-loop guard: if the inner parser succeeds without advancing position, break.
- `PredicateParser.length` is in **chars** (`.chars().count()`), not bytes — required for Unicode/emoji correctness.
- `AndParser` returns `Parser<T>` — preserves the matched value but resets position to original (lookahead).
- `NotParser` returns `Parser<Failure>` — inner failure becomes the success value. Error message: `"Expected failure, got success: {:?}"` on the value.
- `SettableParser` uses `Rc<RefCell<Rc<dyn Parser<T>>>>` — the "owner" (strong Rc). No `Option`:
  `undefined()` seeds the slot with an `UndefinedParser<T>` sentinel (always fails with a
  configurable message, default `"undefined parser"`) instead of leaving it absent, mirroring
  dart's `undefined<R>() => failure<R>(message: message).settable()`. This means a forgotten
  `.set()` call surfaces as an ordinary parse failure (`assert_failure!`-testable, see
  `combinator_tests.rs`'s `settable_undefined_fails_gracefully_until_set`) instead of the
  `.expect("SettableParser delegate not set")` panic this used to have.
- `SettableParserRef` uses `Weak<RefCell<...>>` — the "embedded reference" that breaks Rc cycles.
  Still panics via `.expect("SettableParser owner dropped")` if the owning `SettableParser` itself
  was dropped — a different, more severe failure mode (use-after-free-shaped) than "forgot to
  call `.set()`", deliberately left as a panic since dart's GC-based design has no equivalent
  concept to compare against.
- Call `.borrow()` on `SettableParser` to get a `SettableParserRef` for embedding in sub-parsers.
- Rule: use `.clone()` for forward references (more complex → simpler); use `.borrow()` only for back-references (the one that creates the cycle). The strong chain from the returned root parser keeps all intermediate parsers alive.

## Where `T: Clone` Is Required (flag in release notes)

**Rule of thumb:** a parser needs `T: Clone` exactly when it manufactures a value that
isn't derived from the input and has to be ready to hand that value out more than once
across separate `parse_on(&self, ...)` calls (re-parses, retries inside a repetition, reuse
of the same built grammar). Parsers that only move/transform a value already produced by a
delegate (`map`, `flat_map`, `seq*`, `choice*`, `star`/`plus`/`rep`, etc.) never need it —
there's nothing being reproduced, just passed through. This split is deliberate, not an
accident: we explicitly considered and rejected putting `Clone` on `Parser<T>` or
`ParserExt<T>` as a blanket bound (would lock out legitimate non-`Clone` value types — e.g.
an AST node wrapping a `Box<dyn Trait>` — from the entire library, including combinators
that never clone anything) in favor of keeping it scoped to the specific structs/methods
that actually need it, mirroring how `ParserExt<T>: Parser<T> + Sized where T: Debug` puts
its one blanket bound at the extension-trait level but leaves `Clone` to individual structs.
Current/planned call sites (**this list is exactly what release notes/public docs should
call out as a gotcha** — a user defining a custom value type only discovers this the moment
they reach for one of these, so it's worth being upfront about it rather than letting people
hit the compiler error cold):
- `success(value: T)` — `SuccessParser<T: Clone + Debug>` (`src/parser/misc/success.rs`).
- `.constant(value)` — `ConstantParser<T, V> where V: Clone + Debug`
  (`src/parser/action/constant.rs`); note it's the *replacement* value type `V` that needs
  `Clone`, not the delegate's `T`.
- `.opt_with(value)` / a from-scratch `OptionalParser<T, P>` (dart's `optionalWith` /
  `OptionalParser`) — needs `T: Clone` for the same reason as `success`. **Not yet fixed as
  of this writing**: the current `opt_with` body (`self.rep(0, Some(1)).map(move |vec| ...
  unwrap_or(value))`) doesn't compile for non-`Copy` `T` at all (moves `value` into a `Fn`
  closure) — needs a per-method `where T: Clone` bound plus `.clone()` inside, or better,
  rewriting it as a dedicated struct matching dart's actual `OptionalParser` (`delegate` +
  `otherwise` fields, direct `parse_on`/`fast_parse_on`) instead of composing `.rep()`/`.map()`.
- `ExpressionGroup::build_optional`/`build` and `ExpressionBuilder::build` (implemented,
  `src/expression/`) — **transitively and unconditionally** need `T: Clone`, because `build()`'s
  pipeline always calls `build_optional` as its last step
  (mirroring dart's `_buildOptional(_buildLeft(...))`, which *also* always calls it)
  regardless of whether any individual group ever calls `.optional(...)`. This means anyone
  who calls `ExpressionBuilder::build()` to finish *any* grammar needs `T: Clone`, even if
  they never use `.optional()` — worth calling out explicitly since it's a non-obvious
  transitive consequence, not something visible from `ExpressionBuilder`'s own public API
  surface. Scope the bound to `build`/`build_optional` specifically (not the whole
  `impl<T: Debug + 'static> ExpressionGroup<T>` block) so the registration methods
  (`wrapper`/`prefix`/`postfix`/`left`/`right`/`optional`) stay `Clone`-free.
- Dart's `Parser.copy()` is **not** related to this — it copies the parser *graph* (used by
  dart's reflection/`replace()`/`resolve()` machinery, e.g. how dart's
  `ExpressionBuilder.build()` patches `_loopback` references after the fact), not the parsed
  *value* type. We have no equivalent and don't need one — our `SettableParser`/`.borrow()`
  weak-ref trick achieves the same recursive-grammar result without any tree-walking/copying.

## Testing
- Uses `googletest` crate: `#[gtest]`, `assert_that!`, `eq`, `not`
- Tests split across multiple files: `character_tests.rs`, `combinator_tests.rs`, `repeater_tests.rs`,
  `action_tests.rs`, `matcher_tests.rs`, `misc_tests.rs`, `has_children_tests.rs` (the `HasChildren`
  tree-walk tests — see "Storage model")
- Each test file defines `assert_success!(parser, input, value, pos)` and `assert_failure!(parser, input, message, pos)`
  macros that check both `parse_on` and `fast_parse_on`
- Example grammars: `tests/example-grammars/main.rs` (entry point) +
  `json.rs` + `expr.rs` + `bibtex.rs` + `pascal.rs` + `dart.rs` + `smalltalk/{ast,mod}.rs`
  (written with the `#[grammar]` proc macro — each has genuinely recursive rules) + `tabular.rs` +
  `uri.rs` (hand-written — no recursion, so no macro/`SettableParser` needed) + `math.rs` (uses
  `ExpressionBuilder`, no `#[grammar]` needed since it has no recursive rules of its own)
- 573 tests passing — includes `bibtex::scg_bib_size_and_round_trip`, which makes a real network
  call (fetches ~9600-entry `scg.bib` from GitHub) on every `cargo test` run, by deliberate choice
  (~3s, judged worth it for the coverage; not `#[ignore]`d). Needs `ureq` (dev-dependency).

## What's Implemented
- Character parsers: `any`, `char`, `char_ci`, `letter`, `digit`, `digit_with_radix`, `one_of`, `one_of_ci`,
  `none_of`, `none_of_ci`, `lowercase`, `uppercase`, `whitespace`, `word`, `predicate`, `range(start, end)`
  (validates `start <= end` at construction time — same as dart's `RangeCharPredicate` constructor —
  not at match time, both for parity and so the invalid-range case is a plain panic-on-construction
  test rather than needing an actual parse attempt to trigger). `any_of`/`any_of_ci`
  exist too, as deliberate thin aliases for `one_of`/`one_of_ci` (`src/parser/character.rs`) — dart's actual
  name has always been `anyOf`/`noneOf` (never `oneOf`), but the project keeps `one_of` as the primary name
  since "any of" reads as "one-or-more", which isn't what this parser does (it matches exactly one
  character from the set). The alias exists purely so programmers coming from dart can find the function
  under the name they already know.
- `pattern(&str)`, `pattern_ci(&str)` — char-class primitive (e.g. `pattern("a-zA-Z0-9_-")`, `^` negates).
  `CharKind::{Pattern,PatternCi,NegatedPattern,NegatedPatternCi}` each hold `Vec<RangeInclusive<char>>`
  (negation is a separate variant, not a bool field — mirrors the existing `OneOf`/`NoneOf` split).
  Parsed by a flat 3-char sliding-window scan (no combinator bootstrapping like dart's). `Ci` variants
  store the ranges as originally parsed and fold case at match time (`OneOfCi`-style), rather than
  doubling the ranges up front the way dart does. No `unicode:` flag (dart's only exists to work
  around UTF-16 surrogate pairs; Rust's `char` is already a full Unicode scalar value). Default
  description is derived from the stored ranges (`ranges_to_string`), not the original pattern text —
  reuses `char`'s built-in `Debug` escaping instead of porting dart's `toReadableString`.
- `Seq2`–`Seq9`, `Choice2`–`Choice11` (with configurable failure joiner: `SELECT_FARTHEST_JOINED`).
  `Choice10`/`Choice11` exist solely because Pascal's `statement` rule has 11 alternatives — no
  `Seq10`+ needed yet (max real usage so far is `seq8`).
- `map`, `flat_map`, `rep`, `times`, `star`, `plus`, `opt`, `token`, `trim`, `trim_with(before, after)`,
  `input`, `all_matches`. `trim_with` is `trim()`'s generalization to caller-supplied delimiter
  parsers (`seq3(before, self, after).map(|(_, val, _)| val)`, requires `Bef: Debug, Aft: Debug`) —
  added for Pascal's `spacer()`-based token trimming (dart's `.trim([custom])`).
- `only_if`, `only_if_with_message`, `only_if_with_factory`
- `and()`, `not()` lookaheads
- `rep_sep`, `star_sep`, `plus_sep` (separated repeaters)
- `rep_lazy`, `star_lazy`, `plus_lazy` (lazy repeaters with limit parser)
- `rep_greedy`, `star_greedy`, `plus_greedy` (greedy repeaters: consume as much as possible, then
  backtrack one element at a time until `limit: impl Parser<()>` succeeds — limit's value type is
  fixed to `()`, unlike the lazy variants' `impl Parser<L>`, so non-`()` limiters like `digit()`
  need `.constant(())` first). `GreedyRepeatingParser` (`src/parser/repeater/greedy.rs`). Ported from
  dart's `GreedyRepeatingParser`, which `LimitedRepeatingParser`s into a min-loop (hard-fails via `?`
  if it can't even reach `min` — correct, since there's no valid backtrack point below `min`) then a
  max-loop that grows `elements`/`contexts` (or `count`/`positions` in `fast_parse_on`) in lockstep,
  followed by a pop-one-and-retry search against `limit` once growth stops. Two bugs fixed during
  test-porting (both in the original, not introduced while porting): (1) `parse_on`'s max-loop used
  `self.delegate.parse_on(&current)?`, which propagated the delegate's *normal* termination failure
  as the overall result instead of breaking out of the greedy-growth loop and falling through to the
  backtrack search — this made every greedy parser fail outright the moment the input ran out of
  matching characters (i.e. almost always). Fixed to `match ... { Ok(_) => ..., Err(_) => break }`.
  (2) `fast_parse_on`'s max-loop called `self.fast_parse_on(...)` (itself, recursively) instead of
  `self.delegate.fast_parse_on(...)` — would have silently swallowed multiple elements per loop
  iteration instead of one. Both confirmed fixed by hand-tracing the dart test matrix position-by-
  position before writing `repeater_tests.rs`'s `star_greedy`/`plus_greedy`/`repeat_greedy` tests,
  then verifying all assertions passed on the first run.
- `skip(open, close)` — wraps parser between delimiters, returns inner value
- `skip_left(before)`, `skip_right(after)`, `end()` — variants of skip
- `constant(value)` — replaces the matched value with a clone of `value` (`V: Clone + Debug`)
- `labeled(label)` — replaces failure message
- `string(&str)`, `string_ignore_case(&str)` via `PredicateParser`
- `eof()`, `eof_with_message()` via `EndOfInputParser`
- `epsilon()`, `epsilon_with(T)`, `success(T)`, `failure()`, `failure_with_message(String)`
- `position()` — returns current position as `usize` without consuming
- `SettableParser<T>` / `SettableParserRef<T>` for recursive grammars (cycle-free).
  `SettableParser::undefined()` / `undefined_with_message(String)` — gracefully fails
  (`"undefined parser"` by default) instead of panicking if used before `.set()` (see Key
  Design Decisions for the sentinel-parser mechanism).
- `line_and_column_of`
- `ExpressionBuilder<T>` / `ExpressionGroup<T>` (`src/expression/`, exported via prelude) —
  precedence-climbing grammar builder ported from dart-petitparser's `lib/expression.dart`:
  `primitive()`, `group()`, `wrapper()`, `prefix()`, `postfix()`, `left()`, `right()`,
  `optional()`, `build()`. Tested by `tests/expression_tests.rs` (49 tests, ported from dart's
  own `test/expression_test.dart`). See "What's Next" for the bugs found while porting (group()
  losing registrations, build() leaking a dangling weak ref, build_right's repetition shape,
  choice_of's failure-joiner) and "Where `T: Clone` Is Required" for the transitive `Clone`
  bound on `build()`.
- JSON example grammar (full, with recursive `SettableParser`)
- Arithmetic expression grammar (`tests/example-grammars/expr.rs`)
  - Integer arithmetic: `+`, `-`, `*`, `/` with correct precedence and left-associativity
  - Parenthesized subexpressions via recursive `SettableParser`
  - `fold_ops` pattern: `fn(&(f64, Vec<(char, f64)>)) -> f64` with left fold
  - Note: `SkipParser` does NOT need `T: Clone` — removed that bound
- `#[grammar]` proc macro (`rust-petitparser-macros`) — replaces manual SettableParser
  boilerplate; drives both the expr and JSON example grammars
- `star_string()`/`plus_string()`/`times_string(n)`/`rep_string(min, max)` — character
  repeaters that flatten directly to `String` (dart's `starString`/`plusString`/`timesString`/
  `repeatString`, `lib/src/parser/repeater/character.dart`). Two-tier design, mirroring how dart
  special-cases `SingleCharacterParser`/`PredicateCharacterParser` via `is`-checks for a fast
  path while falling back to a generic `flatten(repeat(...))` otherwise:
  - **Fast path**: `CharacterRepeatingParser` (`src/parser/repeater/character.rs`) — holds
    `test: Rc<dyn Fn(char) -> bool>` (not `CharKind`, so the one struct can serve both
    `CharParser` and `PredicateCharParser` — `CharParser`'s override wraps `kind.matches` in a
    closure, `PredicateCharParser`'s passes `self.test` straight through) plus `description:
    String` + `message: Option<String>` and its own `message_for`, computed at failure time from
    whatever character (or lack thereof) is at the position the scan actually stopped at —
    structurally identical to `CharParser`'s/`PredicateCharParser`'s own `message_for`, and a
    deliberate departure from dart's flat, unframed `"X expected"` messages here, same as
    everywhere else in this port. Needs a manual `Debug` impl (closure field isn't `Debug`),
    modeled on `PredicateCharParser`'s existing one.
  - **Generic fallback**: `CharacterRepeatingParserExt: Parser<char> + Sized` trait
    (`src/parser/ext.rs`), blanket-implemented for every `Parser<char>`, with `rep_string`
    defaulting to `self.rep(min, max).input()`. Exported via `prelude.rs`.
  - **The dispatch mechanism**: `CharParser`/`PredicateCharParser` each get plain *inherent*
    methods of the same four names (in their own `impl CharParser { ... }`/`impl
    PredicateCharParser { ... }` blocks) that construct `CharacterRepeatingParser` directly —
    *not* a trait impl. Inherent methods win over trait methods at the same receiver kind (both
    take `self` by value here), so `char('a').star_string()` resolves to the fast path while
    `char('a').map(|c| c).star_string()` (anything without its own inherent override) falls
    through to the trait default. This is the zero-runtime-cost alternative to dart's `self is
    SingleCharacterParser` runtime check — no `Any`/`downcast_ref` needed.
  - **Bug found while wiring this up**: fixing an `impl CharacterRepeatingParserExt for CharParser
    { }` collision (conflicted with the blanket impl below — that block needed to be deleted
    entirely in favor of inherent methods, not fixed in place) also deleted the blanket impl
    itself (`impl<P: Parser<char>> CharacterRepeatingParserExt for P {}`) by mistake. Symptom was
    *not* an error at the deletion site — the crate kept building cleanly, since nothing called
    `star_string`/etc. yet. It only surfaced once a test tried to exercise the fallback path on a
    type with no inherent override (`char('a').map(|c| c).plus_string()`), with a misleading
    `E0599: method not found` pointing at `MapParser`. Chased as a `MapParser`/`Rc<dyn
    Parser<char>>`-specific trait-resolution puzzle for a while (built minimal repros to compare
    `ParserExt<T>`'s working blanket impl against this one) before noticing the blanket impl
    itself was simply gone from `ext.rs`. Lesson: when a generic/blanket-impl-backed method
    suddenly "isn't found" for an unrelated type, check that the blanket impl itself still exists
    before assuming a deeper trait-resolution issue.
  - Tested in `tests/repeater_tests.rs`'s `string` group (ported from dart's
    `parser_repeater_test.dart`'s `string` group) — `star_string_test`, `plus_string_test`,
    `times_string_test`, `rep_string_test`, `rep_string_unbounded_test` (100k-char stress test,
    matching the existing `rep` unbounded stress test's precedent), `any_plus_string_test`, and
    `plus_string_fallback_test` (exercises the generic path via `.map()`, replacing dart's
    `.settable()` — simpler here since we don't need a real recursive-grammar setup just to get a
    non-`CharParser` `Parser<char>`). Dropped from the port: the `isA<RepeatingCharacterParser>()`
    checks (no reflection/type-introspection available to assert "the fast path was taken"); `any
    (unicode)` (no `unicode:` flag concept in this port); `repeat erroneous` (dart asserts `min >=
    0`/`max >= min`, but no repeater in this codebase validates that invariant, so adding it here
    alone would be new scope, not a port).
- `SeparatedList<T, Sep>` (`src/parser/repeater/separated.rs`, exported via prelude) — `rep_sep`/
  `star_sep`/`plus_sep`/`times_sep` deliberately diverge from dart here: dart's actual
  `starSeparated`/`plusSeparated`/`timesSeparated`/`repeatSeparated` always return a
  `SeparatedList<R, S>` (elements *and* separators kept distinct); ours instead return a
  flattened `Vec<T>` (separators discarded) for the common "I don't care about separators" case
  (CSV values, function-call arguments), with a *separate* `rep_with_sep`/`star_with_sep`/
  `plus_with_sep`/`times_with_sep` family added for when separators matter (e.g. operands
  separated by distinct operators). The flattened family is a thin composition over the rich
  one — `rep_sep(sep, min, max, trailing)` is exactly
  `rep_with_sep(sep, min, max, trailing).map(|sl| sl.elements)` — so the interleaving loop exists
  exactly once; `.map()`'s `fast_parse_on` already skips the closure entirely (see the
  `fast_parse_on` side-effect-gap fix below), so the flattened family costs nothing extra on the
  fast path.
  - **`Trailing` enum** (`Disallowed`/`Allowed`/`Required`) — a deliberate, user-requested
    extension beyond dart, which has no trailing-separator concept at all (its current
    `SeparatedRepeatingParser.parseOn` always discards a dangling separator and rewinds, same as
    our `Disallowed`). Added because real grammars commonly allow (Rust argument lists) or even
    require a separator after every element including the last. Chosen as a 3-variant enum
    instead of a `bool` specifically to avoid boolean blindness at call sites
    (`star_sep(sep, Trailing::Allowed)` reads at the call site; `star_sep(sep, true)` doesn't).
  - **Where trailing is actually resolved — two places, not one.** The first instinct (mine,
    initially wrong) was "leave the existing min/max loops completely untouched and tack on one
    extra step at the very end that tries to consume one more bare separator." That tack-on step
    *is* necessary and present (`SeparatedListRepeatingParser::parse_on`'s final `match
    self.trailing` block) — it's what covers `min == max` (the max-loop body never executes at
    all when `times_sep`-style fixed counts are already satisfied by the min-loop) and "no
    separator found at all" (the max-loop's own separator-level `break`). But it is *not* where
    the common case gets resolved: when a separator succeeds and the *following* element then
    fails — which is the literal definition of "found a trailing separator" — the max loop's
    existing logic takes an early `return` (discard the separator, rewind to before it,
    `Disallowed`-style) before the function ever reaches the tack-on step. A pair of tests
    written specifically to exercise this (`"a,b,c,"` with `Trailing::Allowed`/`Required`) caught
    it immediately: both came back with the separator discarded exactly as if `Disallowed` had
    been passed. Fixed by giving that early-return branch its own three-way fork — `Disallowed`
    pops the separator and rewinds to `previous` (unchanged), `Allowed`/`Required` keep it and
    return at `current` (which already reflects the separator having been consumed) instead of
    popping/rewinding. `fast_parse_on`'s mirror needed the identical fork (`previous` vs.
    `current`, position-only). Lesson for next time a "loops stay the same, add one step at the
    end" design is proposed for an interleaved repeat/separator structure: identify *every* early-
    return inside the loop first — each one is a chance for new state to be resolved on a path
    that never reaches the tack-on.
  - **Empty-match + `Required` is not a contradiction.** A second real bug, caught by its own
    dedicated regression test: the trailing-probe step originally ran unconditionally, so
    `star_with_sep(sep, Trailing::Required)` on empty input hard-failed just because no separator
    was found — even though zero elements vacuously satisfies "every element is followed by a
    required separator" (the same way `star()` always succeeds on empty input). Fixed by guarding
    both the early-return branch's and the tack-on's `Allowed | Required` arms with
    `!elements.is_empty()` (`parse_on`)/`count > 0` (`fast_parse_on`) — already present
    one level up in the same loops as the pattern to follow.
  - Tested in `tests/repeater_tests.rs`: `times_sep_matches_exact_count`/`_fails_when_too_few`/
    `_ignores_extra_elements` plus the `with_sep` group (`star_with_sep_test`/`plus_with_sep_test`/
    `times_with_sep_test`/`rep_with_sep_test`, ported from dart's `parser_repeater_test.dart`'s
    `separated` group with `Trailing::Disallowed`, messages translated to this project's "expected
    X, but found/reached Y" convention) plus a dedicated `Trailing` matrix for both the flattened
    and rich families (`sep_trailing_*`/`with_sep_trailing_*`) covering `Allowed` consuming the
    trailing separator, `Disallowed` stopping before it, `Required` failing without one,
    `Required` succeeding vacuously on an empty match, and the `min == max` edge case for both
    `Allowed` and `Required`.
  - **`SeparatedList`'s own utility methods: done** — `sequential()`, `fold()`/`rfold()` (named to
    match `Iterator::fold`/`DoubleEndedIterator::rfold`'s own convention rather than dart's
    `foldLeft`/`foldRight`), and `Display`. Bounds are scoped to the method that actually needs
    them, not the whole `impl<T, Sep> SeparatedList<T, Sep>` block — `sequential()` only ever
    borrows (`&self.elements[i]`/`&self.separators[i]`), and `fold`/`rfold` consume `self` and
    move `T`/`Sep` by value into the callback (mirroring `Token::join`'s `Clone`-avoidance
    reasoning), so neither needs any bound at all; only `Display` needs `T: Display, Sep:
    Display`. `sequential()` returns `impl Iterator<Item = Interleaved<&T, &Sep>>` — a small new
    public enum (`Element(T)`/`Separator(S)`) standing in for dart's untyped `Iterable<dynamic>`,
    since Rust has no union type for "R or S" — built eagerly into a `Vec` then `.into_iter()`'d
    rather than as a true lazy generator (`std::iter::from_fn` with hand-tracked index/flag state
    is the closest stable-Rust analogue to dart's `sync*`/`yield`, since generators aren't stable;
    not worth the complexity here since a finished parse's `SeparatedList` is never unbounded).
    `fold`/`rfold` panic on an empty list (`"Can't call fold/rfold on an empty SeparatedList"`),
    matching dart's `throwsStateError`, and *deliberately* still require exactly one fewer
    separator than element (reject a trailing separator) even though the struct itself now allows
    `separators.len() == elements.len()` via `Trailing::Allowed`/`Required` — a trailing separator
    is data fed into the fold callback with no right-hand operand to combine with (e.g. `2 + 3 *`
    parsed with a trailing `*`), almost certainly a mistake worth surfacing rather than silently
    dropping, unlike a trailing comma in a parsed CSV/argument list which carries no value at all.
    Two real bugs found and fixed while implementing this, both in the same assert:
    `self.elements.len() - 1 == self.separators.len()` had the comparison backwards (asserts
    elements has *one fewer* than separators, the opposite of the intended relationship — failed
    on the ordinary 3-element/2-separator case) and underflowed outright on the smallest non-empty
    case (single element, zero separators: `0usize - 1` panics with "attempt to subtract with
    overflow" before the comparison ever runs). Fixed to `self.elements.len() - 1 ==
    self.separators.len()` — the subtraction is underflow-safe here specifically because it only
    executes after the preceding `!self.elements.is_empty()` assert already guarantees
    `elements.len() >= 1`. A third bug
    surfaced only once the `toString`/`Display` test was being ported: the original `Display`
    impl formatted each `Interleaved<&T, &Sep>` item via `{:?}` (`Debug`) on the whole enum,
    producing `SeparatedList(Element("1"), Separator("+"), Element("2"))` instead of dart's clean
    `SeparatedList(1, +, 2)` — fixed by requiring `T: Display, Sep: Display` instead of `Debug`
    and pattern-matching to format just the inner value with `{}` before joining.
  - Tested in `tests/repeater_tests.rs`'s `separated_list_*` group, ported from dart's
    `parser_repeater_test.dart`'s `separated list` group (`elements`/`separators`/`sequential`/
    `fold`/`rfold`/`display`, using `fold`/`rfold` for dart's `foldLeft`/`foldRight` and `display`
    for dart's `toString`). Not ported: dart's exact `toString` assertions, which bake the generic
    type arguments into the string via `$runtimeType` (e.g. `"SeparatedList<String,
    String>(...)"`) — no clean Rust equivalent, so the ported test checks our actual
    `"SeparatedList(...)"` format (sans type arguments) instead of dart's substring-only check.
- **`map2`..`map9`** (`src/parser/ext.rs`, `impl_map_tuple!` macro, mirroring `impl_seq!`'s
  per-arity-invocation pattern) — lets a `seqN(...)` result's tuple be mapped with an N-ary
  closure (`seq3(a, b, c).map3(|x, y, z| ...)`) instead of having to destructure a tuple pattern
  inside a single-argument `.map(|(x, y, z)| ...)`. Each is a trait (`MapTuple2`..`MapTuple9`)
  blanket-implemented over *any* `Parser<(T1, ..., Tn)>`, not just literal `SeqN` results, so it
  also works after e.g. `seq3(...).trim()`. Each method just delegates to the existing `.map()`:
  `self.map(move |(t1, ..., tn)| f(t1, ..., tn))`. Two gotchas hit while writing the macro itself:
  (1) the first draft used one metavariable (`$value`) for both the type parameter name (`T1`)
  and the closure's pattern-binding name, since macro substitution is purely textual — this
  compiled, but every generated method's tuple-destructuring closure ended up with parameters
  literally named `T1`/`T2`/etc., tripping `non_snake_case` warnings across every call site. Fixed
  by splitting into two metavariables per slot (`$value` for the type, `$field` for the lowercase
  binding), exactly mirroring `impl_seq!`'s own `($parser, $value, $field)` triples. (2) the
  blanket impl needs to restate the `where $value: Debug` bound that appears on the trait
  declaration — calling `.map()` internally requires `ParserExt<T>: ... where T: Debug`, and
  tuples are `Debug` automatically once every element is, but only if that bound is repeated on
  the `impl` block too, matching `ParserExt`'s own existing blanket impl
  (`impl<T, P: Parser<T>> ParserExt<T> for P where T: Debug {}`) rather than just the trait line.
  Existing `.map(|(...)| ...)` call sites across the codebase were swept and converted to the
  matching `mapN` where the receiver was a genuine `Parser<(...)>` (every `seqN(...)` result, plus
  a couple of `seq2(...).trim()`/nested-`seq2`-of-`seq2` cases) — deliberately *not* converted:
  several spots in `tests/example-grammars/uri.rs` that look identical syntactically
  (`.map(|(a, b)| ...)`) but are actually `Option::map`/`Option::and_then` on an already-`.opt()`'d
  value, which `mapN`'s blanket impl (scoped to `Parser<(...)>`) doesn't cover.
- **`seq!` variadic macro** (`rust-petitparser-macros/src/seq.rs`, function-like `#[proc_macro]`,
  re-exported via `prelude.rs` alongside `grammar`) — `seq!(a, b, c)` expands to `seq3(a, b, c)`,
  picking the right `seqN` by *counting* the comma-separated expressions in the macro's input,
  rather than `macro_rules!` having to declare one match arm per arity (the approach used for
  `impl_seq!`/`impl_map_tuple!`, which generate code at the *definition* site where the arity is
  always known up front — `seq!`'s arity is only known at each *call* site, which is what makes
  this a genuinely different problem needing real parsing, not just textual substitution).
  Implementation: parses the input as `syn::punctuated::Punctuated<Expr, Token![,]>` (the type
  syn itself uses for comma-separated lists — notably, this is *not* `syn::ExprCall`, which
  parses a complete call expression `name(args)` including a callee name; `seq!`'s input has no
  callee, just a bare list), counts elements via `.len()`, builds the target function name with
  `quote::format_ident!("seq{}", n)`, and validates `n` is in `2..=9` — emitting a real
  `syn::Error::to_compile_error()` for out-of-range counts instead of relying on `macro_rules!`'s
  comparatively cryptic "no rules expected this token" failure (one concrete win of going
  proc-macro for this over the `macro_rules!` design originally sketched).
  One real bug during implementation: the first draft built the call via
  `quote! { #function_name(#(input.iter().join(", "))) }`, attempting to manually join the parsed
  expressions into the argument list. This is wrong on two levels — `#(...)` in `quote!` is
  *repetition* syntax and requires a trailing `*` (optionally with a separator before it, e.g.
  `,*`) to be recognized as such at all; without one, `quote!` doesn't error, it just falls back
  to splicing the literal, unevaluated tokens inside the parens (including the literal punctuation
  characters in `.join(", ")`) verbatim into the output, with a stray `#` token ahead of them.
  Confirmed via `cargo expand`, which rendered the result as `let p = (/*ERROR*/);`, and via the
  underlying rustc error (`` expected one of `!` or `[`, found `(` ``) — the exact signature of a bare
  `#` token ending up in expanded code (valid Rust only allows a standalone `#` to start
  `#[...]`/`#![...]` attributes). The fix needs neither `#(...)*` repetition nor `.join()` at all:
  `Punctuated<Expr, Comma>` already implements `ToTokens` with the punctuation correctly placed
  between elements (that's the type's whole purpose), so plain `#input` interpolates `a, b, c`
  directly — `quote! { #function_name(#input) }`.
  **Testing the out-of-range arity error without a new dependency:** the obvious approach for
  testing a macro's compile-error path is `trybuild` (compile a fixture file, assert it fails),
  but that's a new dev-dependency for one test. Used a lighter, dependency-free pattern instead:
  each module's real logic (`seq_impl`/`choice_impl`, see `choice!` below) takes and returns
  `proc_macro2::TokenStream`, not `proc_macro::TokenStream` — the `proc_macro::TokenStream`
  boundary conversion (`.into()` each way) happens only in `lib.rs`'s `#[proc_macro] pub fn
  seq`/`choice` entry points, which just call straight through
  (`seq_impl(input.into()).into()`). `proc_macro2::TokenStream` (unlike `proc_macro::TokenStream`)
  can be constructed and inspected in an ordinary `#[test]`, outside any actual macro-expansion
  context — `proc_macro2` exists specifically to make `syn`/`quote`-based code testable this way,
  which is also why `syn` and `quote` are built on it rather than `proc_macro` directly. This let
  the arity-boundary tests (`rust-petitparser-macros/src/seq.rs`'s `#[cfg(test)] mod tests` —
  `rejects_too_few_parsers`, `rejects_too_many_parsers`, plus `accepts_two_parsers`/
  `accepts_nine_parsers` checking the generated tokens' `.to_string()` directly) live as plain
  unit tests in the macro crate itself, with the happy-path arities (2 through 9) and failure
  propagation covered by ordinary integration tests using real `seq!(...)` invocations
  (`tests/seq_macro_tests.rs`).
  **Trailing-closure fusion (implemented):** `seq!(a, b, c, |x, y, z| ...)` now fuses into
  `seq3(a, b, c).map3(|x, y, z| ...)` in one expansion, rather than requiring a separate
  `.mapN(...)` call afterward — the originally-discussed design, now built. Implementation:
  collect the parsed `Punctuated<Expr, Comma>` into a `Vec<Expr>`, then `.pop()` the last element
  and pattern-match it against `Expr::Closure(c)` — if it matches, `c` (a `syn::ExprClosure`) is
  the fused closure and the remaining `Vec` is the parser list; if not, push the popped value
  back onto the `Vec` unchanged and treat the whole thing as a plain (non-fused) call, same as
  before. The arity used for `seqN`/`mapN` is computed from the parser-only count (closure already
  removed), so a 3-parser-plus-closure call still picks `seq3`/`map3`, not `seq4`/`map4`. The
  empty-vs-non-empty optional suffix (`.mapN(closure)` or nothing) is built as its own
  `TokenStream2` fragment first (`quote! { .#map_name(#closure) }` or plain `quote! {}` for the
  empty case) and spliced in via `#map` — `quote!{}` is the literal "empty token stream" value,
  letting the conditional logic live outside the final `quote!` block rather than needing
  `quote!`'s repetition syntax to express an if/else. Also switched the parser-list interpolation
  from the earlier `#input` trick (which only worked because `input` was still a `Punctuated`,
  with its own comma-joining `ToTokens` impl) to `#(#exprs),*` real repetition syntax, since
  popping the closure off requires converting to a plain `Vec<Expr>` first, which has no such
  impl. **Deliberately not implemented:** validating that the closure's parameter count
  (`ExprClosure.inputs.len()`) matches the parser count before emitting — a mismatch is just left
  to surface as an ordinary Rust type error from the generated `.mapN(closure)` call (confirmed:
  `seq!(char('a'), char('b'), char('c'), |a, b| ...)` — 3 parsers, 2-param closure — produces a
  clean `E0593: closure is expected to take 3 arguments, but it takes 2 arguments` pointing
  straight at the closure). Writing a custom check here would just duplicate validation the type
  system already does correctly; the macro's own arity check (parser count in `2..=9`) is the
  only validation that's actually the macro's own responsibility, since nothing downstream of the
  macro checks *that*. Tested with the same split as everything else here: unit tests in
  `seq.rs` (`fuses_trailing_closure_into_map_call`, `fuses_trailing_closure_at_min_arity` (2),
  `fuses_trailing_closure_at_max_arity` (9), `rejects_too_few_parsers_with_trailing_closure` —
  confirming the closure itself doesn't count toward the parser-arity floor) and integration
  tests using real `seq!(...)` invocations in `tests/seq_macro_tests.rs`
  (`seq_macro_fuses_trailing_closure`, `seq_macro_fuses_trailing_closure_at_min_arity`,
  `seq_macro_fused_closure_failure_still_propagates`).
  **Swept existing `seqN(...).mapN(closure)` chains to the fused form** where one already existed
  — `src/parser/ext.rs` (`trim_with`), `src/expression/group.rs` (`wrapper`, which was still on
  plain `.map(move |(l, t, r)| ...)` with tuple destructuring — looks like it was missed by the
  earlier `mapN` sweep, picked up here while fusing it directly), `tests/example-grammars/
  tabular.rs`, `tests/example-grammars/uri.rs` (`credentials`), `tests/combinator_tests.rs`
  (×3), `tests/misc_tests.rs` (×4, the `position()` tests). Deliberately did **not** sweep bare
  `seqN(...)` calls with no `.mapN(...)` attached — `seq!` only pays for itself when it's
  collapsing two calls into one; converting a standalone `seq3(a, b, c)` to `seq!(a, b, c)` with
  nothing to fuse is a pure stylistic wash (same length, loses the "this is obviously a plain
  function call" property) with no offsetting benefit. Also deliberately left `choiceN(...)`
  call sites alone everywhere — `choice!` has no fusion counterpart (see "What's Next"), so there
  was no genuine candidate to convert, anywhere in the codebase. **Excluded at the time from the
  sweep: all four `#[grammar]`-based example grammars** (`pascal.rs`/`bibtex.rs`/`json.rs`/
  `math.rs`) — `seq!`/`choice!` didn't work inside a `#[grammar]` module at all yet (see the
  "Fixed: `seq!`/`choice!` now work inside `#[grammar]` modules" entry above for the eventual fix;
  `json.rs`'s `member()` rule was converted first as the real-world validation of that fix, and
  the rest of the sweep across all four grammars (plus `smalltalk/mod.rs`) is now done too — see
  the `=>` migration entry above).
- **`choice!` variadic macro** (`rust-petitparser-macros/src/choice.rs`, function-like
  `#[proc_macro]`, re-exported via `prelude.rs` alongside `seq`/`grammar`) — same shape as `seq!`
  above: `choice!(a, b, c)` expands to `choice3(a, b, c)`, via the identical
  parse-as-`Punctuated<Expr, Token![,]>` → count → `format_ident!("choice{}", n)` → `quote!`
  pipeline, the same `2..=9` arity validation with `syn::Error::to_compile_error()`, and the same
  `proc_macro2`-testable-inner-function split for testing that error path without `trybuild`.
  Two real bugs found in the first draft, both caught by `cargo build -p rust-petitparser-macros`
  before any test was even written: (1) `return Error::new_spanned(...).to_compile_error(),` —
  trailing comma instead of semicolon, a plain syntax error (`return EXPR,` isn't valid Rust;
  `return` statements end in `;`), confirmed directly from the rustc diagnostic
  (`` expected one of `.`, `;`, `?`, `}`, or an operator, found `,` ``). (2) the call-building
  `quote!` block was written as `#function_name(input)` — missing the `#` sigil in front of
  `input` entirely (not the repetition-syntax confusion that hit `seq!`'s first draft, just a
  plain dropped sigil). Without `#`, `quote!` doesn't interpolate the variable at all; it emits
  the literal identifier `input` into the generated code, so `choice!(a, b, c)` would have
  expanded to `choice3(input)` — referencing a variable named `input` that doesn't exist at the
  call site, not the three parser expressions the caller actually wrote. Fixed to `#input`,
  mirroring `seq!`'s identical fix. Tested in `rust-petitparser-macros/src/choice.rs`'s
  `#[cfg(test)] mod tests` (same four cases as `seq!`) and `tests/choice_macro_tests.rs`
  (arities 2 through 9 plus failure-propagation, using real `choice!(...)` invocations).
  Type-unification gotcha worth flagging to users: Rust requires every alternative passed to
  `choice!`/`choiceN` to produce the same `T`, unlike dart's dynamically-typed `toChoiceParser()`.
  We hit this twice during the example-grammar ports, pre-dating `choice!` itself —
  `bibtex.rs`'s `field_string_within_braces` (three alternatives erased to `Parser<()>` via
  `.constant(())` since only the consumed span mattered, recovered later via
  `.input_with_message(...)`) and `pascal.rs`'s whole grammar (typed `Parser<()>` everywhere from
  the start, since it's a pure recognizer with no AST, so the 11-way `statement()` choice never
  even hit a type mismatch). Neither case needed actual heterogeneous *values* preserved; if a
  future caller does, the fix is a small sum-type enum with each arm `.map()`-ed into the
  matching variant, not anything `choice!`/`choiceN` themselves need to solve.
- **Fixed: `seq!`/`choice!` now work inside `#[grammar]` modules.** Root cause (see prior "What's
  Next" entry, now resolved): `ParserCallRewriter`'s `visit_expr_mut` override only matched
  `Expr::Call` nodes; a `seq!(...)`/`choice!(...)` invocation is an opaque `Expr::Macro`/
  `Stmt::Macro` at that point in expansion (macro arguments are raw, unparsed
  `proc_macro2::TokenStream`, not `syn::Expr` nodes), so `VisitMut`'s default traversal had
  nothing typed to recurse into and the rule-call rewrite silently never happened. The actual fix
  turned out to be more general than "handle `Expr::Macro`": `syn`'s generated `VisitMut` trait
  ships a dedicated, deliberately-empty-by-default hook —
  `fn visit_token_stream_mut(&mut self, i: &mut proc_macro2::TokenStream) {}` — that
  `visit_macro_mut`'s default implementation calls unconditionally, and *every* macro-position
  variant's default (`visit_expr_macro_mut`, `visit_stmt_macro_mut`, `visit_item_macro_mut`,
  `visit_pat_macro_mut`, etc. — expression position, statement position, item position, pattern
  position, ...) routes through `visit_macro_mut` to get there. Overriding this **one** method
  handles every macro invocation anywhere in a rule body uniformly, rather than needing a
  separate override per AST position. Implementation
  (`ParserCallRewriter::visit_token_stream_mut`, `rust-petitparser-macros/src/grammar.rs`):
  parse the incoming tokens as `Punctuated<Expr, Token![,]>` (mirroring `seq!`/`choice!`'s own
  input-parsing shape); on success, recursively call `self.visit_expr_mut(&mut each)` on every
  parsed expression (re-entering the existing rewrite logic, so a rule call nested inside a
  trailing closure's body, or inside a macro nested inside another macro's arguments, still gets
  rewritten — confirmed by a dedicated test); on failure (the macro's tokens aren't a
  comma-separated expression list — `println!`'s format-string-plus-args, `matches!`, etc.),
  leave the tokens untouched rather than erroring, so unrelated macros are unaffected. A
  deliberately generic design, not scoped to literally `seq`/`choice` by name, since the same
  "rule calls go opaque inside macro tokens" problem applies to *any* macro a rule body might
  invoke (the original bug-hunt used `println!` as the minimal repro, precisely to confirm it
  wasn't `seq!`/`choice!`-specific).
  Required first converting `grammar_impl` from `proc_macro::TokenStream` to
  `proc_macro2::TokenStream` (mirroring `seq_impl`/`choice_impl`'s existing pattern, with
  `lib.rs`'s `pub fn grammar` entry point doing the `.into()` conversion both ways) — this is
  what makes `grammar_impl` callable from an ordinary `#[test]` (and steppable in a real
  debugger), rather than needing a full `cargo expand` cycle per iteration. Tested in
  `rust-petitparser-macros/src/grammar.rs`'s `#[cfg(test)] mod tests`: a `println!`-wrapped rule
  call (proving genericity), `seq!(...)` as a `.mapN(...)`-chain receiver (genuine `Expr::Macro`
  position, not just statement position), `seq!(...)` with trailing-closure fusion *and* a rule
  call nested inside the closure body (proving the recursion goes all the way down), and
  `choice!(...)`. Validated end-to-end against real macro expansion by converting `json.rs`'s
  `member()` rule from `seq3(...).map3(...)` to `seq!(...)` with fused trailing closure — compiles
  and the JSON test suite still passes unchanged.
- **`seq!` migrated from implicit trailing-closure detection to an explicit `=>` separator** —
  `seq!(parsers... => map_target)`, inspired by `quote_spanned!`'s `span => tokens` convention.
  `=>` **replaces** the old trailing-comma-closure sugar entirely (one mechanism, no silent-
  misdetection risk). Motivation: the old fusion check (`Some(Expr::Closure(c)) => fuse`) was
  purely syntactic and couldn't tell "named function used as the map callback" apart from "a
  parser value referenced by a bare variable" — both are just `Expr::Path` at the syntax level,
  and proc-macros only see syntax, never types. This bit for real: `tests/example-grammars/
  smalltalk/mod.rs`'s `unary_expression()`/`binary_expression()`/`cascade_expression()` originally
  passed `build_message`/`build_cascade` (plain functions, not closure literals) as the trailing
  `seq!` argument, which silently fell through to "just another parser" instead of fusing into
  `.mapN(...)` — surfaced as a `Seq3<...>: Parser<...>` not satisfied error, fixed at the time by
  eta-expanding into literal closures. The `=>` design fixes the underlying ambiguity properly:
  once the separator itself signals "what follows is the map target," there's no need to even
  check whether it's `Expr::Closure` — anything after `=>` goes straight into `.mapN(...)`, since
  `.mapN` already accepts any `Fn`-shaped expression (so the eta-expanded closures could be
  unwound back to plain `build_message`/`build_cascade` again, though that wasn't done since it's
  a cosmetic wash either way).
  - **Shared parse shape, not duplicated**: `rust-petitparser-macros/src/seq_input.rs`'s
    `SeqInput` (`parsers: Punctuated<Expr, Token![,]>`, `target: Option<Expr>`, plus `Parse`/
    `ToTokens` impls) is used by both `seq_impl` (build `seqN(...).mapN(target)`) and
    `#[grammar]`'s `ParserCallRewriter`. The rewriter needed updating too: its
    `visit_token_stream_mut` only tried a plain `Punctuated<Expr, Token![,]>` parse, which now
    *fails* outright on any `=>`-form `seq!` call (the arrow isn't a valid continuation token for
    a comma list) — silently skipping rule-call rewriting inside it. Fixed by falling back to
    `SeqInput` when the plain comma-list parse fails, recursing into rule calls on both the
    parser list and the target.
  - **Real bug caught by testing, not anticipated up front**: `SeqInput::parse` didn't tolerate a
    trailing comma *after* the target (`=> |x, y| { ... },`, common for diff-friendly multi-line
    closures) — left unconsumed input behind for the top-level `parse2` call to reject. Two real
    call sites (`sequence()`, `method()` in `smalltalk/mod.rs`) had exactly this shape. Fixed by
    having `SeqInput::parse` optionally consume one trailing comma after the target.
  - **Migration script** (one-off, not checked into the repo — lived in the scratch directory):
    a `syn`-based tool that finds every old-style fused `seq!(...)` call and swaps just the single
    comma token immediately before the closure for `=>`, leaving every other character untouched.
    Deliberately *not* a full `syn::parse_file` → `prettyplease::unparse` round trip — `syn` drops
    ordinary (non-doc) comments entirely when parsing, since they're stripped before tokenization
    and never enter the AST at all, and this codebase's files are heavily commented. A whole-file
    reparse-and-reprint would have silently deleted every `//` comment. The single-token swap
    avoids this since it never reconstructs anything; it slices the original source text once,
    using `proc-macro2`'s `span-locations` feature to map each token's `LineColumn` back to a byte
    offset. Converted every old-style call across the repo: `src/expression/group.rs`,
    `src/parser/ext.rs`, `smalltalk/mod.rs` (35 sites), `tabular.rs`, `misc_tests.rs`, `uri.rs`,
    `seq_macro_tests.rs`, `combinator_tests.rs`, `bibtex.rs`, `json.rs`.
  - Also fixed along the way, while improving `#[grammar]`'s own diagnostics: `grammar_impl`'s
    `set_calls`/`undefined_decls`/`field_decls` switched from `quote!` to `quote_spanned!(f.sig.
    ident.span() => ...)`, so a type error in a rule's body now points at that rule's own location
    in the source instead of at the `#[grammar]` attribute line (`quote!`'s literal scaffolding
    tokens default to `Span::call_site()`, which during attribute-macro expansion resolves to the
    attribute's own line — even though the interpolated rule-body statements inside already
    carried correct original spans). This had a real side effect: rustc suppresses several lints
    (`dead_code`, `unused_braces` among them) for code it recognizes as macro-generated, and that
    recognition is span-based — attaching a "real" source span un-suppresses them. Two lints came
    back as a result, both fixed: `dead_code` on the generated struct's fields (genuinely true —
    private, non-`start` rule fields are written once via `Self { field_name, ... }` but never
    read back through `self.field`, since inter-rule references resolve through `new()`'s local
    variables instead; fixed via `#[allow(dead_code)]` on the generated struct), and
    `unused_braces` on `set_calls`'s `{ #(#stmts)* }` block (redundant whenever a rule body is a
    single expression, but the braces are still structurally required for multi-statement bodies;
    fixed via `#[allow(unused_braces)]` on the generated `fn new()`).
  - Full workspace (build/test/clippy/fmt) green as of this writing.
- **`Context` gained a `text: Rc<String>` field, threaded everywhere alongside `buffer:
  Rc<[char]>`** (`src/core/context.rs`) — groundwork for porting dart's regex-backed
  `PatternParser` next (see "What's Next"): the `regex` crate operates on UTF-8 byte offsets
  over a `&str`/`String`, and rebuilding that from `buffer: Rc<[char]>` on every match attempt
  would be an O(n) tax on the hot path, so `text` is built once and cloned (cheap `Rc` bump)
  everywhere a `Context` already gets cloned. Added `Context::new(input: &str, position: usize)
  -> Self` as the one place that actually builds both fields from scratch — used pervasively by
  tests as a replacement for ad hoc buffer-building helpers.
  - **`fast_parse_on`'s signature changed from `(buffer: Rc<[char]>, position: usize) ->
    Option<usize>` to `(context: &Context) -> Option<usize>`** — once `Context` had a second
    field, the old loose-parameter signature would have needed a third parameter bolted on (and
    would again for any future field); taking `&Context` directly means new fields are available
    to every override for free. Touched all ~21 files that override it, plus the `Rc<P>` blanket
    impl (`core/parser.rs`) and `#[grammar]`'s generated `impl Parser<T>` block
    (`rust-petitparser-macros/src/grammar.rs`) — the latter was the one place a stale hand-written
    `quote!` template could have silently kept the old signature, but it had already been kept in
    sync. Confirmed by the user mid-refactor to be a real ergonomics win (not just mechanical
    churn) — see [[feedback_context_unification]] in project memory.
  - **`Token<T>` switched from `{ value, buffer: Rc<[char]>, start: usize, end: usize }` to
    `{ value, context: Context, end: usize }`** (`src/core/token.rs`) — `context.position`
    subsumes the old `start` field (a new `Token::start()` accessor reads it back),
    `context.buffer`/`context.text` ride along for free instead of `Token` needing its own
    separate `text` field. `end: usize` stays a sibling field since `Context` only models one
    position, not a range. `line_and_column_of`/`position_string` changed from taking loose
    `(buffer: Rc<[char]>, position: usize)` to a single `&Context`, and `line_and_column_of` now
    returns a `TextLocation { line: usize, column: usize }` struct (exported via `prelude.rs`)
    instead of a `(usize, usize)` tuple — `line_and_column_of`'s internal newline-scan now reuses
    the context's existing `buffer`/`text` handles via `context.with_position(0)` (a new
    `HasContext` helper) rather than rebuilding anything.
  - **`all_matches` now takes a full `Context` directly** (not `buffer: Rc<[char]>, start:
    usize`) — strengthens its pre-existing design rationale (its caller, `line_and_column_of`,
    already holds a `Context`-shaped pair and should hand it over rather than rebuild it) rather
    than changing it. By contrast, `accept`/`accept_at` deliberately **kept** taking `&str` (plus
    `start: usize` for `accept_at`) and build a fresh `Context` internally — they're public,
    one-shot leaf entry points (mirroring `parse(&str)` and dart's own API), so exposing the
    internal `Context` type on their public signature would have been a regression, not a
    parallel improvement.
  - One real test bug found while fixing up call sites for the new signatures:
    `tests/action_tests.rs`'s `token_test` built its expected `Token` from `success.context` (the
    *end* context, position 3) instead of the starting context — `TokenParser`'s actual
    implementation correctly uses the start context for `Token.context` with `end` held
    separately, so the test's own expected value was wrong, not the implementation.
  - `regex = "1.12.4"` was added to `Cargo.toml` in anticipation of the `PatternParser` port (see
    below) and is now actually used by `src/parser/regex.rs`.
- **Ported dart's regex-backed `PatternParser`** (`lib/src/parser/predicate/pattern.dart`) as
  `src/parser/regex.rs` — scoped to `regex::Regex` specifically rather than dart's fully generic
  `Pattern` (String or RegExp); a literal-string pattern is better served by this codebase's
  existing `string()`. `regex(pattern: &str) -> RegexParser` is the bare constructor (named
  `regex`, not `pattern`, since `pattern()` was already taken by the char-class primitive);
  `RegexParser { regex: Regex, message: String }`'s fields are public, so a pre-compiled
  `regex::Regex` can also be passed directly (mirrors dart's "pass a compiled pattern instance"
  constructor, avoiding per-parse recompilation). Result type `RegexMatch { text: String, start:
  usize, end: usize, groups: Vec<Option<String>> }` mirrors dart's `Match` (`isPatternMatch` in
  dart's test matchers: `.group(0)`/`.start`/`.end`/`.group(1+i)` for captures) — `start`/`end`
  are **char-indexed** (not byte-indexed), matching this codebase's existing convention. `groups`
  is `Vec<Option<String>>`, not `Vec<String>`, because a capture group can either not participate
  in a match at all (`None`, e.g. `(x)?` against input with no `x`) or participate but match the
  empty string (`Some(String::new())`, e.g. `(x*)`) — these are observably different and dart's
  nullable `match.group(i)` makes the same distinction; both cases get dedicated tests
  (`regex_non_participating_optional_group_is_none`/`regex_optional_group_matching_empty_string_is_some_empty`
  in `tests/character_tests.rs`). The `ignoreCase`/`unicode` flags dart's `RegExp` constructor
  exposes aren't handled specially by `RegexParser` itself — same as dart, where they live on the
  `RegExp`/`Pattern` object passed in, not on `PatternParser` — and Rust's `char` being a full
  Unicode scalar value already sidesteps the UTF-16-surrogate-pair concern dart's `unicode:` flag
  exists for.
  - **Anchoring**: `regex`'s `find_at`/`captures_at` find the next match *at or after* a byte
    offset, not *exactly at* it (unlike dart's `Pattern.matchAsPrefix`, which is truly anchored).
    Both `parse_on` and `fast_parse_on` check `match.start() == byte_pos` after the call; a match
    starting later is a parse failure even though `find_at` "found something."
  - **Byte/char offset translation, and a real bug caught only by a dedicated Unicode test**: the
    `regex` crate operates on UTF-8 byte offsets; this codebase indexes by char throughout. A
    `byte_pos(context)` helper (`context.text.char_indices().nth(context.position).map(|(b,
    _)| b).unwrap_or(context.text.len())`) converts the incoming char position to the byte offset
    `find_at`/`captures_at` need — the `.unwrap_or(text.len())` matters because `char_indices()`
    has no entry for "one past the last char," which is otherwise a legitimate position to search
    from (e.g. a pattern that can match empty). The first draft used this same `byte_pos` value
    directly as `RegexMatch.start`/`.end` and as the resulting `Context`'s new position — silently
    correct for pure-ASCII input (where byte and char offsets coincide) but wrong for anything
    else: parsing `"日12"` (3 chars) starting at char position 1 produced `RegexMatch { start: 3,
    end: 5 }` and a resulting position of 5 — past the end of a 3-char buffer, which would panic
    on the next parse step. None of dart's own `PatternParser` tests catch this class of bug at
    all (dart strings have no byte/char distinction to get wrong), so it needed a dedicated,
    port-specific regression test with a non-ASCII prefix
    (`regex_start_and_end_are_char_indexed_not_byte_indexed`/
    `regex_fast_parse_on_position_is_char_indexed_not_byte_indexed` in `tests/character_tests.rs`)
    to catch at all. Fixed by keeping `byte_pos` scoped to indexing into `context.text`/calling
    `find_at`/`captures_at`, while `RegexMatch.start`/the new `Context` position are computed from
    `context.position` (the original char position) and the matched text's `.chars().count()`,
    never from `byte_pos` directly. Only the overall match's start/end positions need this
    translation — capture-group *text* itself (`m.as_str().to_string()`) needs none.
  - **`groups` excludes the full match (group 0)**, via `captures.iter().skip(1)` — the `regex`
    crate's `Captures::iter()` includes group 0 first, but dart's `groups` (per `isPatternMatch`'s
    `List.generate(match.groupCount, (g) => match.group(1 + g))`) deliberately excludes it, and
    `RegexMatch.text` already covers the same information. An early draft kept group 0 in and
    compensated for the mismatch in the test helper instead (prepending the expected full-match
    text into the expected `groups` vector) — passed against the draft implementation, but wasn't
    actually dart parity. Fixed by skipping group 0 in the real implementation and simplifying the
    test helper back down once the real fix landed.
  - **`fast_parse_on` uses `find_at`, not `captures_at`** — `parse_on` needs `captures_at` to
    extract capture-group text, but `fast_parse_on` only needs the overall match's bounds, so the
    cheaper `find_at` (no capture-group span bookkeeping) avoids paying for work the fast path
    doesn't use, the same "fast path skips slow-path-only work" principle already established for
    `Map`/`Constant`/`Token` (see the `fast_parse_on` side-effect-gap fix elsewhere in this doc).
  - Tested in `tests/character_tests.rs`'s `regex` section: `regex_test`/`regex_groups` port
    dart's `parser_predicate_test.dart` 'regexp'/'regexp groups' groups verbatim (dart's 'string'
    sub-test, a literal-string `Pattern`, isn't ported — out of scope per the `string()` note
    above); plus the `None`-vs-`Some("")` group tests, the two Unicode position-correctness
    regression tests, and `regex_bare_constructor_builds_default_message` (the `regex(&str)`
    constructor itself wasn't exercised by the dart-ported tests, which all construct
    `RegexParser` directly with a custom message).

## What's Next

**→ The linter port is COMPLETE**, and so is the reflection layer it sits on. Done as of this
writing: all 12 implemented rules + the equality core; deferred refactor 1 (borrow `ParserKind`);
deferred refactor 3 (collapse the `is_*` flags into `kind()`); the `allChildren`/`findPath`
reflection *tests* (`tests/reflection_tests.rs`); and the `Other(Rc<dyn CustomParserKind>)`
custom-parser-equality extension point (see below). Deferred refactor 2 (`Rc<str>` messages) is
resolved as **won't-do** (see its note — the motivation is already covered by whole-parser `Rc`;
measure before revisiting). What actually remains, by size:
- **Small / well-scoped:** the Rust-specific `s.set(s.clone())`-vs-`.borrow()` leak lint; cycle-safe
  recursive `Debug` for `SettableParser`/`SettableParserRef`; the parked `FnWithText`/`func!` Debug
  improvement.
- **Big lever:** `transformParser` + `copy()` (graph rewrite/copy), which unlocks the debug tools
  (`trace`/`progress`/`profile`) *and* `optimize`/`replace`/`resolve` + `expectParserInvariants`.
- **Separate initiatives:** the `dart-petitparser-examples` grammars (lisp/prolog interpreters,
  regexp engine); a benchmark-driven look at failure-path perf (lazy messages, not `Rc<str>`).

See the dedicated bullets below for each. Roughly equal priority within a size band.

- **Debug tools (`trace`/`progress`/`profile`) — not ported, gated on `transformParser`+`copy`.**
  Dart's `lib/debug.dart` exposes three parser-instrumentation tools (`lib/src/debug/{trace,
  progress,profile}.dart`):
  - `trace(parser)` — prints an indented enter/exit tree of every parser activation and its
    Success/Failure result.
  - `progress(parser)` — prints a `*`-per-input-position bar as it parses; backward jumps
    visualize backtracking.
  - `profile(parser)` — per-parser activation count + total microseconds, the "where is my grammar
    spending time" view.
  **Genuine motivation, not just parity:** `profile`/`progress` are exactly the tools for the
  backtracking-cost concerns this codebase keeps flagging (the `Rc`-storage tradeoff, the
  no-packrat/exponential-backtracking note). So these are worth building *for our own debugging*,
  not only for completeness.
  **Why they're gated:** all three are literally
  `transformParser(root, (parser) => parser.callCC(...logging...))` — they rewrap *every* node in
  the graph with a `callCC` interceptor. That needs `transformParser` (`reflection/transform.dart`),
  which rebuilds the parser graph, which in turn needs a `Parser.copy()`-style
  reconstruct-with-new-children on every parser type. We have the `callCC`/`ContinuationParser`
  primitive already (`src/parser/ext.rs`), but **not** `transformParser` and **not** `copy()` — the
  latter deliberately skipped (see "Key Design Decisions": our `SettableParser`/`.borrow()` trick
  handles *recursion* without graph-copying, but graph *rewriting* is a separate capability we never
  built).
  **So the real task is the prerequisite**, and it's the harder half: port `transformParser` +
  a `copy_with_children(new_children)` method across every parser type. That's the same
  downcast-and-rebuild shape as the `kind()` sweep but harder — it must *reconstruct* a typed
  parser from a `dyn` node (new instance with swapped children), not just read properties. The
  cyclic-grammar case needs care too (dart's `transformParser` has dedicated loop handling — see
  its `transform_test.dart` 'loop (existing)'/'loop (new)' cases, which we'd also want to port).
  Once `transformParser`+`copy` exist, the three debug tools fall out almost directly from dart
  (each is ~40 lines wrapping `callCC`). Recommended sequencing: treat `transformParser`/`copy` as
  its own initiative; the debug tools (and `optimize`/`replace`/`resolve`, which share the same
  dependency) are the payoff that follows.

- **Porting `dart-petitparser-examples`** (`/home/toddobryan/code/dart/dart-petitparser-examples`)
  into this repo, as a separate initiative from the test-parity porting below. json and math/expr
  are already covered by `tests/example-grammars/{json,expr}.rs`. Done so far: **tabular**
  (CSV/TSV) — fully portable with zero new core features since it has no recursive rules (no
  `#[grammar]`/`SettableParser` needed, just plain methods on a `TabularDefinition` struct holding
  `Rc<dyn Parser<String>>` config fields). This also surfaced that `newline()` existed in
  `src/parser/misc/newline.rs` but was never re-exported from `prelude.rs` — fixed.
  `pattern()`/`pattern_ci()` are now implemented (see What's Implemented), which unblocked **bibtex**
  (`tests/example-grammars/bibtex.rs`, done — uses `#[grammar]` since `field_string_within_braces`
  recurses for nested `{...}` groups; field name "type" → `kind` since `type` is a Rust keyword;
  `field_string_within_{quotes,braces}` choice arms are typed `Parser<()>` since the surrounding
  `.input_with_message(...)` only cares about consumed span, not value — sidesteps dart's looser
  typing there, which mixes `String` and tuple values in the same `toChoiceParser()` list).
  Dart's bibtex test suite also has a live-network `scg.bib` group (fetches a real ~9600-entry
  file from GitHub, checks size + round-trips every entry) — ported as
  `bibtex::scg_bib_size_and_round_trip`. Initially `#[ignore]`d, then deliberately un-ignored
  (runs under plain `cargo test`, ~3s) — fast enough that the coverage was judged worth always
  running rather than opting in. Needed adding `ureq` as a dev-dependency (blocking HTTP client,
  matches this project having no async anywhere). `#[ignore]` composes fine with `#[gtest]` if
  this needs to be revisited (e.g. if CI loses network access) — the macro re-emits all of a
  function's existing attributes onto the generated test item, so just stack `#[ignore]`
  above/below `#[gtest]` as normal.
  Next candidates, roughly by size/feature-gap:
  - **uri** done (`tests/example-grammars/uri.rs`) — no recursion, plain top-level functions
    (`uri()`/`authority()`/`query()`), no struct/config needed (unlike `tabular`'s csv/tsv
    variants). Dart's `Map<Symbol, dynamic>` result became concrete `UriParts`/`Authority` structs.
    Dart re-parses the matched authority/query substrings independently (`lib_authority.authority
    .parse(...)`) rather than embedding those sub-grammars in the main parse tree — mirrored that
    directly (`parse_authority`/`parse_query` helpers called from `uri()`'s `.map()`), so no
    recursion or cross-rule sharing was needed there either. Ported all 8 field-checking dart tests
    plus the full 36-URL "url-regex" smoke-test list as one parameterized test.
  - **pascal** done (`tests/example-grammars/pascal.rs`, ~50 rules via `#[grammar]`; `block`,
    `factor`, and `comment` are genuinely recursive). Key adaptations from dart's
    `PascalGrammarDefinition`:
    - Dart's rule named `type` had to become `pascal_type` (Rust keyword).
    - Dart's parameterized `ref1(token, source)` helper (which can't be a macro-managed zero-arg
      rule since it takes an argument) became two free functions *outside* the `#[grammar]`
      module: `token_parser(p, spacer)` and `token_str(literal, spacer)`, both taking the
      `spacer()` rule's `SettableParserRef<()>` as an explicit `P: Parser<S> + Clone` parameter,
      called from rule bodies as `token_str("program", spacer())` — nested zero-arg rule-calls
      inside a non-rule function's argument list are still correctly rewritten by the macro's
      `VisitMut` (it falls through to default recursive traversal for non-matching call exprs).
    - `token_str` returns `Rc<dyn Parser<String>>`, not `impl Parser<String>` — its two branches
      (keyword vs plain literal) are different concrete types, and `impl Trait` return position
      requires one concrete type across all paths.
    - Dart's `_keywords` set (populated as a side effect of constructing `token()` calls) became a
      static `const KEYWORDS: &[&str]` of the 36 keywords actually used in the grammar (grepped
      from `grammar.dart`), checked by a plain `is_keyword()` — no need to replicate the stateful
      collection trick.
    - **Gotcha (the one real bug during porting):** `spacer()` itself is `.plus()` (one-or-more
      whitespace/comment units, matching dart exactly), but dart's `TrimmingParser.trim(spacer)`
      loops it internally so the *trim* is effectively zero-or-more even though a single
      application of `spacer()` requires ≥1. Our `trim_with` has no such internal loop, so
      `token_parser` must pass `spacer.star()` (not bare `spacer`) as both delimiters — passing
      the bare rule made every token require leading/trailing whitespace, breaking anything
      without it (e.g. `program foo;` with no space before `foo`... actually *any* token at the
      very start of input, since there's nothing to trim there at all).
    - `statement()`'s relational-operator choice (`<, <=, =, <>, >=, >, in`) is ported in dart's
      exact (PEG-ordered, `<` before `<=`) order — for input `<=` this would wrongly match just
      `<` and backtrack out the whole optional relational clause. This is a latent bug inherited
      from upstream; dart's own test suite never exercises `<=`/`<>`/`>=` standalone either, so
      test parity doesn't require fixing it. Left as-is rather than silently "improving" upstream
      behavior.
    - The `comparestrings` full-program test uses a cleaned-up (properly newline-separated)
      version of dart's fixture rather than a byte-for-byte port — dart's original relies on
      adjacent string-literal concatenation (two `join('\n')` list entries with no comma between
      them) that has no Rust equivalent and wasn't meaningful test content on its own.
  - **`ExpressionBuilder`/`ExpressionGroup` core feature: done** (`src/expression/{builder,group}.rs`,
    exported via prelude). dart-petitparser's `ExpressionBuilder` (`primitive`/`group`/`wrapper`/
    `prefix`/`postfix`/`left`/`right`/`optional` precedence-climbing API) — a real new abstraction,
    not a one-off translation. Tested via `tests/expression_tests.rs` (49 tests, ported from dart's
    own core-feature test `test/expression_test.dart` — *not* `dart-petitparser-examples`' simpler
    `math_test.dart` — since dart's own test has zero dedicated `ExpressionGroup` unit tests and
    instead exercises every feature, including two wrappers on one group, postfix, and the
    `optional()` assert-failure case, entirely through the public `ExpressionBuilder` API).
    `dart-petitparser-examples`' actual **math.dart example: done**
    (`tests/example-grammars/math.rs`) — confirmed small once the builder/group machinery
    existed: AST (`Expr::{Value,Variable,Application}`), `common.dart`'s constant/function
    tables, and the `ExpressionBuilder<Expr>` wiring, plus all 11 portable `math_test.dart`
    groups (skipped `linter`, no reflection-based linter equivalent). Notable adaptations:
    - `Application`'s third field is a plain `fn(&[f64]) -> f64` rather than dart's
      `Function`/`Function.apply(function, args)` dynamic dispatch — Rust has no runtime-variadic
      apply, and `create_binding`'s switch on `arguments.length` only ever produces 1- or 2-arg
      applications anyway, so a slice-taking function pointer covers every case dart's `functions1`/
      `functions2` maps do, without needing separate 1-arg/2-arg closure types.
    - Added an actual `Display for Expr` (`Value{v}`/`Variable{name}`/`Application{name}`,
      mirroring dart's three `toString()` overrides) instead of leaving `Application`'s `name`
      field unused — dart's test calls `ast.toString()` after every `verify()` (a trivial
      "doesn't throw" check, since dart's `toString()` can't return null); ported as
      `result.value.to_string().is_empty()` being false inside the shared `verify_with` helper,
      which both exercises `Display` on every case and gives the otherwise clippy-flagged-dead
      `name` field a real reason to exist.
    - Dart's `verify(input, result, {variables = const {}})` named/defaulted parameter became two
      functions instead of one — `verify(input, expected)` (the common no-variables case) and
      `verify_with(input, expected, &HashMap<String, f64>)` — since Rust has no default arguments.
    - Dart's `expect(() => verify('x', double.nan, variables: {}), throwsArgumentError)` (evaluating
      an unbound variable throws) is its own `#[should_panic(expected = "Unknown variable: x")]`
      test (`variable_unknown_panics`) rather than folded into `variable()`'s other (non-panicking)
      assertions — `#[should_panic]` applies to the whole test function, so a mixed-outcome test
      can't be expressed in one `#[gtest]` fn the way dart's single `test('variable', ...)` block
      can mix a normal call and an `expect(() => ..., throwsArgumentError)` closure.
    - One clippy false-positive: `verify("3.141", 3.141)` (a literal decimal-parsing test, ported
      verbatim from dart) trips `clippy::approx_constant` purely because `3.141` is lexically close
      to `PI` — scoped `#[allow(clippy::approx_constant)]` on `number()` rather than changing the
      test value, since the literal is intentionally arbitrary, not a botched constant.
    Several real bugs surfaced while writing `expression_tests.rs`'s ~1000
    lines, found by checking every position/message against the real `dart` SDK (`dart test
    test/expression_test.dart` and ad hoc scratch scripts) rather than hand-deriving them:
    1. **`ExpressionBuilder::group()` silently discarded every registered operator.** It returned
       an owned `ExpressionGroup<T>` cloned into `self.groups` *before* the caller had a chance to
       call `.prefix()`/`.left()`/etc. on it — so every mutation landed on a throwaway snapshot,
       and `build()` always folded over empty, no-op groups. Caught with a probe test
       (`.left('+', ...)` registered, then `"1+2"` parsed as `1.0` instead of `3.0` — the `+` was
       never recognized at all). Fixed by returning `&mut ExpressionGroup<T>` borrowed from
       `self.groups.last_mut()` instead. Rode along: `ExpressionBuilder::build` changed from
       `&mut self` to consuming `self` by value, since that's what let `group()`'s fix work
       cleanly and also removed the `.clone()`s `build()` previously needed just to move `self
       .groups`/`self.primitives` out from behind a `&mut self` borrow.
    2. **`ExpressionBuilder::build()` returned the wrong value, leaking a dangling weak reference.**
       After `self.loopback.set(parser)`, it returned `parser` directly instead of `self.loopback`.
       Since `build(self)` consumes the whole builder — including `self.loopback`, the *only*
       strong `Rc` keeping the shared `RefCell` alive — returning anything other than `loopback`
       itself meant that `Rc` got dropped the instant `build()` returned. Every `SettableParserRef`
       embedded in the tree (e.g. inside a `wrapper()`'s recursive sub-parser) instantly became
       dangling, surfacing as `.expect("SettableParser owner dropped")` panics the moment anything
       actually recursed (e.g. parsing `"(1 + 2)"` — non-recursive inputs like `"1+2+3"` never hit
       it, which is why a quick sanity check across a few inputs, not just one, mattered). Fixed by
       returning `self.loopback` itself (which already delegates to `parser` after `.set()`) —
       behaviorally identical, but now the caller holds the strong owner for exactly as long as
       they use the returned parser. Not the same class of bug as the `T: Clone` story elsewhere in
       this doc — that one is about *value* reproduction; this one is about *parser object*
       lifetime, a cost specific to choosing `Rc`/`Weak` over dart's GC.
    3. **`choice_of`'s default failure-joiner didn't match dart's.** Checked dart's source directly:
       `ChoiceParser`'s default `failureJoiner` is `selectLast` (always the failure of whichever
       alternative was tried *last*, regardless of how far any other alternative actually got) —
       confirmed by tracing why dart reports `'('` as failing with `'number expected'` at position
       0 (not the seemingly-more-obvious position 1, where the paren-wrapper's own recursive
       attempt actually bottoms out) and verifying against the real dart SDK. Our `choice2`/`choiceN`
       default to `SELECT_FARTHEST` instead — a deliberate, pre-existing, unrelated choice for the
       rest of the library, not something to change globally. Fixed narrowly: `group.rs`'s internal
       `choice_of`/`select_last` helper (used only by `ExpressionBuilder`/`ExpressionGroup` to build
       dart's `buildChoice`-equivalent) now explicitly overrides each pairwise `choice2` with
       `joiner: SELECT_SECOND`, which — applied pairwise through the fold — reproduces `selectLast`'s
       N-ary behavior exactly.
    4. **`build_right`'s repetition shape didn't match `build_left`'s (or dart's).** `build_left`
       correctly parses "term, then zero-or-more `(op, term)` pairs" (`seq2(inner, seq2(op,
       inner).star())`), mirroring dart's `inner.plusSeparated(sep)`. `build_right` instead parsed
       "zero-or-more `(term, op)` pairs, then one mandatory final term"
       (`seq2(seq2(inner, op).star(), inner)`) — equivalent for fully-valid input, but with a
       different backtracking boundary: a *greedy* `.star()` of `(term, op)` pairs has nothing to
       give back to the mandatory trailing term once it's consumed everything, so a trailing
       dangling operator (`"1 ^ 2 ^"`) or an exact-multiple-of-pairs input (`"ab"` against
       `epsilon_with(())`-based right-recursion) caused a hard failure instead of gracefully
       succeeding with whatever had already matched (dart backtracks the *whole* failed `(sep,
       inner)` unit, never committing the dangling separator at all). Fixed by restructuring
       `build_right` to the same "term, then `(op, term)*`" shape as `build_left`, with the fold
       itself reversed (pop terms/ops from the end in lockstep) instead of the repetition shape.
       Caught by two test failures (`pow_error`'s `"1 ^ 2 ^"` case and `builder_epsilon_right`),
       both of which disappeared once the shape was fixed — no other tests regressed.
  - **lisp**, **prolog** — full interpreters (cons cells, environments, native functions), not
    just grammars. Substantial scope beyond parsing.
  - **dart** (the grammar, not this project) — done (`tests/example-grammars/dart.rs`, ~180
    rules via `#[grammar]`, 81 tests ported from dart's own `test/dart_test.dart`). Every rule —
    including leaf lexical tokens — is typed `impl Parser<()>` throughout, a deliberate departure
    from pascal.rs's partial value-preservation: confirmed exhaustively that
    `dart_test.dart`'s ~570 lines never inspect any extracted parse value, only call
    `isSuccess(input)`/`isFailure(input)` (implicit end-of-input + no-exception checks), so a
    full erasure design loses no test coverage while sidestepping type-juggling across ~100+
    productions. **Erasure convention: every rule body ends in `.constant(())`** — not
    `.map(|_| ())` — unless it's a bare delegating call to an already-`()`-typed rule; a leaf
    construct used inline as one `choiceN(...)` arm gets `.constant(())` at that inline site,
    since `choiceN`'s arms must share one value type. `.constant(value)` is the dedicated
    combinator for "discard the delegate's matched value, succeed with a clone of `value`
    instead" — exactly this shape — versus `.map(|_| ())`'s closure expressing the same thing
    more indirectly; prefer it over `.map(|_| ())` everywhere in this codebase, not just here.
    (Originally written as `.map(|_| ())` throughout; swept to `.constant(())` across
    `dart.rs`/`pascal.rs`/`bibtex.rs`/`smalltalk/mod.rs` once this was noticed.)
    - Rust-keyword collision: dart's rule named `type` → `dart_type` (mirrors pascal's `type` →
      `pascal_type`).
    - `token_str`/`token_parser` (the free helper functions outside the `#[grammar]` module) have
      only one code path here — unlike pascal.rs's keyword-vs-plain-literal branching
      (`Rc<dyn Parser<String>>` return type, `is_keyword` check) — because checking dart's actual
      `grammar.dart` source confirmed its own `token()` has no keyword/word-boundary guard at all
      (unlike dart's *pascal* grammar, which does — verified by reading both upstream sources side
      by side before deciding, rather than assuming one dart grammar's conventions apply to
      another). Faithfully ported as the simpler one-path version; not an improvement over
      upstream, a fidelity check.
    - Combinator-arity ceiling hit in three places (manually split, found by counting before
      writing rather than via compiler errors): `class_definition()`'s native-class alternative
      (10 seq parts → nested `choice2` of two `seq5`-pairs joined by an outer structure),
      `non_labelled_statement()` (12 choice alternatives → outer `choice2` of two `choice6`s),
      `assignment_operator()` (13 choice alternatives → outer `choice2` of `choice7` + `choice6`).
    - Two transcription bugs self-caught while writing (not found via tests, since this grammar
      has no recursive-value assertions to catch them — caught by re-reading the freshly-written
      code against dart's source): `import_directive()`'s show/hide clause
      (`((show|hide) selector) identifier.plusSeparated(',')).optional()`) was initially written
      as nonsense placeholder code, and was entirely *missing* from the sibling `export`
      alternative (a 3-part `seq3` that needed widening to `seq4`); `factory_constructor_declaration()`
      was missing its final `formalParameterList` part (`seq4` widened to `seq5`).
  - **smalltalk** — done (`tests/example-grammars/smalltalk/{ast,mod}.rs`, ~60 rules via
    `#[grammar]`, 29 test functions covering all 144 `verify(...)` cases from dart's
    `test/smalltalk_test.dart` plus the `start`/full-method smoke test). Unlike dart.rs, this
    grammar *does* build a real AST (smalltalk_test.dart's matchers inspect `.value`/`.name`/
    `.receiver`/`.selector`/`.selectorType`/`.arguments`/etc.), so the value-erasure trick doesn't
    apply here.
    - **One grammar, not two.** dart keeps `SmalltalkGrammarDefinition` (pure recognizer) and
      `SmalltalkParserDefinition extends ...` (AST-building override) as separate classes, then
      `verify()` runs both per case. Since the AST-building `.map()` never changes success/failure
      (only the returned value), a second, value-erased ~50-rule copy of the same grammar would
      carry no test signal beyond what the AST-building version's success already implies — built
      one grammar producing real AST values directly, and folded dart's "grammar" sub-test (just
      checks parsing doesn't throw) into the "parser" sub-test's success check.
    - **AST design**: a single recursive `Node` enum (`Literal`/`Variable`/`Assignment`/
      `Message`/`Cascade`/`Array`/`Block`/`Return`) plus separate `Method`/`Pragma`/`Sequence`/
      `Literal` types (`tests/example-grammars/smalltalk/ast.rs`) — a deliberate simplification of
      dart's class-per-node-type hierarchy (`ValueNode`/`MessageNode`/`CascadeNode`/etc., all
      `extends`/`with` mixins). Token bookkeeping (dart's `IsSurrounded.beforeToken/afterToken`,
      `BlockNode.separators`, `CascadeNode.semicolons`, `HasStatements.periods`) and the
      `Visitor`/`NodeCollector` machinery are both dropped — confirmed first that
      `smalltalk_test.dart`'s matchers never inspect any of that (only the logical shape), and
      that `NodeCollector.allNodes(ast)`'s one use (a non-empty check) is trivially true for any
      parsed AST, so neither carries any test signal here. dart's `selectorType` is a getter
      derived from the selector string and argument count at read time; ported as a constructor-
      time computation (`selector_type_of`) instead of a separately-threaded flag.
    - **Dynamic-list flattening sidestepped entirely.** dart's `buildUnary`/`buildBinary`/
      `buildKeyword`/`addTo<T>` exist because petitparser's untyped combinators return raw nested
      `List<dynamic>`s that have to be walked and filtered by runtime type at AST-construction
      time. Our statically-typed `seqN`/`mapN` combinators already produce exact tuples — every
      "message part" rule (`unary_message`/`binary_message`/`keyword_message`) returns a plain
      `(String, Vec<Node>)` tuple directly, and `build_message`/`build_cascade`/`build_assignment`
      (mirroring dart's `buildMessage`/`buildCascade`/`buildAssignment`) fold those tuples with no
      flattening step at all.
    - **`token(source, message)`'s dynamic dispatch became two statically-typed helpers.** dart's
      single `token()` checks at runtime whether its argument is a `String` or a `Parser`
      (`ArgumentError` otherwise — not ported, unreachable under static typing). Split into
      `token_str` (literal punctuation/keywords, always erased to `()`, same shape as dart.rs's
      helper of the same name) and `token_parser` (value-preserving generalization dart.rs never
      needed, since dart.rs erases everything to `()` but smalltalk's AST needs real values out of
      identifier/selector/number/string tokens).
    - **Numbers computed directly, not string-built-then-reparsed.** dart's `buildNumber` parses a
      *string* assembled from the matched span (`numberToken().value`) back through
      `num.parse`/`int.parse`. Ported as direct `f64` computation while parsing instead (radix
      integers via `i64::from_str_radix`, decimals via `"{int}.{frac}".parse()`) — same result, no
      round trip. `exponent()` faithfully requires a literal `-` (ported as-is from dart's
      `char('-').seq(decimalInteger)`), so a positive exponent like `3e4` isn't actually reachable
      through this grammar — untested upstream too (no `dart_test.dart` case exercises it), left
      as-is per the "port bugs faithfully" convention rather than silently fixing it.
    - **`binary()` uses `one_of(...)`, not `pattern(...)`.** dart's binary-selector-character rule
      is `anyOf(r'!%&*+,-/<=>?@\|~')` — an exact character *set* (`anyOf`, our `one_of`), not a
      range-based char class. Using `pattern(...)` here instead would have been a real bug: that
      literal char sequence contains `,-/` and `@\|`, which a range-aware `pattern()` parser could
      misread as range syntax (comma-to-slash); `one_of`'s exact-set semantics sidestep that
      entirely. Caught during design, before writing the rule, by checking dart's actual
      `anyOf`-vs-`pattern`-equivalent call rather than assuming.
    - **`statements()`'s leading/double/trailing-period handling, verified against the real dart
      SDK before porting.** dart's `statements()` = `(expressionReturn|expression)
      .starSeparated(periodToken.plus()).skip(after: periodToken.star())` handles double periods
      (`"1 . . 2"` → two statements, via the separator itself being `periodToken.plus()`, which
      greedily eats consecutive periods as one separator) and trailing periods (`"1 . 2 . 3 ."` →
      three statements) but conspicuously has no leading-period handling of its own — that's
      actually `sequence()`'s job (`temporaries().seq(periodToken().star()).seq(statements())`,
      with the middle `.star()` eating any leading periods before `statements()` ever runs); all
      of dart's `Statements*`/`Sequence*` test cases go through `grammar.sequence`, never bare
      `grammar.statements`, which is what makes this division of labor not show up as a gap.
      Resolved by writing a small scratch dart program against the real SDK
      (`dart run bin/_scratch_check.dart`, using `resolve(production()).end()` — raw `ref0(...)`
      output has no `.end()` until resolved) to confirm `". 1"`/`".1"`/`"1 . . 2"`/`"1 . 2 . 3 ."`
      all actually succeed before committing to a design, rather than guessing from the source
      alone. Ported as `choice2(expression_return(), expression()).star_sep(period_token().plus(),
      Trailing::Allowed)` for `statements()` — our existing `Trailing::Allowed` (a deliberate
      extension beyond dart, see `SeparatedList` above) turns out to exactly reproduce dart's
      "trailing `periodToken.star()`" behavior for free, since allowing *one* occurrence of a
      separator that is itself `.plus()`-greedy is equivalent to allowing *zero-or-more* trailing
      periods.
    - **One real bug, caught immediately by a dedicated test, not deduced in advance:**
      `comment()` was first written as `seq3(char('"'), char('"').not().star(), char('"'))` —
      `.not()` is zero-width (a lookahead that doesn't consume), so `.star()` over it never
      advances position and panics on the "delegate succeeded without consuming" infinite-loop
      guard. The fix is `.neg()` (`any().skip_left(self.not_with_message(...))` — consumes one
      character *while* checking it isn't a quote), which is what dart's `char('"').neg().star()`
      actually uses; `.not()` and `.neg()` look similar but are not interchangeable, and this
      grammar is the first place in the codebase that needed `.neg()` for repetition rather than a
      one-off guard.
    - **`seq!` macro didn't work inside `#[grammar]` rule bodies that call other rules — fixed
      since.** Tried `seq!(period_token().star(), pragmas(), ..., |closure|)` inside
      `method_sequence()`; failed with `E0618` (`period_token`/`pragmas`/etc. resolved to their
      `SettableParser<T>` *fields*, not function calls) because the `#[grammar]` macro's
      `VisitMut` rule-name rewriter only walks parsed `Expr` nodes (`Expr::Call`, recursing into
      its arguments) — a `seq!(...)` invocation is an opaque `Expr::Macro` at that point in
      expansion, so the rewriter never saw the zero-arg rule calls inside its token stream to
      rewrite them to `.borrow()`. Worked around at the time by using the explicit
      `seq8(...).map8(...)` form instead, which *is* plain nested `Expr::Call`s the rewriter
      walks normally. The underlying limitation is now fixed (see "Fixed: `seq!`/`choice!` now
      work inside `#[grammar]` modules" in "What's Implemented"), and `method_sequence()` has
      since been converted to `seq!(...)` with the `=>` arrow form, along with the rest of the
      `pascal.rs`/`bibtex.rs`/`math.rs` sweep.
  - **regexp** — a self-contained regex-engine-with-NFA project, conceptually separate from
    "porting a grammar."
  - Decided: keep self-contained grammar+test examples (tabular-shaped) in
    `tests/example-grammars/`; revisit a separate `rust-petitparser-examples` workspace crate only
    when something needs a real binary (lisp/prolog REPLs) or its own dependencies.
- Port remaining dart-petitparser tests toward parity (scope A = features we already have).
  Done so far: character/predicate gap-fills, `string_ignore_case`, `context_tests.rs`;
  `lazy > repeat` block in `repeater_tests.rs` (remaining `repeat_lazy` cases, unbounded
  `repeat_lazy`, and the `star_lazy`/`plus_lazy` non-consuming-delegate panic tests, with a
  shared `panic_message` helper); newline + `position()` tests (`misc_tests.rs` — writing the
  bare-`\r` case here surfaced and fixed a real `NewlineParser::parse_on` panic: it indexed
  `buffer[position + 1]` to check for `\r\n` without first checking that index was in bounds,
  so a trailing `\r` with nothing after it crashed instead of matching as a lone `\r`); combinator
  choice failure-joiner matrix (select-first/select-last/farthest/farthest-joined, positions
  hand-verified against dart's matrix — message *text* follows our own "expected X, but
  found/reached Y" convention, not dart's "X expected" suffix style), `seq3` per-position
  failures, `settable` passthrough, `skip` none/before-only/after-only, `.neg()`
  (`combinator_tests.rs`); richer `map`/`.input()` (nested composition)/`trim_with`
  (custom-delimiter)/a full `Token` accessor matrix (value/buffer/start/end/length/line/column/
  input across mixed `\n`/`\r`/`\r\n` line endings) (`action_tests.rs`); possessive
  `rep` unbounded (100k-element stress test) and the full `greedy > star/plus/repeat/repeat
  unbounded/infinite loop` group (`repeater_tests.rs`) — this is what surfaced the two
  `GreedyRepeatingParser` bugs described above; representative sequence subset beyond `seq3`
  (extended `seq4_test` with all 4 failure positions, added `seq9_test` to confirm the pattern
  holds at the macro-generated max arity too) (`combinator_tests.rs`).
  **Scope A is now fully ported** — nothing left in the "remaining portable-now" bucket.

  ### >>> THE IMMEDIATE NEXT STEP: finish the linter <<<
  The linter infrastructure is now fully landed. **Done so far:**
  - `src/reflection/analyzer.rs` — `Analyzer` struct with DFS graph walk (`parsers` Vec), fixed-
    point first-set computation, and cycle-set computation. Public API: `new(root)`, `parsers`,
    `is_nullable(&p)`, `first_set(&p)`, `cycle_set(&p)`. `PtrKey = *const ()` for Rc identity.
    Sentinel pattern: a private `Rc<dyn HasChildren>` wrapping `EpsilonParser<()>` is seeded into
    first_sets as the nullable marker; NOT in `parsers`, but IS in `by_ptr`.
  - `src/reflection/linter.rs` — `LinterType` (Info/Warning/Error), `LinterIssue`, `LinterRule`
    trait, `linter(root, rules) -> Vec<LinterIssue>` walker.
  - `src/reflection/linter_rules.rs` — `NullableRepeater` and `UnresolvedSettable` rules;
    `ALL_LINTER_RULES` constant. Exported from `prelude.rs`.
  - `tests/linter_tests.rs` — 41 tests covering graph traversal, is_nullable, first_set,
    cycle_set, UnresolvedSettable, NullableRepeater, and ALL_LINTER_RULES smoke tests. All pass.
  - **Key bugs fixed during this work:**
    - `compute_first_sets` seeding: built `p_first_set` but forgot `first_sets.insert(ptr(&p),
      p_first_set)` — silently discarded every seeded first_set before the fix.
    - **Blanket `impl HasChildren for Rc<P>` only delegated `children()`** — all kind-query
      methods (`is_directly_nullable`, `is_sequence`, `is_choice`, `is_repeating`,
      `is_separated_repeating`, `is_settable`, `is_undefined`, etc.) fell through to the trait
      default returning `false`. Root cause: when `p: Rc<dyn HasChildren>` and you call
      `p.is_directly_nullable()`, Rust uses the static blanket impl (which doesn't override it)
      rather than the vtable. Fix: add `(**self).method()` delegation for every method in the
      blanket impl (`src/core/parser.rs`). This was the root cause of ALL is_nullable/linter-rule
      failures — once fixed, 39 of 41 tests passed immediately.
    - `SettableParser::is_undefined()` previously called `self.delegate.borrow().is_undefined()`
      on `Rc<dyn Parser<T>>` — same blanket-impl problem. Now fixed transitively by the blanket
      impl fix above (the borrow gives `Ref<Rc<dyn Parser<T>>>`, and calling `is_undefined()` on
      `Rc<dyn Parser<T>>` now correctly delegates through to the underlying type).
    - `cycle_set()` public method was missing. Added.
  - **Structural deduplication gotcha (documented in tests):** `seq2(a.clone(), a.clone())` where
    `a: Rc<dyn Parser<T>>` does NOT deduplicate. `seq2`/`choice2` wrap each argument in
    `Rc::new(arg)`, creating a new outer allocation for each. Two clones of `a` → two distinct
    outer Rcs with different ptrs → DFS sees 3 nodes (seq2 + 2 wrappers), not 2. True structural
    sharing only works with `SettableParser`/`.borrow()` weak refs (no wrapping in `Rc::new`).
  - **First-set semantics:** a leaf parser (including `EpsilonParser`) IS inserted into its own
    first_set as a "start parser" (just like a char parser). When epsilon is a child of a choice or
    sequence, its own ptr propagates as a non-sentinel entry in the parent's first_set. The
    nullable sentinel is separate. So `choice2(char('a'), epsilon())` has a first_set with 2
    non-sentinel entries (not 1 as might be expected).

  **Rule status (13 dart rules total): PORT COMPLETE — 12 implemented, 1 deliberately skipped.**
  - **Done (12):** `NullableRepeater` ✓, `UnresolvedSettable` ✓, `LeftRecursion` ✓, `NestedChoice` ✓,
    `UnreachableChoice` ✓, `CharacterRepeater` ✓, `UnnecessaryFlatten` ✓ (ours: `UnnecessaryInput`),
    `UnoptimizedFlatten` ✓ (ours: `UnoptimizedInput`), `DuplicateParser` ✓, `RepeatedChoice` ✓,
    `OverlappingChoice` ✓, `UnusedResult` ✓. All in `src/reflection/linter_rules.rs`, each with a
    with-issue + without-issue test in `tests/linter_tests.rs` (63 tests).
  - **Equality core (landed):** property-carrying `#[non_exhaustive] enum ParserKind`
    (`src/core/kind.rs`) returned by `HasChildren::kind()`, deriving `PartialEq` with a `NeverEq`
    marker *struct* (`impl PartialEq { fn eq → false }`) for opaque closure/value variants
    (`Map`/`Constant`/`Epsilon`/`FlatMap`/`Continuation`/`Success`/`Other`) and `*const ()`/fn-ptr
    identity tokens for reference-equality variants (`PredicateChar`/`Predicate`/`OnlyIf`/
    `CharacterRepeating`). `structural_eq` + `parsers_equal` (fresh-`seen` wrapper) +
    `is_parser_iterable_equal` in `src/reflection/equality.rs`, porting dart's `isEqualTo` with the
    left-only `ptr()` cycle guard. `MapParser` (bare generic `f: F`) is the sole conservative-`false`
    case; `PredicateChar`/`Predicate`/`only_if` get faithful reference equality via `Rc::ptr_eq`/
    fn-ptr. `ParserKind::Other(Rc<dyn CustomParserKind>)` is the catch-all for custom `HasChildren`
    implementors (`TabularDefinition`): the payload is a `CustomParserKind` trait object (`Any +
    Debug`, with `impl PartialEq for dyn CustomParserKind` delegating to `eq_custom`) that lets a
    downstream parser opt into the equality-based lints by defining how two of its kind compare —
    typically a type check (`(other as &dyn Any).downcast_ref::<MyKind>().is_some()`), since children
    are compared by `structural_eq`'s recursion. `AlwaysDistinct` is the ready-made opt-out (never
    equal). This is the *one* place `Any`/downcast lives — isolated to the external-parser extension
    point, keeping the built-in core downcast-free. Tested in `src/core/kind.rs`'s `#[cfg(test)]
    mod`; `TabularDefinition` uses a real `TabularKind` (type check) as the in-repo example. The
    `impl HasChildren for Rc<P>` blanket delegates `kind()` through `(**self)`, and the `#[grammar]`
    macro emits `kind()` delegating to `self.start.kind()`.
  - **`UnusedResult` (landed):** uses `Analyzer::all_children(&p)` (transitive-children DFS,
    self-included only if cyclically reachable — matches dart's test matrix) + `is_result_producing`
    (`is_constant_parser`/`is_input`/`is_map_parser`/`is_elements_at_parser`/`is_pick_parser`/
    `is_token_parser`/`is_only_if`; `cast`/`castList` absent, all maps result-producing) + the full
    `ParserPath`/`find_path`/`find_all_paths`/`depth_first_search` machinery
    (`src/reflection/path.rs` + `analyzer.rs`) — the `sync*` generator ported as recursion returning
    a `Vec<ParserPath>` with `push`/`pop` backtracking and per-path (not global) `ptr` cycle
    avoidance. (The findPath machinery was built in full rather than the originally-planned
    simplified message.)
  - **`UnnecessaryResolvable`: DELIBERATELY SKIPPED** (like `cast`/`castList`). It's just
    `is_settable()` (no equality needed), but it flags every `ResolvableParser` because dart expects
    you to strip them via `resolve(parser)` before parsing. **Our architecture makes every
    `#[grammar]` rule a `SettableParser` that is load-bearing at parse time** — we have no
    `resolve()` pass and the settables *are* the recursion mechanism, so a faithful port would flag
    every rule in every grammar as a warning. We consciously accepted the "must resolve every
    parser" inefficiency when we chose the all-`Settable` design, so this lint doesn't apply to us.
    Not implemented; no test.
  - **Complete variant list for `ParserKind` (32 variants):** every concrete `HasChildren` impl
    needs a `kind()`. `SeqN`→one `Sequence`, `ChoiceN`→one `Choice { joiner }`; the three repeaters
    (`Possessive`/`Greedy`/`Lazy`) stay distinct; `SettableParser`+`SettableParserRef`→one `Settable`
    (judgment call — merge unless you want owner vs weak-ref to compare unequal); `UndefinedParser`
    is its own `Undefined { message }`. Data variants: `Char { kind: CharKind, message }`,
    `CharRepeating { test: *const (), message, min, max }`, `Choice { joiner: FailureJoiner }` (fn-ptr,
    not generic → no erasure), `ElementsAt { indexes: Vec<i32> }`, `End { message }`, `Failure { message }`,
    `Input { message }`, `Labeled { label }`, `Not { message }`, `Pick { index: i32 }`,
    `Predicate { predicate: *const (), length, message }`, `PredicateChar { test: *const (), message }`,
    `Regex { pattern: String, message }` (via `regex.as_str()` — `Regex` isn't `PartialEq`),
    `{Possessive,Greedy,Lazy}Repeating { min, max }`, `SeparatedRepeating { min, max, trailing }`
    (`Trailing` derives `PartialEq`). Reference (fn-ptr erased to `*const ()`): `OnlyIf { predicate,
    factory }`. Opaque (`NeverEq` marker, always compare unequal): `Constant`, `Continuation`,
    `Epsilon`, `FlatMap`, `Map`, `Success`. Unit: `And`, `Newline`, `Position`, `Sequence`, `Skip`,
    `Settable`, `Token`. **NOTE `SkipParser` and `SeparatedListRepeatingParser` are easy to miss** —
    their `impl … HasChildren` wraps the where-clause so `for TypeName` is on the next line, which a
    `grep "HasChildren for"` skips right past.
  - **Deferred refactor 1 — make `ParserKind` borrow (`ParserKind<'_>`), not own.** Currently
    `kind(&self) -> ParserKind` clones every message `String` (and `CharKind`, `Vec<i32>`, …) into an
    owned variant. Change to `fn kind(&self) -> ParserKind<'_>` with data variants holding `&'a str` /
    `&'a CharKind` / `&'a [i32]` borrowed from `&self` (`message: self.message.as_deref()`), so `kind()`
    allocates nothing. Sound because a `ParserKind` is always compare-then-discard: `structural_eq`
    builds both kinds, compares while both parsers are alive (we hold the `Rc`s), and drops them — no
    kind is ever stored. Only cost is a lifetime param on the enum; `*const ()`/unit variants are
    unaffected. Parked mid-implementation (owned+cloning works for now); revisit for the alloc win.
  - **Deferred refactor 2 — `Option<Rc<str>>` for message storage. RE-EVALUATED: probably NOT
    worth it; do not do it as a blanket sweep without a profile first.** The original idea: every
    parser's `message: Option<String>`/`String` → `Option<Rc<str>>`/`Rc<str>` (`Rc<str>`, not
    `Rc<String>`/`Rc<Option<String>>` — single indirection; `Option<Rc<str>>` keeps `None` free),
    making message clones refcount bumps, with the "real prize" being the parse-time failure path.
    A close look (2026-07 session) found the premise mostly doesn't hold:
    - **The `kind()`-clone motivation is gone** — deferred refactor 1 (borrowing `ParserKind`)
      already made `kind()` allocation-free, so messages no longer get cloned there at all.
    - **The "don't duplicate heap data across shared parsers" motivation is already covered** by
      the whole-parser `Rc<dyn Parser<T>>` storage (see "Storage model"): a reused sub-parser is one
      `Rc` pointed at N times, so its message/`Vec` fields exist *once*. The parse hot path clones
      *zero* parsers (every `parse_on`/`fast_parse_on` is `&self`); construction `Rc`-wraps rather
      than deep-copies. So concrete leaf value-clones (the only thing that would duplicate a message
      `String`) are rare and never on the parse path. Field-level `Rc` is therefore *second-order* on
      top of the already-pulled first-order lever, and doesn't even dedup *separately constructed*
      parsers (`string("x")` in three rules still builds three `Rc<str>`s).
    - **For the failure path specifically, `Rc<str>` can *regress* the hot case.** The most frequent
      failures are char parsers, and `CharParser::message_for` builds a *dynamic, unique-per-failure*
      message via `format!` (the `message: None` default path — the common one; only a custom
      `message: Some(m)` returns a stored clone). Making `Failure.message: Rc<str>` forces that
      `format!` → `String` → `Rc<str>` (`Rc<str>: From<String>` copies into a fresh refcounted
      buffer — **two** allocs where today there's one). `Rc` only helps *static/stored* messages that
      are cloned per failure (predicate/string/failure parsers — colder than char failures).
    - **The real parse-time lever is lazy message construction, not `Rc<str>`.** Most failures are
      built then immediately discarded (a choice drops the first alternative's failure; a repetition
      drops its terminating failure), yet `message_for` runs eagerly *before* `context.failure`. The
      win is deferring message construction until read (e.g. `context.failure(impl FnOnce -> String)`
      or `Failure.message` as `{ Eager(Rc<str>) | Lazy(...) }`), so a discarded failure never runs
      its `format!`. That's a different, more targeted change.
    Bottom line: only pursue `Rc<str>` for non-perf reasons (structs shrink 24→16 bytes per message
    field), and only after a profile shows leaf value-cloning / construction as a real cost. If
    parse-time failure cost is the goal, do the laziness change instead. Measure first — the
    "bites in backtracking-heavy grammars" claim was never benchmarked.
  - **Deferred refactor 3 — collapse the ~18 `is_*` boolean flags into `kind()`. DONE.** All the
    raw `is_*`/`repeating_min`/`input_message` methods were removed from `HasChildren`, the `Rc<P>`
    blanket (now just `children()`+`kind()`), and ~36 per-type overrides; every classification is a
    `matches!(p.kind(), …)`. `is_result_producing` stays as the one derived helper (built on
    `kind()`). Non-1:1 mappings handled: `is_char` → `Char | PredicateChar`, `is_repeating` → all
    four repeater variants, `is_directly_nullable` → a value-dependent helper in `analyzer.rs`
    (`Position | Epsilon | Success | <repeaters with min: 0>`); `repeating_min` was dead and deleted.
    Also **un-merged `Settable`/`SettableRef`** (`SettableParserRef::kind()` → new `SettableRef`
    variant) so `UnresolvedSettable` (reformulated as "owner is `Settable` **and** child is
    `Undefined`") still fires once per unresolved rule, and an owner vs a `.borrow()` of it no longer
    compare structurally equal (fixed a latent `DuplicateParser` false-positive). The historical
    rationale for the refactor is preserved below for context.
    ORIGINAL NOTE (gated on refactor 1, now both done): Now that `kind()` encodes everything the
    `HasChildren` booleans do, the flags (`is_char`/`is_choice`/`is_sequence`/`is_input`/
    `is_repeating`/`is_map_parser`/…) are redundant: `p.is_char()` → `matches!(p.kind(),
    ParserKind::Char { .. })`, etc. **Win:** ~73 override sites across ~30 files removed, `kind()`
    becomes the single source of truth (a parser can't have `is_char()` disagree with `kind()`), the
    `impl HasChildren for Rc<P>` blanket shrinks from ~18 delegations to just `kind()`+`children()`,
    and custom parsers get *safer* (they return `Other`, so every `matches!` correctly yields
    `false` — they can't lie by overriding a flag). **Why gated on #1:** `is_directly_nullable`/
    `is_sequence` run inside the analyzer's fixpoint loops (`compute_first_sets` iterates to
    convergence, hitting them on every node each pass); replacing cheap bools with
    `matches!(p.kind(), …)` while `kind()` is *owned* builds+clones a whole `ParserKind` (message
    `String`s, `CharKind`, `Vec<i32>`) per check just to discard it — a per-check allocation in a hot
    loop. Borrowing `kind()` (refactor 1) makes `matches!` alloc-free, so the trade is only clean
    afterward. **Three non-1:1 mappings to reproduce carefully** (the rest are trivial swaps):
    (a) `is_repeating` covers *four* variants — `Possessive`/`Greedy`/`Lazy` **and
    `SeparatedListRepeating`** (all four override it; forgetting the separated one silently breaks
    `NullableRepeater` for separated repeaters); (b) `repeating_min` extracts the `min` field, so
    it's a `match … { Possessive{min,..} | … => Some(min), _ => None }`, not a `matches!`;
    (c) `is_directly_nullable` depends on field *values* (`min == 0` for repeaters, plus
    `Epsilon`/`Position`/…), so its replacement needs `min: 0` patterns — reproduce its exact
    per-type truth table. **One method resists:** `input_message()` returns a borrowed `&str` from
    `self`, which an owned `kind()` can't hand back — but its only caller does `.is_none()`, i.e.
    `matches!(kind, Input { message: None })`, so just don't try to recover the borrowing accessor
    from `kind()` (after refactor 1 a borrowing accessor works too). The 63 analyzer+linter tests
    cover semantic drift. `is_result_producing` (a default OR of several flags) becomes a `matches!`
    over several variants.

  The kind-flag approach (methods on `HasChildren`) is fully working. The 13 dart classes map to
  `HasChildren` method overrides: `is_repeating`/`is_separated_repeating`/`is_possessive_repeating`/
  `repeating_min`, `is_sequence`/`is_choice`/`is_settable`/`is_undefined`/`is_directly_nullable`,
  `is_char`/`is_string_predicate`/`is_input`/`is_newline`/`is_char_repeating`/`is_map_parser`/
  `is_constant_parser`/`is_token_parser`/`is_only_if` — all already defined on `HasChildren` with
  appropriate overrides in each parser type.

  Other deferred reflection bits: `copy`/`transformParser` (parser-graph copying + rewriting) —
  still not ported, and now the gating dependency for the debug tools (see the dedicated "Debug
  tools" entry below). Deep-equality is **done** (the `ParserKind`/`structural_eq` core landed with
  the linter). `copy`/`transformParser` are also what dart's `expectParserInvariants` assertions and
  `optimize`/`replace`/`resolve` machinery need, none of which we've ported. Regex char parsers are
  no longer in this deferred list — see "Ported dart's regex-backed `PatternParser`" above.

  **Parked (prototyped, reverted): `FnWithText` — capture a mapping function's source text for
  `Debug` output.** The pain point: a `MapParser` (and any function-carrying parser) prints
  `f: "<mapping function>"`, so you can't tell which one you're looking at or what it does — and
  since we deliberately don't implement `Display` for parsers, `Debug` is the only view. Goal: let
  a parser print the actual code of its closure (e.g. `"|c| c.to_digit(10).unwrap() + 1"`) or a
  named function's name (`"add_one"`). Prototyped on `MapParser` only, confirmed working, then
  reverted to keep the linter work clean — captured here so it can be rebuilt.
  - **The appealing-but-impossible API: `p.map(func!(|x| ...))`.** The idea was a wrapper struct
    `FnWithText<F> { f: F, text: Option<&'static str> }` where `func!(closure)` fills `text` via
    `stringify!`, plus a bare closure coercing in through `impl Into<FnWithText<F>>` so both forms
    work. It fails on stable for two independent reasons: (1) you **cannot `impl Fn` on
    `FnWithText`** — that needs the unstable `fn_traits`/`unboxed_closures`, so the wrapper can't
    be a transparent drop-in callable (not actually needed, since combinators own their call site
    and just call `(self.f.f)(x)`); and (2) — the real killer — **`impl Into<FnWithText<F>>`
    breaks closure-parameter inference.** Routing the closure through an `Into` layer means the
    `where F: Fn(T) -> U` bound no longer flows back to pin the closure's parameter type when the
    literal is checked, so `.map(|x| ...)` fails with `E0282` (type annotations needed) on real
    call sites like `rep_sep`'s `.map(|sl| sl.elements)`. Not fixable by rearranging bounds — a
    closure literal needs its *expected type* to be directly `Fn`-shaped, and `impl Into<...>`
    isn't. Forcing `.map(|x: T| ...)` annotations everywhere is a non-starter.
  - **The form that works: a `map!` macro + a text-taking method.** Keep `.map(f)` exactly as-is
    (bare `f: F`, inference intact), add `map_with_text(f, text: Option<&'static str>)`, and a
    `map!(p, |x| ...)` macro expanding to `p.map_with_text(|x| ..., Some(stringify!(|x| ...)))` —
    the closure stays a *direct* argument to a method whose parameter is `F: Fn(T) -> U`, so
    inference works and `stringify!` runs on the same tokens. `MapParser` stores
    `f: FnWithText<F>`; `Debug` prints `self.f.text.unwrap_or("<function>")`. Verified output:
    plain `.map` → `f: "<function>"`; `map!` closure → `f: "|c| c.to_digit(10).unwrap() + 1"`
    (`stringify!` kept it nearly verbatim, not heavily re-spaced); `map!` named fn → `f: "add_one"`.
    Full suite + clippy stayed green. The tradeoff: the nice-print call site is `map!(p, f)`, not
    `p.map(func!(f))` — same info, one macro at the front instead of wrapping the argument.
  - **User's stated preference for if/when we return to this:** not opposed to *forcing* a
    `func!(...)`-style wrapper at **every** site a parser takes a function, accepting the churn,
    since it improves Debug cheaply. That means changing every bare `Fn` field in the parser
    structs (`MapParser.f`, `OnlyIfParser.predicate/factory`, `FlatMapParser`, `ContinuationParser`,
    the `mapN`/`seq! =>` expansions, …) to `FnWithText`. **Open question to test first if we go
    this route:** whether a single macro-only path (no bare-closure path, method takes
    `FnWithText<F>` directly) preserves inference — the per-combinator `map!`→direct-arg form
    *definitely* does (proven above); a unified `func!`-wrapper-into-a-`FnWithText`-taking-method
    has the same nested-closure-in-a-struct-literal inference risk as the `Into` form and must be
    verified before committing to the sweep.
  - **Keep this off the equality path** (same trap as the `#[track_caller]` idea): `text` is not
    identity — two `map!(p, |x| x+1)` at different sites have identical text but are different
    closures, so text-equality would false-positive `RepeatedChoice`. `MapParser` equality stays
    conservative-`false` regardless; `FnWithText` is purely for printing. (An alternative discussed
    and passed over: `#[track_caller]` capturing `file:line:col` instead of text — no macro, no
    call-site changes, but gives a location rather than the code.)

  **Deferred: cycle-safe recursive `Debug` for `SettableParser`/`SettableParserRef`.** Surfaced
  while writing `LeftRecursion`'s message (which formats each `analyzer.cycle_set(parser)` member
  via `{:?}`): `SettableParserRef`'s `Debug` just does `format!("{:?}", self.delegate)` on a
  `Weak<...>`, and `std::rc::Weak`'s own `Debug` is hardcoded to always print the literal string
  `"(Weak)"` — it never upgrades, so this is no more informative than the placeholder it replaced.
  The safe-but-uninformative behavior is not an accident to just patch over: naively upgrading and
  recursing into the target risks genuine infinite recursion/stack overflow for a real self-
  reference (`s.set(s.borrow())`), where the target *is* the same `SettableParserRef` being
  printed — `Weak`'s refusal to look past itself is accidentally what keeps today's code safe.
  dart sidesteps this entirely because `Parser.toString()` is just `'$runtimeType'` (shallow, never
  recurses into children) — there's no equivalent "just print the class name" for a `dyn` trait
  object in Rust without hand-building one from `HasChildren`'s existing kind-flags
  (`is_settable`/`is_choice`/`is_sequence`/`is_char`/etc., all already non-recursive one-hop checks
  built for the linter). Two designs discussed, not yet built:
  - Minimal: a `kind_name(&Rc<dyn HasChildren>) -> &'static str` built from the kind flags, used by
    `SettableParserRef::fmt` after `.upgrade()` to print e.g. `"-> SettableParser"` instead of
    `"(Weak)"` — bounded depth 1 always (never calls `{:?}` on the target, just inspects its kind),
    so no recursion risk. Cheap, but still not a full "show me the structure" debug view.
  - Fuller: genuinely recursive `Debug` with cycle detection, so e.g. `dbg!(some_recursive_grammar)`
    prints the real nested structure and only stops with a back-reference marker
    (`"-> #2 (cycle)"`) when it's about to revisit a node already on the current print's path.
    Needs a `thread_local!` `Vec<*const ()>` "currently visiting" stack (reusing the `ptr()`/
    `PtrKey` pattern from `src/reflection/analyzer.rs:8-12` for the identity) — `Debug::fmt`'s
    fixed `(&self, &mut Formatter) -> Result` signature has no room to thread state as a parameter,
    so cross-call state needs either this or a dedicated non-`Debug` pretty-printer function taking
    an explicit `&mut HashSet<*const ()>` (the latter is arguably cleaner — no global mutable
    state — but means callers can no longer just reach for `{:?}`). Must use a `Drop` guard to pop
    the stack on every exit path (including `?`-propagated ones), not manual push/pop, or an early
    return leaves stale entries poisoning later prints in the same thread. For the numbering to be
    legible, both `SettableParser::fmt` and `SettableParserRef::fmt` need to tag their *own* output
    with their push-time stack position, not just detect repeats. Scope question: a cycle can only
    *correctly* close through `SettableParserRef` (the documented `.borrow()`-for-back-references
    rule), so guarding just that type covers every `#[grammar]`-generated grammar; guarding
    `SettableParser::fmt` too is defense-in-depth against someone violating that rule by hand (see
    the next entry — that violation is a real, distinct bug worth its own detector, not just safe
    `Debug` output).
  - Not blocking `LeftRecursion` itself: `Analyzer::cycle_set` already returns the flat,
    deduplicated cycle membership directly (no need to rediscover it by recursing through `Debug`),
    so the minimal `kind_name` option is enough to make that one rule's message useful; the fuller
    recursive design is a bigger, more generally-useful thing (ad hoc debugging of any recursive
    grammar) that doesn't need to gate finishing the remaining linter rules.

  **Deferred, Rust-specific — no dart equivalent, dart has no strong/weak distinction to get
  wrong: detect `s.set(s.clone())` used where `s.set(s.borrow())` was meant.** This is a real,
  easy-to-make mistake given the codebase's own "Rule: use `.clone()` for forward references; use
  `.borrow()` only for back-references" convention (Key Design Decisions) — get it backwards on a
  self/mutual reference and you've built a genuine strong `Rc` cycle: leaks memory forever (never
  reaches refcount 0) and, per the entry above, would stack-overflow on `Debug` print once/if that
  becomes recursive. **Investigated whether this is detectable with today's infrastructure — it is
  not, without a new signal.** The reason: `s.set(s.borrow())` (correct) and `s.set(s.clone())`
  (broken) produce *byte-for-byte identical* `HasChildren::children()` graphs. Traced both by hand:
  in the correct case, `SettableParserRef::children()` upgrades the `Weak` and returns the current
  `RefCell` content, which (post-`.set()`) is the `SettableParserRef` itself — a length-1 self-loop
  at that node's own `Rc::as_ptr`. In the broken case, `SettableParser::children()` reads the same
  `RefCell` (shared via the `.clone()`) and gets back the exact same shape: a length-1 self-loop at
  that node's own `Rc::as_ptr`. `Analyzer`'s whole graph-walk is built on `Rc::as_ptr` identity over
  `children()` — which erases whether an edge was a strong `Rc` or a `Weak` upgrade, so nothing
  built purely on `children()` (including `cycle_set`, which is further scoped to zero-width/
  nullable-prefix reachability for left-recursion specifically — a different, narrower question
  than "is there an unbroken strong cycle anywhere") can tell these two cases apart. To make it
  detectable: add a new `HasChildren::is_weak_reference(&self) -> bool { false }` default,
  overridden `true` only by `SettableParserRef` (the *only* type that can legitimately close a
  cycle — it exists exclusively via `.borrow()`, never by misuse). Then a new rule needs its own
  graph walk (can't reuse `cycle_set` as-is — that one deliberately stops at a sequence's first
  non-nullable child, which is wrong here: a strong `Rc` cycle leaks memory and crashes `Debug`
  print regardless of whether the recursive edge is reachable without consuming input, e.g.
  `s.set(seq2(char('a'), s.clone()).map(...))` parses fine — no `LeftRecursion` issue, the `'a'`
  breaks the infinite-parse-loop concern — but still leaks and still isn't `Debug`-safe) — walk
  *all* children unconditionally, and flag any path that revisits a node already on the current
  path without having crossed an `is_weak_reference() == true` node along that segment. Severity
  probably `Error` (it's an unconditional leak + latent crash, not a style nit). Name TBD (not one
  of the 13 dart rule names, since dart has no strong/weak split to misuse) — something like
  `UnbrokenSettableCycle` or `StrongSelfReference`.
  (`not_with_message`/`neg`/`neg_with_message`/`pattern`/`pattern_ci`/custom-delimiter
  `trim_with`/greedy repeaters/graceful `undefined()` failure/`opt_with`/`continuation`
  (`call_cc`)/`range`/`accept`/`accept_at`/`elements_at` (dart's `permute`, with a `permute`
  alias)/`pick`/`Token::join`/string repeaters (`star_string`/`plus_string`/`times_string`/
  `rep_string`)/`SeparatedList<T, Sep>` typed results with a `Trailing` flag (`rep_with_sep`/
  `star_with_sep`/`plus_with_sep`/`times_with_sep`)/`SeparatedList`'s own utility methods
  (`sequential`/`fold`/`rfold`/`Display`, dart's `sequential`/`foldLeft`/`foldRight`/`toString`,
  see "What's Implemented") are now implemented. `Token::join` is a notable example of why
  `T: Clone`/`Debug` bounds belong on the *method* that needs them, not the surrounding
  `impl<T> Token<T>` block — consuming the input iterator by value to build the result `Vec<T>`
  means `join` doesn't actually need `T: Clone` at all, and an earlier draft that collected into
  a `Vec<Token<T>>` and cloned out of it broke `Token::new` for every other caller by putting the
  bound on the whole block instead.)
- **Fixed: `fast_parse_on` side-effect gap.** `fast_parse_on`'s contract is "compute the resulting
  position only, no value needed" — `InputParser` already honored this. `MapParser`,
  `ConstantParser`, and `TokenParser` did not override it at all, falling back to the default blanket impl (which
  calls `parse_on` and discards the position) — meaning their closures/cloning/`Token`-building
  *did* run on the fast path, contrary to what "position-only" implies. Checked dart's actual
  source (`lib/src/parser/action/{map,where,token}.dart`) before fixing, to confirm which ones
  are *supposed* to skip: dart's `MapParser.fastParseOn` delegates straight through when
  `!hasSideEffects` (its default), and `TokenParser.fastParseOn` always delegates straight
  through — confirming both were genuine gaps here, not deliberate design. Fixed by adding
  `fast_parse_on` overrides to `map.rs`/`constant.rs`/`token.rs` that delegate directly to
  `self.delegate.fast_parse_on(...)`, skipping the closure/clone/`Token::new` entirely. Verified
  with closure-call-counting tests (`action_tests.rs`: `map_fast_parse_on_skips_the_mapping_
  function`, `constant_fast_parse_on_skips_cloning_the_replacement_value`,
  `token_fast_parse_on_skips_building_the_token_and_inner_side_effects` — the last one chains
  `digit().map(...).token()` to prove the skip composes through both layers).
  **`OnlyIfParser`/`FlatMapParser` deliberately left unchanged** — confirmed dart's `WhereParser`
  (= our `only_if`) *also* doesn't override `fastParseOn`, because it can't: the predicate's
  result determines success/failure, so the value must be computed to know the outcome.
  `FlatMapParser` has no dart equivalent, but the same reasoning applies even more directly — the
  second parser is *chosen by* the first value, so there's no position to compute without it.
- **Deliberately NOT ported:** dart's `cast` / `castList` parsers. Rust has no easy runtime
  cast, and the idiomatic equivalent is for callers to `impl From<T>` and use `.map(Into::into)`
  / `.into()`. Don't add these even for test parity.
- **Trailing-closure fusion for `choice!`** — `seq!` got this (see "What's Implemented"); `choice!`
  deliberately didn't, and still hasn't. `choiceN` already produces a single `T` (not a tuple), so
  there's no destructuring-tuple-in-a-closure problem for it to solve the way `seq!` had — if
  `choice!` ever gets a fused form, the motivating case would have to be something else entirely,
  not a port of `seq!`'s reasoning.

## `#[grammar]` Proc Macro (implemented)

### Usage
```rust
#[grammar]
mod expr_grammar {
    pub fn start() -> impl Parser<f64> { add_expr().end() }
    fn atom() -> impl Parser<f64> { ... }           // internal, no accessor
    fn mul_expr() -> impl Parser<f64> { ... }       // internal, no accessor
    pub fn add_expr() -> impl Parser<f64> { ... }   // exposed for testing
}

let g = ExprGrammar::new();
g.parse_on(&ctx);             // Parser<f64> impl delegates to start
g.add_expr().parse_on(&ctx);  // pub accessor → SettableParserRef<f64>
```

### What the macro generates
- Struct named after module in PascalCase (`expr_grammar` → `ExprGrammar`)
- All rules stored as `SettableParser<T>` fields (strong Rcs — keep parsers alive)
- `new()` method: declares all `SettableParser::undefined()`, calls `.set()` on each
- Inter-rule calls in bodies (e.g. `atom()`) rewritten to `atom.borrow()` (Weak ref)
- Public accessor methods for `pub fn` rules → return `SettableParserRef<T>`
- `Parser<T>` impl delegates to `self.start`
- `start()` return type determines the grammar's output type `T`

### Crate structure
- New crate: `rust-petitparser-macros` with `proc-macro = true`
- Dependencies: `syn` (features = ["full"]), `quote`, `proc-macro2`, `heck`
- Re-export `grammar` from main crate's `lib.rs`

### Key implementation steps
1. `parse_macro_input!(item as ItemMod)` — parse the module
2. `module.ident.to_string().to_upper_camel_case()` (heck) — struct name
3. Collect `Item::Fn` items; split on `Visibility::Public` vs `Visibility::Inherited`
4. Extract `T` from `-> impl Parser<T>`: drill through `ReturnType::Type → Type::ImplTrait → TraitBound → PathArguments::AngleBracketed`
5. `VisitMut` on each function body: replace zero-arg calls to known rule names with `rule_name.borrow()`
   - In `visit_expr_mut`: match `Expr::Call` with empty args + `Expr::Path` func → known name → `*expr = parse_quote!(#ident.borrow()); return;`
6. `quote!` to emit struct, `new()`, accessors, `Parser<T>` impl

### Testing during development
- `cargo install cargo-expand` then `cargo expand --test example-grammars` to see generated code

### Gotchas
- **No left-recursion handling.** Like dart-petitparser, the macro builds a PEG-style recursive
  descent grammar with no packrat/left-recursion support. A rule that re-enters itself without
  consuming input first (e.g. `array → json_value → array`) recurses forever → stack overflow.
  Make recursive rules consume a leading token before recursing — e.g. wrap a repetition with
  `.skip(open, close)` around the *whole* `value (sep value)*`, NOT around the separator. The
  original JSON stack overflow was exactly this: `.skip(...)` had been attached to the separator,
  so `array` started by parsing `json_value()` directly. A larger `RUST_MIN_STACK` won't help —
  the recursion is infinite, not merely deep.
