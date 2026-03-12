# T12 — Changes

## Files changed

### `RustSynth/crates/rustsynth_export_obj/Cargo.toml`
- Added `rustsynth_render_api = { path = "../rustsynth_render_api" }` dependency.
  Used for `adapter::decompose_transform`, `sphere_center_radius`, `cylinder_endpoints`.

### `RustSynth/crates/rustsynth_export_obj/src/tessellate.rs`
- Replaced empty stub with all four tessellation functions.
- `tessellate_box`: builds 6 quads with per-face outward normals; vertex order CCW when
  viewed from outside.
- `tessellate_grid`: 12 edge segments as thin quads (wireframe appearance in OBJ viewers).
- `tessellate_sphere`: UV sphere with configurable `dt` (theta steps) and `dp` (phi steps);
  poles handled as degenerate triangles collapsed to point.
- `tessellate_cylinder`: closed cylinder (two end caps + side quads); cap normals ±Y.
- `ObjGroup::merge` accumulates geometry from multiple objects into one group, adjusting
  1-based vertex/normal indices by the running offset.

### `RustSynth/crates/rustsynth_export_obj/src/exporter.rs`
- Replaced empty stub with `ObjExporter` and `ObjOutput`.
- `export(&self, scene: &Scene) -> Result<ObjOutput>`:
  1. Groups objects by tag+colour key into a `BTreeMap<String, GroupData>`.
  2. Tessellates each object by `PrimitiveKind` and merges into its group's `ObjGroup`.
  3. `serialize_obj()` emits `mtllib`, then for each group: `usemtl`, `g`, `v`, `vn`, `f`/`l`/`p`.
  4. `serialize_mtl()` emits one `newmtl … Kd … d` block per group.
- `Line` objects accumulate endpoint pairs into `l` records (no tessellation needed).
- `Dot` objects accumulate single `Vec3` positions into `p` records.

### `RustSynth/crates/rustsynth_export_obj/src/lib.rs`
- Updated re-exports: `ObjExporter`, `ObjOutput`.

## Tests run
- `cargo test -p rustsynth_export_obj` — 9 passed, 0 failed.
- `cargo test --workspace` — 64 passed, 0 failed.
