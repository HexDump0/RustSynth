# T10A — Changes

## Files changed

### `RustSynth/crates/rustsynth_render_api/src/camera.rs`
- Replaced empty stub with full `ArcballCamera` implementation.
- Fields: `pivot: Vec3`, `yaw: f32`, `pitch: f32`, `distance: f32`, `fov_y: f32`,
  `aspect: f32`, `near: f32`, `far: f32`.
- Methods: `eye()`, `view_matrix()`, `proj_matrix()`, `view_proj()`,
  `orbit(dyaw, dpitch)`, `zoom(delta)`, `pan(dx, dy)`, `reset()`.
- `Default` impl: 45° fov, distance 10, pitch 30°, sane near/far clipping planes.

### `RustSynth/crates/rustsynth_render_api/src/backend.rs`
- Replaced minimal 5-method trait stub with the full 9-method `ViewportBackend` trait.
- Added `PointerButton` and `InputEvent` enums for input routing from the GTK4 shell.

### `RustSynth/crates/rustsynth_render_api/src/adapter.rs`
- Replaced empty file with geometry decomposition helpers shared between exporters and
  viewport backends.
- `decompose_transform(obj)` → `(base, dir1, dir2, dir3)` column vectors from `Mat4`.
- `matrix_row_str`, `column_matrix_str`, `pov_matrix_str`, `cam_column_matrix_str` —
  string formatters for template exporter placeholder substitution.
- `sphere_center_radius(obj)` → `(center: Vec3, radius: f32)`.
- `cylinder_endpoints(obj)` → `(base: Vec3, top: Vec3, radius: f32)`.

### `RustSynth/crates/rustsynth_render_api/src/lib.rs`
- Updated re-exports: `InputEvent`, `PointerButton`, `ViewportBackend`, `ArcballCamera`.

### `RustSynth/crates/rustsynth_viewport_gl/src/backend.rs`
- Added `camera: ArcballCamera` field to `GlBackend`.
- Added `Default` impl.
- Added stub implementations for `camera()`, `camera_mut()`, `handle_input()`, `backend_name()`.

### `RustSynth/crates/rustsynth_viewport_bevy/src/backend.rs`
- Same changes as `GlBackend` above, applied to `BevyBackend`.
- `backend_name()` returns `"bevy-stub"`.

### `AI/MASTER_TODO.md`
- Marked T10A `DONE`.
- T14 unblocked: `BLOCKED` → `READY`.
- T17 unblocked: `BLOCKED` → `READY`.

## Tests run
- `cargo test --workspace` — 64 passed, 0 failed.
