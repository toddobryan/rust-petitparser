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
    character/character.rs  - CharParser, PredicateCharParser, char(), letter(), etc.
    combinator/
      choice.rs   - Choice2-9 via choice_impl! macro
      sequence.rs - Seq2-9 via impl_seq! macro
    action/
      map.rs      - MapParser<T, P, F> (needs PhantomData<T>)
      token.rs    - TokenParser<P>
    repeater/
      possessive.rs - StarParser, PlusParser, OptParser
    ext.rs        - ParserExt<T> extension trait (map, star, plus, opt, token, all_matches)
    misc/
      newline.rs
  matcher/
    matches.rs    - MatchesIterator<T,P>, MatchesIterable<T,P> (IntoIterator)
```

## Key Design Decisions
- `Parser<T>` is generic over `T` (not an associated type). This means impls where `T` only appears in the `where` clause (not the Self type or trait) require `PhantomData<T>` in the struct — see `MapParser`.
- `ParserExt<T>: Parser<T> + Sized` extension trait provides method syntax. Blanket impl covers all parsers.
- `choice_impl!` macro uses `Option<Failure>` accumulation pattern (avoids needing to separate "first" from "rest").
- `impl_seq!` macro uses `?` operator with sequential context threading.
- `MatchesIterable` implements `IntoIterator` → `MatchesIterator`; supports `overlapping` flag.
- Repetition parsers (`star`, `plus`) include an infinite-loop guard: if the inner parser succeeds without advancing position, break.

## Testing
- Uses `googletest` crate: `#[gtest]`, `assert_that!`, `eq`, `not`
- Test file: `tests/parsing_tests.rs`

## What's Implemented
- Character parsers (`CharParser`, `PredicateCharParser`, `char`, `letter`, `digit`, `one_of`, etc.)
- `Seq2`–`Seq9`, `Choice2`–`Choice9`
- `map`, `star`, `plus`, `opt`, `token`
- Repetition unified as `PossessiveRepeatingParser { min, max }` exposed via `rep(min, max)`; `star/plus/opt` delegate to it. Inner parser **must consume input** (runtime assert), so `char('x').star().opt()` on `"y"` panics — that's documented by `star_with_opt_that_doesnt_consume_should_panic`.
- `all_matches` (via `MatchesIterable`/`MatchesIterator`)
- `line_and_column_of` (was a TODO stub, now implemented)
- `SettableParser<T>` in `parser/combinator/settable.rs` — placeholder + late binding for recursive grammars. `undefined()` constructs; `.set(p)` installs delegate; `.clone()` shares the cell via `Rc<RefCell<…>>`.

## SettableParser — open items / known issues
1. **`set` should probably take `&self`, not `&mut self`.** The `Rc<RefCell<…>>` is already interior mutability — taking `&mut self` blocks `.set()` through a clone and forces `let mut` on the binding, neither of which buys safety. Single-char change once you're ready.
2. **Reference cycle leak.** After `expr.set(choice2(inner, leaf))` where `inner` contains `expr.clone()`, the Rc strong count never reaches zero. Acceptable for now; document near the struct or revisit with `Weak`-based clones if it becomes a problem.
3. **Unused `PhantomData` import** in `settable.rs` — clippy nit.
4. **Optional ergonomics:** add a top-level `pub fn undefined<T>() -> SettableParser<T>` so users can write `let expr = undefined::<i32>();` instead of `SettableParser::<i32>::undefined()`.

## What's Next
- String literal parser (match an exact string)
- End-of-input parser (`.end()`) — needed so recursive grammars don't accept prefixes of their input
- A real recursive test grammar (arithmetic with precedence, or JSON) to exercise mutual recursion and shake out remaining surprises
- Decide on / port `GrammarDefinition` + `ref()` sugar — *probably defer* until a few hand-written grammars show whether the boilerplate hurts
