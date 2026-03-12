# T12 — Implement OBJ exporter

## Task goal
Port the legacy `ObjRenderer.cpp` (339 lines) to a fully tested Rust exporter that
tessellates every `SceneObject` primitive into polygonal geometry and emits a standard
Wavefront `.obj` + `.mtl` file pair.

## Approach
- Read `ObjRenderer.cpp` in full to understand per-primitive tessellation algorithms and
  the grouping/material model.
- Chose UV-sphere tessellation (configurable lat/lon slices) to match the legacy approach.
- Used a `BTreeMap<String, GroupData>` keyed by tag+colour to collect geometry per group
  before serialisation, ensuring deterministic output order.
- Kept OBJ face records in the standard `v`/`vn`/`f` (or `l`/`p`) format compatible with
  all major 3D applications.

## Result

### `crates/rustsynth_export_obj/src/tessellate.rs`
```rust
pub struct VertexNormal { v: usize, n: usize }  // 1-based OBJ indices
pub type Face = Vec<VertexNormal>;
pub struct ObjGroup { name, vertices: Vec<Vec3>, normals: Vec<Vec3>, faces: Vec<Face> }
impl ObjGroup { pub fn merge(&mut self, other), pub fn deduplicate(&mut self) }

pub fn tessellate_box(origin, dir1, dir2, dir3) -> ObjGroup    // 6 quads, outward normals
pub fn tessellate_grid(origin, dir1, dir2, dir3) -> ObjGroup   // 12 wireframe edge quads
pub fn tessellate_sphere(dt, dp, transform: Mat4) -> ObjGroup  // UV sphere, configurable slices
pub fn tessellate_cylinder(base, top, radius, segments) -> ObjGroup  // closed cylinder
```

### `crates/rustsynth_export_obj/src/exporter.rs`
```rust
pub struct ObjExporter {
    sphere_segments: u32,
    group_by_tag: bool,
    group_by_color: bool,
    mtl_file_name: String,
}
pub struct ObjOutput { pub obj: String, pub mtl: String }
impl ObjExporter { pub fn export(&self, scene: &Scene) -> Result<ObjOutput> }
```
- `Line` → `l` records (2-vertex polyline).
- `Dot` → `p` records (single-vertex point).
- All other primitives → `f` face records via tessellation helpers.
- MTL: one `newmtl` per group with `Kd r g b` and `d alpha`.

## Tests (9 passing)
- `box_has_six_faces` — tessellate_box returns exactly 6 faces.
- `sphere_face_count` — UV sphere face count matches `segments² × 2` formula.
- `cylinder_side_faces` — side faces equal `segments`.
- `export_box_has_faces` — exported OBJ string contains `f` records.
- `export_sphere_has_faces` — exported OBJ string contains `f` records.
- `export_line_has_l_record` — line primitive emits `l` record.
- `export_dot_has_p_record` — dot primitive emits `p` record.
- `grouping_by_color` — two differently coloured objects produce two `g` groups.
- `mtl_has_material_for_each_group` — MTL has one `newmtl` per unique group.

## Status
Complete.
