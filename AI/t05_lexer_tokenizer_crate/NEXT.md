# T05 — Next

## Recommended next task
- T06 — implement the parser AST crate.

## Known blockers
- None.

## Notes for the next agent
- The lexer preserves the legacy token shape (`text`, numeric fields, `is_integer`, position, and token kind) to make the parser port straightforward.
- Square-bracket payloads such as `[1 0 0 ...]` remain a single `UserString` token, matching the legacy tokenizer behavior.
- Fraction literals like `1/3` are already reduced to numeric tokens, which should simplify parser support for examples like `Menger.es`.
