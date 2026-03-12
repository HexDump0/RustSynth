# T08 — Next

## Recommended next task
T09 and T10 are both `DONE`. The next ready tasks are **T10A**, **T11**, **T12**, **T13**.

## Notes for the next agent

### BFS vs DFS semantics
- BFS is the legacy default. The generation counter ticks once per full pass through the
  work queue, matching `Builder.cpp`'s `iteration` variable.
- DFS prepends new items to the front of the deque — children are processed before siblings.
- In DFS mode, `set maxdepth N` applies as a global per-rule depth cap via `set_rules_max_depth`.

### Per-rule max-depth budget
- First call to a rule with `max_depth M` stores `M - 1` in `state.max_depths`.
- Each re-entry decrements by 1. When it reaches 0 the retirement rule (if any) is enqueued
  with the budget reset, then the body is skipped.
- This matches the legacy `CustomRule.cpp` `maxDepth` check exactly.

### sync_random
- When `sync_random` is true, all branches at the same BFS generation share the same RNG seed
  — a fresh seed is drawn once per generation and re-applied before each branch. This reproduces
  the legacy `syncRandom` symmetry feature.

### Known gaps
- `set` commands inside rule bodies are currently no-ops. The legacy app allows some dynamic
  set usage (e.g. changing the background mid-execution), but this is rare and deferred.
- The `seed` field on `State` (for per-branch determinism) is scaffolded but not yet wired to
  the BFS re-seeding path for every branch; it is only used when a branch explicitly carries
  a non-zero seed.
