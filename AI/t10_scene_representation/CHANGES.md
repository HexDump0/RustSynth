# T10 — Changes

## Files changed

### `RustSynth/crates/rustsynth_scene/src/scene.rs`
- Added `raw_settings: Vec<(String, String)>` field to `Scene`.
  Stores pass-through `set` keys (e.g. raytracer template parameters) in source order.

### `RustSynth/crates/rustsynth_scene/src/camera.rs`
- Added `impl Default for CameraState`:
  - `rotation`: 16-element identity matrix array (`[1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1]`).
  - `translation`: `[0.0, 0.0, 0.0]`.
  - `pivot`: `[0.0, 0.0, 0.0]`.
  - `scale`: `1.0`.

### `RustSynth/crates/rustsynth_scene/src/primitive.rs`
- Changed `Triangle` from a unit variant to `Triangle(String)`.
  The `String` carries the raw bracket payload from `triangle[...]` references.

### `AI/MASTER_TODO.md`
- Marked T10 `DONE`.
- Changed T10A from `BLOCKED` to `READY`.
- Changed T11, T12 from `BLOCKED` to `READY`.
- Changed T13 from `BLOCKED` to `READY`.

## Tests run
- `cargo test --workspace` — 51 passed, 0 failed
