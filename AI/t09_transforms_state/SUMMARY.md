# T09 — Implement transformations and state parity tests

## Task goal
Implement the full set of EisenScript transform operators in `rustsynth_eval`, with tests
that verify parity with the legacy `Transformation.cpp` behaviour.

## Approach
- Read `Transformation.cpp` carefully for:
  - The pivot convention: all rotations and scales pivot about the unit-cube centre `(0.5, 0.5, 0.5)`.
  - HSV colour semantics: `hue` is additive (mod 360), `sat`/`brightness`/`alpha` are multiplicative
    and clamped to `[0, 1]`, absolute `color` replaces all HSV fields, `blend` mixes in HSV space
    using the formula `(current + strength * blend) / (1 + strength)`.
  - Plane reflections via `Fx/Fy/Fz` (unit-axis) and `ReflectNx/Ny/Nz` (arbitrary normal).
  - `Matrix([f64;9])` inserts a 3×3 sub-matrix (upper-left) into the current 4×4 transform.
- Implemented all operators in `apply_one`, composed via `apply_transforms`.
- Fixed the `Rng::new` bug (xorshift64 fixed-point at 0) by applying a splitmix64 mixing step
  to the seed — this ensures small seeds (0–19) produce well-distributed RNG outputs.

## Result
- `transform.rs` — `apply_transforms(ops, state, rng) -> State`, `apply_one(op, state, rng) -> State`.
- `state.rs` — `State { transform: Mat4, color: Hsva, max_depths: HashMap<String, i32>, seed: u64 }`.
- 13 transform tests: translations, uniform scale (centre-fixed), flip-x, rz 90°, rz 360° identity,
  hue wrap, sat clamp, brightness halve, alpha reduce, hex colour, random colour (consecutive calls),
  identity matrix.
- `rustsynth_core/src/rng.rs` — `Rng::new` now hashes seed through splitmix64.

## Status
Complete.
