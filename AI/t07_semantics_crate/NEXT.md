# T07 — Next

## Recommended next task
T08 is already `DONE`. See T08/T09/T10 task folders.

## Notes for the next agent

### Resolution design decisions
- Same-name rules with different weights are folded into a single `Ambiguous` node whose
  `variants` list is ordered by insertion (declaration order in the source file).
- Class tags (`name::class`) are stored in the `Primitive` node's `class_tag: Option<String>`
  field and passed through to the scene object's `tag` field — they are not validated against
  any renderer vocabulary.
- `triangle[...]` is treated as a primitive regardless of bracket content. The full bracket
  string is stored as the `kind_name` and also as the `Triangle(String)` payload in the scene
  `PrimitiveKind`.
- The resolver does **not** inline or expand ambiguous rules — it preserves the choice for
  the evaluator's weighted random selection.

### Known gaps
- Cyclic rule detection is not implemented (infinite-recursion risk mitigated by
  `max_generations` / `max_objects` caps in the evaluator).
- `set recurse` inside a rule body is parsed but currently a no-op in the evaluator.
