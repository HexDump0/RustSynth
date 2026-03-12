# T17 — Changes

## New files

### `crates/rustsynth_viewport_wgpu/Cargo.toml`
- New crate added to workspace
- Dependencies: `wgpu 24`, `bytemuck 1` (with `derive`), `pollster 0.4`, plus workspace deps (`rustsynth_core`, `rustsynth_scene`, `rustsynth_render_api`, `anyhow`, `log`, `glam`)

### `crates/rustsynth_viewport_wgpu/src/lib.rs`
- Crate root, exports `WgpuBackend` and five modules

### `crates/rustsynth_viewport_wgpu/src/gpu_types.rs`
- `Vertex` — position (f32×3), normal (f32×3), color (f32×4) with wgpu vertex layout
- `CameraUniform` — view_proj (f32×16), eye_pos (f32×4) for shader binding

### `crates/rustsynth_viewport_wgpu/src/shader.rs`
- `MAIN_SHADER` — WGSL source string
- Vertex shader: transforms position by `camera.view_proj`, passes world position/normal/color to fragment
- Fragment shader: Blinn-Phong with two directional lights, per-vertex color, ambient+diffuse+specular

### `crates/rustsynth_viewport_wgpu/src/geometry.rs`
- `scene_to_mesh()` — converts `&[SceneObject]` to merged `(Vec<Vertex>, Vec<u32>)`
- Per-primitive generators: `make_box`, `make_sphere`, `make_cylinder`, `make_line`, `make_dot`, `make_grid`, `make_triangle`
- 5 unit tests

### `crates/rustsynth_viewport_wgpu/src/pipeline.rs`
- `Pipeline::new()` — creates shader module, bind group layout, render pipeline with depth stencil
- `create_depth_texture()` — helper for depth buffer creation/recreation on resize

### `crates/rustsynth_viewport_wgpu/src/backend.rs`
- `WgpuBackend` — full `ViewportBackend` implementation (9 methods)
- `render_to_view()` — public API for GtkGLArea integration
- `device()`, `queue()` — accessors for external integration
- `set_clear_color()`, `set_instance()` — configuration helpers
- 7 unit tests

## Modified files

### `Cargo.toml` (workspace root)
- Added `"crates/rustsynth_viewport_wgpu"` to workspace members

## Test results

78 workspace tests passed, 0 failed. 14 new tests in `rustsynth_viewport_wgpu`.
