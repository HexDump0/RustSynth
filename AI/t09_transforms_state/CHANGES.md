# T09 — Changes

## Files changed

### `RustSynth/crates/rustsynth_eval/src/transform.rs`
Replaced stub. Full transform application:
- `apply_transforms(ops, state, rng) -> State` — folds a slice of ops.
- `apply_one(op, state, rng) -> State` — handles every `TransformOp` variant.
- Helpers: `rotate_about_pivot(pivot, axis, deg)`, `scale_about_center(sx, sy, sz)`,
  `scale_about_center_matrix(m4)`, `plane_reflection(normal)`, `rgba_to_hsva(rgba)`,
  `named_color(name)` (basic SVG colour names → `Rgba`).
- Colour logic:
  - `Hue(dh)` — additive, wraps mod 360.
  - `Sat(f)` / `Brightness(f)` / `Alpha(f)` — multiplicative, clamped to `[0, 1]`.
  - `Color("random")` — random hue via `rng.next_f64() * 360.0`, full s/v/a.
  - `Color(hex)` / `Color(name)` — absolute replacement of HSV state.
  - `Blend { color, strength }` — `(current + strength * blend) / (1 + strength)` in HSV space.

### `RustSynth/crates/rustsynth_eval/src/state.rs`
Replaced stub:
- `State { transform: Mat4, color: Hsva, max_depths: HashMap<String, i32>, seed: u64 }`
- `Default`: identity transform, `Hsva(0, 1, 1, 1)` (legacy default colour).

### `RustSynth/crates/rustsynth_core/src/rng.rs`
- `Rng::new(seed)` now applies one round of splitmix64 before using the value as the
  xorshift64 state. This fixes the fixed-point at 0 (seed 0 previously always returned 0)
  and ensures small seeds produce well-distributed first outputs.

### `RustSynth/crates/rustsynth_core/src/math.rs`
Added `Mat3` to re-exports: `pub use glam::{Mat3, Mat4, Quat, Vec3, Vec4}`.

### `AI/MASTER_TODO.md`
Marked T09 `DONE`.

## Tests run
- `cargo test -p rustsynth_eval` — 21 passed
- `cargo test --workspace` — 51 passed, 0 failed
