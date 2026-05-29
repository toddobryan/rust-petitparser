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
- `all_matches` (via `MatchesIterable`/`MatchesIterator`)
- `line_and_column_of` (was a TODO stub, now implemented)

## What's Next
- String literal parser (match an exact string)
- End-of-input parser
- Recursive/lazy parsers (for recursive grammars)
