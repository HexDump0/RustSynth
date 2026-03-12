# T10 — Next

## Recommended next task
Four tasks are now `READY`. Suggested order:

1. **T10A** — Define renderer boundary and viewport API.  
   This is the key boundary task. T14, T15, T16 are all blocked on it.
2. **T11** — Template exporter (uses `Scene`, `raw_settings`, `CameraState`).
3. **T12** — OBJ exporter (straightforward, only needs `objects` and `transform`).
4. **T13** — Scripting compatibility decision doc (no code, just research + decision).

## Notes for the next agent

### Scene model design decisions
- `SceneObject::transform` is a `Mat4` in column-major order (glam convention).
  The upper-left 3×3 encodes rotation × non-uniform scale; the last column is translation.
  The pivot offset `(0.5, 0.5, 0.5)` is **baked in** — see T09 notes.
- `SceneObject::color` is the final `Rgba` for the object; `alpha` is stored separately
  because the legacy renderer treats it as a distinct blending parameter.
- `Scene::raw_settings` is ordered (Vec, not HashMap) to preserve source order for template
  exporters that emit settings in the order they appear.
- `CameraState` stores the rotation as a flat `[f32; 16]` (row-major logical layout, but
  the `rotation_mat4()` helper returns a column-major `Mat4` for glam). The field order
  matches the legacy `set rotation [r0c0 r0c1 r0c2 r1c0 ...]` syntax.

### What T10A needs to define
- A `Renderer` or `Viewport` trait that accepts a `&Scene` and a render/paint request.
- An async or event-based notification back to the app shell when a frame is ready.
- How the viewport backend integrates with a GTK4 drawing area or native window handle.

### What the exporters need (T11 / T12)
- T11 (template): reads `raw_settings` for template variable substitution, reads `objects`
  for geometry, reads `camera` for camera block output.
- T12 (OBJ): reads `objects`, applies `transform` to unit-cube vertices, writes `.obj` + `.mtl`.
  The legacy OBJ exporter is in `SyntopiaCore/GLEngine/Raytracer/`.
