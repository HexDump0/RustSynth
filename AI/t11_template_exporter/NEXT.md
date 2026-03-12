# T11 — Next

## Recommended next task: T12 — Implement OBJ exporter

T11 and T12 are independent; T12 can be done immediately after T11.

## What T12 needs

- `rustsynth_scene` — same `Scene` / `SceneObject` model T11 uses.
- `rustsynth_render_api::adapter` — `decompose_transform`, `sphere_center_radius`,
  `cylinder_endpoints` geometry helpers already used by T11.
- OBJ format reference: legacy `ObjRenderer.cpp` (339 lines) in
  `StructureSynth/SyntopiaCore/GLEngine/Raytracer/`.

## What T12 produces

- `rustsynth_export_obj` crate: `ObjExporter`, `ObjOutput { obj: String, mtl: String }`.
- Tessellates each primitive to triangles/quads and writes standard `.obj` + `.mtl` pair.
- Groups by tag or colour so the MTL file has one material per visual group.

## After T12

T13 (scripting decision doc) is the last `READY` task. After that, T14 (GTK4 app shell)
is the main focus.

## Notes for the T12 agent

- The legacy `ObjRenderer.cpp` tessellates:
  - **Box** — 6 quads (24 vertices, normals outward).
  - **Sphere** — UV sphere, configurable lat/lon subdivisions.
  - **Cylinder** — closed cylinder, configurable segments.
  - **Grid** — wireframe (line segments as degenerate quads or `l` records).
  - **Line / Dot** — emitted as `l` / `p` records in OBJ.
- The `.mtl` file needs one `newmtl` entry per unique colour (or per group).
  Use `usemtl <name>` before each group's face records in the `.obj`.
- OBJ indices are 1-based. Accumulate a running vertex/normal offset when
  merging multiple groups into one file.
