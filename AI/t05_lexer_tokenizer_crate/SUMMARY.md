# T05 — Implement lexer/tokenizer crate

## Task goal
Implement the EisenScript lexer/tokenizer in Rust with legacy-compatible token kinds, operator normalization, comment stripping, and numeric parsing.

## Approach
- Read the legacy `Tokenizer.cpp` and fixture notes for Ball/Menger.
- Replaced the lexer stub with a token model that mirrors the legacy symbol representation.
- Implemented comment stripping for `//`, `/* */`, and line-start `#` directives.
- Preserved lowercasing of user strings and normalization of abbreviated operators like `w`, `md`, `a`, `b`, `c`, and `h`.
- Added support for fraction literals like `1/3` and square-bracket payloads as single tokens.
- Added fixture-backed tests for Ball, Menger, comments, and bracketed vectors.

## Result
- `lex()` now returns a full token stream plus diagnostics.
- Legacy token categories are available for the parser task.
- The lexer behavior is covered by focused unit tests.

## Status
Complete.
