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
    predicate/
      predicate.rs  - PredicateParser, string(&str), string_ignore_case(&str)
    ext.rs        - ParserExt<T> extension trait: map, flat_map, rep, times, star, plus, opt,
                    token, trim, input, input_with_message, only_if, only_if_with_message,
                    only_if_with_factory, all_matches, and, not, labeled, skip,
                    rep_sep, star_sep, plus_sep
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
- `SettableParser` uses `Rc<RefCell<Option<Rc<dyn Parser<T>>>>>` — allows setting after cloning for recursive grammars. Can leak via Rc cycle.

## Testing
- Uses `googletest` crate: `#[gtest]`, `assert_that!`, `eq`, `not`
- Tests split across multiple files: `character_tests.rs`, `combinator_tests.rs`, `repeater_tests.rs`,
  `action_tests.rs`, `matcher_tests.rs`, `misc_tests.rs`
- Example grammars: `tests/example-grammars/main.rs` (entry point) + `tests/example-grammars/json.rs`
- ~135+ tests passing

## What's Implemented
- Character parsers: `any`, `char`, `char_ci`, `letter`, `digit`, `digit_with_radix`, `one_of`, `one_of_ci`,
  `none_of`, `none_of_ci`, `lowercase`, `uppercase`, `whitespace`, `word`, `predicate`
- `Seq2`–`Seq9`, `Choice2`–`Choice9` (with configurable failure joiner: `SELECT_FARTHEST_JOINED`)
- `map`, `flat_map`, `rep`, `times`, `star`, `plus`, `opt`, `token`, `trim`, `input`, `all_matches`
- `only_if`, `only_if_with_message`, `only_if_with_factory`
- `and()`, `not()` lookaheads
- `rep_sep`, `star_sep`, `plus_sep` (separated repeaters)
- `skip(open, close)` — wraps parser between delimiters, returns inner value
- `labeled(label)` — replaces failure message
- `string(&str)`, `string_ignore_case(&str)` via `PredicateParser`
- `eof()`, `eof_with_message()` via `EndOfInputParser`
- `epsilon()`, `epsilon_with(T)`, `success(T)`, `failure()`, `failure_with_message(String)`
- `position()` — returns current position as `usize` without consuming
- `SettableParser<T>` for recursive grammars
- `line_and_column_of`
- JSON example grammar (full, with recursive `SettableParser`)

## What's Next
- Lazy repeaters (stop-at-first vs possessive)
- `flat_map` / `and_then` — done this session
- Arithmetic expression example grammar (precedence climbing)
- `GrammarDefinition` — probably defer
