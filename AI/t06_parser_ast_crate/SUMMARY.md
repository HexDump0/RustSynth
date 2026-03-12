# T06 — Implement parser AST crate

## Task goal
Replace the stub `ast.rs` and `parser.rs` in `rustsynth_eisenscript` with a complete
EisenScript AST and recursive-descent parser, covering all syntax forms present in the
legacy fixtures.

## Approach
- Read the legacy `EisenParser.cpp` to understand operator keywords, `set` commands,
  `rule name w N maxdepth M > retirement` syntax, and the `N * { ... } target` multiply form.
- Designed a compact, serde-friendly AST: `Script → Statement* → RuleDef | Action | SetCmd`.
- Implemented a single-pass recursive-descent parser that runs the lexer first and consumes
  the token stream.
- `collect_target()` merges `name` + `[payload]` into a single string for `triangle[...]` refs.
- Fraction literals from the lexer (e.g. `1/3`) are stored as `f64` values.
- Parser produces a `ParseResult { script, diagnostics }` so callers can see errors without
  panicking.

## Result
- Full `ast.rs` with `Script`, `Statement`, `RuleDef`, `BodyItem`, `Action`, `TransformLoop`,
  `SetCmd`, `TransformOp` (covers all legacy operators).
- Full `parser.rs` with `parse(source: &str) -> ParseResult`.
- 14 parser tests covering Ball, Menger, Default fixtures and edge cases.
- 22 tests total in `rustsynth_eisenscript` — all passing.

## Status
Complete.
