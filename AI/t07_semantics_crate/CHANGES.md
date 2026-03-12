# T07 — Changes

## Files changed

### `RustSynth/crates/rustsynth_semantics/src/rule_graph.rs`
Replaced stub. Key types:
- `RuleGraph { rules: HashMap<String, RuleNode>, start_items: Vec<StartItem>, recurse_depth: bool }`
- `RuleGraph::get(name) -> Option<&RuleNode>`
- `StartItem::Action(Action) | SetCmd(SetCmd)` — top-level program items
- `RuleNode::Custom(CustomRuleNode) | Ambiguous(AmbiguousRuleNode) | Primitive { kind_name, class_tag }`
- `CustomRuleNode { name, weight, max_depth, retirement, body }` + `from_rule_def()` constructor
- `AmbiguousRuleNode { name, variants: Vec<CustomRuleNode> }` + `total_weight() -> f64`

### `RustSynth/crates/rustsynth_semantics/src/resolution.rs`
Replaced stub. Two-pass resolver:
- Pass 1: collects user `RuleDef`s; primitive-shadowing produces an error diagnostic.
- Pass 2: `collect_all_targets()` gathers every action target in user rules and top-level
  actions; inserts `Primitive` nodes for built-ins, `triangle[...]` refs, and `name::class`
  class-tagged references; emits error for `::class` on non-primitive user rules.

### `RustSynth/crates/rustsynth_semantics/src/validation.rs`
Replaced stub. `validate(graph) -> Vec<Diagnostic>` checks all action targets and
retirement names are present in `graph.rules`.

### `RustSynth/crates/rustsynth_semantics/src/primitive.rs`
- Added `"triangle"` to `BUILTINS` slice.
- Added `is_triangle_ref(name: &str) -> bool` — checks `name.starts_with("triangle[")`.

### `RustSynth/crates/rustsynth_semantics/src/lib.rs`
Added full public re-exports:
`RuleGraph`, `RuleNode`, `CustomRuleNode`, `AmbiguousRuleNode`, `StartItem`,
`resolve`, `validate`.

### `AI/MASTER_TODO.md`
Marked T07 `DONE`.

## Tests run
- `cargo test --workspace` — 51 passed, 0 failed
