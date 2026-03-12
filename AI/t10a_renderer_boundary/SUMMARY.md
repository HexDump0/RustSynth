# T10A — Define renderer boundary and viewport API

## Task goal
Define the shared viewport/renderer abstraction that sits between the headless core
(`rustsynth_scene`) and any concrete rendering backend (wgpu, OpenGL, Bevy).

## Approach
- Studied the legacy `GLEngine` and `Builder.cpp` to identify what a renderer needs from
  the scene (transform matrix, colour, alpha, primitive kind, camera state).
- Chose an arcball orbit camera model to match the legacy StructureSynth camera behaviour.
- Defined `ViewportBackend` as a single trait implemented by every rendering backend.
- Added `InputEvent` / `PointerButton` enums so the GTK4 shell can feed pointer and scroll
  events into any backend without knowing which one is active.
- Added geometry decomposition helpers in `adapter.rs` to keep transform unpacking logic
  in one place (used by both exporters and viewport backends).

## Result

### `crates/rustsynth_render_api/src/camera.rs`
`ArcballCamera { pivot, yaw, pitch, distance, fov_y, aspect, near, far }` with full
`view_matrix()`, `proj_matrix()`, `view_proj()`, `orbit()`, `zoom()`, `pan()`, `reset()`.

### `crates/rustsynth_render_api/src/backend.rs`
```rust
pub enum PointerButton { Primary, Secondary, Middle }
pub enum InputEvent { PointerDrag { button, dx, dy }, Scroll { delta }, Pan { dx, dy }, ResetCamera }

pub trait ViewportBackend {
    fn init(&mut self) -> anyhow::Result<()>;
    fn shutdown(&mut self);
    fn load_scene(&mut self, scene: &Scene) -> anyhow::Result<()>;
    fn render_frame(&mut self) -> anyhow::Result<()>;
    fn resize(&mut self, width: u32, height: u32);
    fn camera(&self) -> &ArcballCamera;
    fn camera_mut(&mut self) -> &mut ArcballCamera;
    fn handle_input(&mut self, event: InputEvent) -> bool;
    fn backend_name(&self) -> &'static str;
}
```

### `crates/rustsynth_render_api/src/adapter.rs`
`decompose_transform`, `matrix_row_str`, `column_matrix_str`, `pov_matrix_str`,
`cam_column_matrix_str`, `sphere_center_radius`, `cylinder_endpoints`.

### Stub backends updated
`GlBackend` (`rustsynth_viewport_gl`) and `BevyBackend` (`rustsynth_viewport_bevy`) updated
with stub implementations for all 4 new methods so the workspace compiles.

## Status
Complete. All workspace tests pass (64 total).
