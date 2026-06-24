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
`use rust_petitparser::prelude::*;` — the prelude re-exports the `Parser` trait, `ParserExt`,
`Context`/`Success`/`Failure`/`ParseResult`, `Token`, all parser constructors, the
`assert_success!`/`assert_failure!` macros, and `grammar`.

## Module Structure
```
src/
  prelude.rs    - public re-export surface (see Workspace Layout)
  core/
    context.rs    - Context, Success, Failure, ParseResult
    parser.rs     - Parser trait
    result.rs     - ParseResult type alias
    token.rs      - Token<T>, line_and_column_of, position_string
    test_helpers.rs
  parser/
    character.rs  - CharParser, PredicateCharParser, any(), char(), letter(), digit(), one_of(), etc.
    combinator/
      choice.rs     - Choice2-9 via choice_impl! macro
      sequence.rs   - Seq2-9 via impl_seq! macro
      settable.rs   - SettableParser<T> / SettableParserRef<T> for recursive grammars
      lookahead.rs  - AndParser<T,P>, NotParser<T,P>
      skip.rs       - SkipParser (open, content, close → content)
    action/
      map.rs      - MapParser<T, P, F> (needs PhantomData<T>)
      token.rs    - TokenParser<P>
      input.rs    - InputParser (flatten matched chars to String)
      only_if.rs  - OnlyIfParser (predicate gate on success value)
      flat_map.rs - FlatMapParser (monadic bind: value → next parser)
      constant.rs - ConstantParser<T,P,V> (.constant(value) — replace matched value with a clone, V: Clone)
    repeater/
      possessive.rs  - PossessiveRepeatingParser { min, max }
      separated.rs   - SeparatedRepeatingParser (rep_sep/star_sep/plus_sep)
      lazy.rs        - LazyRepeatingParser<P, T, PC, TC> { delegate, limit, min, max }
    predicate.rs  - PredicateParser, string(&str), string_ignore_case(&str)
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
```

## Key Design Decisions
- `Parser<T>` is generic over `T` (not an associated type). This means impls where `T` only appears in the `where` clause (not the Self type or trait) require `PhantomData<T>` in the struct — see `MapParser`.
- `ParserExt<T>: Parser<T> + Sized where T: Debug` extension trait provides method syntax. Blanket impl covers all parsers.
- `accept(input: &str)`/`accept_at(input: &str, start)` take `&str` and build the `Rc<[char]>` buffer
  internally, matching both dart's actual signature and this crate's own `Parser::parse(&str)`
  convenience wrapper — these are one-shot leaf calls, never invoked elsewhere in the codebase with a
  pre-existing buffer, so there's no reuse to preserve. `all_matches` deliberately stayed on
  `Rc<[char]>` instead of getting the same treatment: `core/token.rs`'s `line_and_column_of` calls it
  internally on a buffer it already holds (the original parse buffer, shared via `Rc`), and forcing
  `&str` there would mean collecting that buffer into a `String` just to re-split it back into an
  `Rc<[char]>` inside `all_matches` — a wasteful, lossy round trip purely to satisfy the signature.
  Same combinator, two different call patterns, two different right answers.
- Blanket `impl<T, P: Parser<T> + ?Sized> Parser<T> for Rc<P>` (in `core/parser.rs`) — lets `Rc<P>` and `Rc<dyn Parser<T>>` be used as parsers (delegates `parse_on`/`fast_parse_on`). Needed to share a sub-parser across multiple combinators via `Rc::new(..).clone()` (our parsers aren't generally `Clone`). `Rc<P>` is `Sized`, so it auto-gets `ParserExt` too.
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
- `.constant(value)` — `ConstantParser<T, P, V> where V: Clone + Debug`
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
  `action_tests.rs`, `matcher_tests.rs`, `misc_tests.rs`
- Each test file defines `assert_success!(parser, input, value, pos)` and `assert_failure!(parser, input, message, pos)`
  macros that check both `parse_on` and `fast_parse_on`
- Example grammars: `tests/example-grammars/main.rs` (entry point) +
  `json.rs` + `expr.rs` + `bibtex.rs` + `pascal.rs` (written with the `#[grammar]` proc macro —
  each has genuinely recursive rules) + `tabular.rs` + `uri.rs` (hand-written — no recursion, so
  no macro/`SettableParser` needed)
- 376 tests passing — includes `bibtex::scg_bib_size_and_round_trip`, which makes a real network
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
  need `.map(|_| ())` first). `GreedyRepeatingParser` (`src/parser/repeater/greedy.rs`). Ported from
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

## What's Next
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
  - **smalltalk**, **dart** (the grammar, not this project) — large grammars only (200–800+
    lines), no eval, otherwise straightforward composition (pascal, the smallest of the three,
    is now done — see above).
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
  Deferred (needs new features, scope B/C): `SeparatedList` typed results, string repeaters,
  regex char parsers, reflection/introspection
  (`children`, `copy`, deep-equality — needed for dart's `expectParserInvariants` assertions).
  (`not_with_message`/`neg`/`neg_with_message`/`pattern`/`pattern_ci`/custom-delimiter
  `trim_with`/greedy repeaters/graceful `undefined()` failure/`opt_with`/`continuation`
  (`call_cc`)/`range`/`accept`/`accept_at`/`elements_at` (dart's `permute`, with a `permute`
  alias)/`pick`/`Token::join` are now implemented. `Token::join` is a notable example of why
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
- **Variadic `seq!` / `choice!` macros** so callers don't hard-code the arity (no more `seq3`,
  `choice6`). Idea: a `macro_rules!` that counts its args and dispatches to the existing
  fixed-arity `Seq2..9` / `Choice2..9` (e.g. `seq!(a,b,c)` → `seq3(a,b,c)`). Keeps the current
  tuple-typed return (`seq3` → `(A,B,C)`); the macro is just sugar over `impl_seq!`/`choice_impl!`.
  Gotcha: still capped at arity 9 until more `SeqN`/`ChoiceN` are generated (or a nested fallback
  is added); decide what happens at 10+ args.

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
