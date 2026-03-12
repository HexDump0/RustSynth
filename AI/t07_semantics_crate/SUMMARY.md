# T07 — Implement semantic resolution layer

## Task goal
Build the name-resolution and validation layer that transforms a raw `Script` AST into a
`RuleGraph` — a fully resolved, rule-indexed representation ready for evaluation.

## Approach
- Studied `Builder.cpp` and `RuleSet.cpp` to understand how legacy code merges weighted
  rule variants, resolves primitives, and handles `triangle[...]` mesh payloads.
- Designed a three-variant `RuleNode` enum: `Custom`, `Ambiguous` (weighted list), `Primitive`.
- Two-pass resolution:
  1. Insert all user-defined `RuleDef`s — same-name rules accumulate in `Ambiguous`.
  2. Scan all action targets: insert `Primitive` nodes for built-ins, `triangle[...]` refs,
     and `name::class` class-tagged primitive references; emit diagnostic for `::class` on
     non-primitives.
- Separate `validate()` pass checks that every action target and retirement name resolves
  to a known rule.

## Result
- `rule_graph.rs` — `RuleGraph`, `RuleNode`, `CustomRuleNode`, `AmbiguousRuleNode`, `StartItem`.
- `resolution.rs` — `resolve(script) -> (RuleGraph, Vec<Diagnostic>)`.
- `validation.rs` — `validate(graph) -> Vec<Diagnostic>`.
- `primitive.rs` — `is_builtin()`, `is_triangle_ref()` helpers; `"triangle"` added to `BUILTINS`.
- `lib.rs` — full public API re-exported.

## Status
Complete.
