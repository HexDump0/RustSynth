# T08 — Implement evaluator/builder core

## Task goal
Implement the full BFS/DFS evaluation engine in `rustsynth_eval` that mirrors the legacy
`Builder.cpp`: expand a `RuleGraph` rule-by-rule, manage recursion budgets, apply transforms
and color state to each branch, and emit primitive objects into a `Scene`.

## Approach
- Read `Builder.cpp`, `Action.cpp`, `AmbiguousRule.cpp`, `CustomRule.cpp`, `State.cpp` in the
  legacy source to confirm BFS generation-counter semantics, per-rule max-depth budget tracking,
  the cartesian-product loop expansion for `N * { ... }`, and the weighted random selection.
- Implemented `BuildConfig` with all knobs the legacy `set` commands expose.
- BFS uses a generation counter and swaps the work queue each generation. DFS prepends
  new items to the front of the deque.
- Per-rule depth is stored as `HashMap<String, i32>` inside `State`. First visit initialises
  the budget to `max_depth - 1`; each re-entry decrements; exhaustion triggers retirement rule.
- `push_action_states` does the cartesian product of loop counters, applying transforms
  cumulatively for each `(counter_0, counter_1, ...)` combination — matching `Action.cpp`.
- Camera `set` commands are parsed from the `[x y z]` / `[r0c0 ... r2c2]` bracket formats.

## Result
- `builder.rs` — `BuildConfig`, `build()`, full BFS/DFS loops, `apply_rule`, `apply_custom`,
  `pick_variant`, `push_action_states`, `apply_set_command`, `emit_object`, `should_prune`.
- 7 builder tests: simple box, bare target, multiply syntax, set background, maxdepth limit,
  ambiguous variant selection, Menger fixture object count.
- Full test suite: 51 tests, all passing.

## Status
Complete.
