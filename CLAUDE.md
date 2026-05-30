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
    repeater/
      possessive.rs - PossessiveRepeatingParser { min, max }
    predicate/
      predicate.rs  - PredicateParser, string(&str), string_ignore_case(&str)
    ext.rs        - ParserExt<T> extension trait: map, rep, star, plus, opt, token, trim, all_matches, and, not
    misc/
      newline.rs
      end.rs      - EndOfInputParser, eof(), eof_with_message()
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
- `SettableParser` uses `Rc<RefCell<Option<Rc<dyn Parser<T>>>>>` — allows setting after cloning for recursive grammars. Can leak via Rc cycle.

## Testing
- Uses `googletest` crate: `#[gtest]`, `assert_that!`, `eq`, `not`
- Test file: `tests/parsing_tests.rs`
- 50 tests passing

## What's Implemented
- Character parsers: `any`, `char`, `letter`, `digit`, `one_of`, `none_of`, `lowercase`, `uppercase`, `whitespace`, `word`, `predicate`
- `Seq2`–`Seq9`, `Choice2`–`Choice9`
- `map`, `rep`, `star`, `plus`, `opt`, `token`, `trim`, `all_matches`
- `and()`, `not()` lookaheads
- `string(&str)`, `string_ignore_case(&str)` via `PredicateParser`
- `eof()`, `eof_with_message()` via `EndOfInputParser`
- `SettableParser<T>` for recursive grammars
- `line_and_column_of`

## What's Next
- `separated_by(sep)` — parse `p (sep p)*`, returns `Vec<T>`
- `flatten` — collapse `Vec<char>` or nested result to `String`
- A real recursive grammar (arithmetic with precedence, or JSON) to exercise mutual recursion
- Decide on / port `GrammarDefinition` + `ref()` sugar — probably defer until hand-written grammars show whether boilerplate hurts
