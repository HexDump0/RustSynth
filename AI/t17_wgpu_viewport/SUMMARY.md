# T17 — wgpu viewport backend

## Result

Implemented `rustsynth_viewport_wgpu`, the chosen viewport backend for RustSynth. The crate provides a full `ViewportBackend` trait implementation using `wgpu`, ready for integration with the GTK4 app shell via `GtkGLArea`.

## Architecture

### Crate modules

| Module | Purpose |
|---|---|
| `backend` | `WgpuBackend` struct implementing all 9 `ViewportBackend` methods |
| `geometry` | Mesh generation — converts `SceneObject` list into vertex/index buffers |
| `gpu_types` | `bytemuck`-compatible `Vertex` and `CameraUniform` structs for GPU upload |
| `pipeline` | Render pipeline, bind group layout, and depth texture creation |
| `shader` | Embedded WGSL shader with Blinn-Phong two-light illumination |

### Key design decisions

1. **All geometry is pre-baked at `load_scene` time** — no per-frame allocation. Scene objects are tessellated into a single merged vertex/index buffer pair and uploaded once.

2. **Two rendering modes**:
   - **GtkGLArea integration**: the app shell calls `render_to_view(&texture_view)` from the GTK `render` signal, providing the framebuffer.
   - **Standalone surface**: if a `wgpu::Surface` is configured, `render_frame()` acquires the swapchain texture automatically.

3. **WGSL shader** uses Blinn-Phong with two directional lights for coverage similar to legacy Structure Synth. Two-sided lighting (no backface culling) handles arbitrarily oriented geometry.

4. **Camera uniform** uploads `view_proj` matrix and `eye_pos` once per frame via `queue.write_buffer`.

5. **Input handling** maps to arcball camera: primary drag → orbit, secondary/middle drag → pan, scroll → zoom, reset.

### Primitive support

| Primitive | Geometry strategy |
|---|---|
| Box | Oriented bounding box from decomposed transform (24 verts, 36 indices) |
| Sphere | UV sphere at center/radius from `adapter::sphere_center_radius` |
| Cylinder | Capped cylinder from `adapter::cylinder_endpoints` |
| Line | Thin cylinder approximation |
| Dot | Small sphere at object center |
| Grid | Flat quad from transform X/Y axes |
| Triangle | Parsed from raw vertex payload string |
| Mesh | Falls back to box representation |
| Template | Invisible (no geometry) |

## Status

Complete. All workspace tests pass: **78 total** (14 new in this crate).
