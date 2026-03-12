# T06 — Next

## Recommended next task
T07 is already `DONE` (resolved in the same session as T06).
The next ready tasks are: **T10A**, **T11**, **T12**, **T13** (see MASTER_TODO.md).

## Known blockers
None.

## Notes for the next agent

### AST design decisions
- `TransformOp` uses `f64` for all numeric parameters because the legacy code uses `double`.
- `RuleDef::weight` defaults to `1.0`; `max_depth: None` means no per-rule limit.
- A `retirement` name is an `Option<String>` — if exhausted and no retirement name is set,
  the branch is simply dropped.
- `triangle[payload]` is stored as a plain `String` target in `Action` — the brackets and
  content are preserved verbatim.

### Parser edge cases confirmed working
- `rule R maxdepth 3 > R2 { ... }` — maxdepth + retirement in one declaration.
- `3 * { rz 120 } box` — multiply syntax correctly creates one `TransformLoop` with
  `count=3` and a single `Rz(120.0)` operator.
- `rule R w 10 { ... }` — weight on custom rule.
- `box::shiny` — class tag preserved in target string.
- `blend #rrggbb 0.5` — parsed as `TransformOp::Blend { color, strength }`.
