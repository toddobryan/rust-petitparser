# rust-petitparser

A Rust port of PetitParser (originally Dart/Java).

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
      to.rs       - ToParser<T,P,V> (.to(value) — replace matched value with a clone, V: Clone)
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
- Blanket `impl<T, P: Parser<T> + ?Sized> Parser<T> for Rc<P>` (in `core/parser.rs`) — lets `Rc<P>` and `Rc<dyn Parser<T>>` be used as parsers (delegates `parse_on`/`fast_parse_on`). Needed to share a sub-parser across multiple combinators via `Rc::new(..).clone()` (our parsers aren't generally `Clone`). `Rc<P>` is `Sized`, so it auto-gets `ParserExt` too.
- `choice_impl!` macro uses `Option<Failure>` accumulation pattern (avoids needing to separate "first" from "rest").
- `impl_seq!` macro uses `?` operator with sequential context threading.
- `MatchesIterable` implements `IntoIterator` → `MatchesIterator`; supports `overlapping` flag.
- Repetition parsers (`star`, `plus`) include an infinite-loop guard: if the inner parser succeeds without advancing position, break.
- `PredicateParser.length` is in **chars** (`.chars().count()`), not bytes — required for Unicode/emoji correctness.
- `AndParser` returns `Parser<T>` — preserves the matched value but resets position to original (lookahead).
- `NotParser` returns `Parser<Failure>` — inner failure becomes the success value. Error message: `"Expected failure, got success: {:?}"` on the value.
- `SettableParser` uses `Rc<RefCell<Option<Rc<dyn Parser<T>>>>>` — the "owner" (strong Rc).
- `SettableParserRef` uses `Weak<RefCell<...>>` — the "embedded reference" that breaks Rc cycles.
- Call `.borrow()` on `SettableParser` to get a `SettableParserRef` for embedding in sub-parsers.
- Rule: use `.clone()` for forward references (more complex → simpler); use `.borrow()` only for back-references (the one that creates the cycle). The strong chain from the returned root parser keeps all intermediate parsers alive.

## Testing
- Uses `googletest` crate: `#[gtest]`, `assert_that!`, `eq`, `not`
- Tests split across multiple files: `character_tests.rs`, `combinator_tests.rs`, `repeater_tests.rs`,
  `action_tests.rs`, `matcher_tests.rs`, `misc_tests.rs`
- Each test file defines `assert_success!(parser, input, value, pos)` and `assert_failure!(parser, input, message, pos)`
  macros that check both `parse_on` and `fast_parse_on`
- Example grammars: `tests/example-grammars/main.rs` (entry point) +
  `json.rs` + `expr.rs` — both now written with the `#[grammar]` proc macro
- 137 tests passing

## What's Implemented
- Character parsers: `any`, `char`, `char_ci`, `letter`, `digit`, `digit_with_radix`, `one_of`, `one_of_ci`,
  `none_of`, `none_of_ci`, `lowercase`, `uppercase`, `whitespace`, `word`, `predicate`
- `Seq2`–`Seq9`, `Choice2`–`Choice9` (with configurable failure joiner: `SELECT_FARTHEST_JOINED`)
- `map`, `flat_map`, `rep`, `times`, `star`, `plus`, `opt`, `token`, `trim`, `input`, `all_matches`
- `only_if`, `only_if_with_message`, `only_if_with_factory`
- `and()`, `not()` lookaheads
- `rep_sep`, `star_sep`, `plus_sep` (separated repeaters)
- `rep_lazy`, `star_lazy`, `plus_lazy` (lazy repeaters with limit parser)
- `skip(open, close)` — wraps parser between delimiters, returns inner value
- `skip_left(before)`, `skip_right(after)`, `end()` — variants of skip
- `to(value)` — replaces the matched value with a clone of `value` (`V: Clone + Debug`)
- `labeled(label)` — replaces failure message
- `string(&str)`, `string_ignore_case(&str)` via `PredicateParser`
- `eof()`, `eof_with_message()` via `EndOfInputParser`
- `epsilon()`, `epsilon_with(T)`, `success(T)`, `failure()`, `failure_with_message(String)`
- `position()` — returns current position as `usize` without consuming
- `SettableParser<T>` / `SettableParserRef<T>` for recursive grammars (cycle-free)
- `line_and_column_of`
- JSON example grammar (full, with recursive `SettableParser`)
- Arithmetic expression grammar (`tests/example-grammars/expr.rs`)
  - Integer arithmetic: `+`, `-`, `*`, `/` with correct precedence and left-associativity
  - Parenthesized subexpressions via recursive `SettableParser`
  - `fold_ops` pattern: `fn(&(f64, Vec<(char, f64)>)) -> f64` with left fold
  - Note: `SkipParser` does NOT need `T: Clone` — removed that bound
- `#[grammar]` proc macro (`rust-petitparser-macros`) — replaces manual SettableParser
  boilerplate; drives both the expr and JSON example grammars

## What's Next
- Port remaining dart-petitparser tests toward parity (scope A = features we already have).
  Done so far: character/predicate gap-fills, `string_ignore_case`, `context_tests.rs`.
  Remaining portable-now: newline + `position()` tests; combinator (choice joiner matrices,
  `seq3` failure positions, settable wrap+message, skip none/before/after); finish the
  commented-out `lazy > repeat` block in `repeater_tests.rs`; possessive `times`/`repeat`/
  unbounded/infinite-loop; richer action `input`/`map`/`token`; representative sequence subset.
  Deferred (needs new features, scope B/C): greedy repeaters, `SeparatedList` typed results,
  string repeaters, `opt_with`, graceful `undefined()` failure,
  `permute`/`pick`/`continuation`/`join`, custom-delimiter `trim`,
  `pattern`/`range`/regex/unicode char parsers, reflection/introspection (`children`, `copy`,
  deep-equality — needed for dart's `expectParserInvariants` assertions).
  (`not_with_message`/`neg`/`neg_with_message` are now implemented.)
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
