# T08 — Changes

## Files changed

### `RustSynth/crates/rustsynth_eval/src/builder.rs`
Replaced stub. Full evaluation engine:
- `BuildConfig { max_generations, max_objects, min_dim, max_dim, sync_random, mode, seed }`
- `build(graph, cfg) -> Scene` — main entry point.
- `run_bfs` / `run_dfs` — expansion loops using a `VecDeque<(String, State)>`.
- `apply_rule` — dispatches on `RuleNode` variant.
- `apply_custom` — manages per-rule max-depth budget in `state.max_depths`, applies body actions.
- `pick_variant` — weighted random selection using seeded RNG.
- `push_action_states` — cartesian product of transform loops (mirrors `Action.cpp`).
- `apply_set_command` — handles all `set` keys: `maxdepth`, `maxobjects`, `minsize`, `maxsize`,
  `seed`, `syncrandom`, `recursion`, `background`, `translation`, `rotation`, `pivot`, `scale`.
  Unknown keys are stored in `scene.raw_settings`.
- `emit_object` — maps kind_name string to `PrimitiveKind`, pushes `SceneObject` to scene.
- `should_prune` — size-based culling using `transform_vector3(Vec3::ONE)` length.
- Camera parsing helpers: `parse_vec3_bracket`, `parse_mat3_as_mat4_bracket` (row-major input
  → column-major `Mat4` output matching the legacy `State.cpp` camera format).

### `RustSynth/crates/rustsynth_eval/src/lib.rs`
Added public re-exports: `build`, `BuildConfig`, `RecursionMode`, `State`.

### `RustSynth/crates/rustsynth_eval/Cargo.toml`
Added `glam = { workspace = true }` dependency.

### `AI/MASTER_TODO.md`
Marked T08 `DONE`.

## Tests run
- `cargo test -p rustsynth_eval` — 21 passed
- `cargo test --workspace` — 51 passed, 0 failed
