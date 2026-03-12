# T10 — Define canonical scene representation

## Task goal
Finalise the `rustsynth_scene` crate as the renderer-agnostic scene model that the evaluator
writes to and all export/viewport backends read from.

## Approach
- Reviewed what `Builder.cpp` stores per-object: transform matrix, colour, alpha, primitive
  type, and optional class tag.
- Extended `Scene` with a `raw_settings` field to carry pass-through `set` keys (raytracer
  hints, template parameters) that have no meaning in the evaluator.
- Added `Default` impl to `CameraState` (identity rotation, zero translation/pivot, scale 1).
- Changed `PrimitiveKind::Triangle` from a unit variant to `Triangle(String)` to carry the
  mesh vertex payload from `triangle[...]` references.

## Result
- `scene.rs` — `Scene { objects, background, camera, raw_settings: Vec<(String, String)> }`.
- `camera.rs` — `CameraState` with `Default` impl.
- `primitive.rs` — `PrimitiveKind::Triangle(String)`.
- `object.rs` — `SceneObject { kind, transform: Mat4, color: Rgba, alpha: f32, tag: Option<String> }`.

## Status
Complete.
