# rust-petitparser

A Rust port of PetitParser (originally Dart/Java).

## User Preferences
- Provide hints and structural guidance, not full code — let the user implement with guidance
- Give type signatures, key gotchas, and design rationale

## Module Structure
```
src/
  core/
    context.rs    - Context, Success, Failure, ParseResult
    parser.rs     - Parser trait
    result.rs     - ParseResult type alias
    token.rs      - Token<T> struct
    error.rs
  parser/
    character/character.rs  - CharParser, PredicateCharParser, any(), char(), letter(), digit(), one_of(), etc.
    combinator/
      choice.rs     - Choice2-9 via choice_impl! macro
      sequence.rs   - Seq2-9 via impl_seq! macro
      settable.rs   - SettableParser<T> for recursive grammars
      lookahead.rs  - AndParser<T,P>, NotParser<T,P>
    action/
      map.rs      - MapParser<T, P, F> (needs PhantomData<T>)
      token.rs    - TokenParser<P>
      input.rs    - InputParser (flatten matched chars to String)
      only_if.rs  - OnlyIfParser (predicate gate on success value)
      flat_map.rs - FlatMapParser (monadic bind: value → next parser)
    repeater/
      possessive.rs  - PossessiveRepeatingParser { min, max }
      separated.rs   - SeparatedRepeatingParser (rep_sep/star_sep/plus_sep)
      lazy.rs        - LazyRepeatingParser<P, T, PC, TC> { delegate, limit, min, max }
    predicate/
      predicate.rs  - PredicateParser, string(&str), string_ignore_case(&str)
    ext.rs        - ParserExt<T> extension trait: map, flat_map, rep, times, star, plus, opt,
                    token, trim, input, input_with_message, only_if, only_if_with_message,
                    only_if_with_factory, all_matches, and, not, labeled, skip,
                    skip_left, skip_right, end,
                    rep_sep, star_sep, plus_sep,
                    rep_lazy, star_lazy, plus_lazy
    combinator/
      skip.rs     - SkipParser (open, content, close → content)
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
```

## Key Design Decisions
- `Parser<T>` is generic over `T` (not an associated type). This means impls where `T` only appears in the `where` clause (not the Self type or trait) require `PhantomData<T>` in the struct — see `MapParser`.
- `ParserExt<T>: Parser<T> + Sized where T: Debug` extension trait provides method syntax. Blanket impl covers all parsers.
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
- Example grammars: `tests/example-grammars/main.rs` (entry point) + `tests/example-grammars/json.rs` + `tests/example-grammars/expr.rs`
- 159 tests passing

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

## What's Next
- Port remaining dart-petitparser tests (repeat_lazy success cases, etc.)
- `#[grammar]` proc macro (replaces manual SettableParser boilerplate)

## `#[grammar]` Proc Macro Design

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
