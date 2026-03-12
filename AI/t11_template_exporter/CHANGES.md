# T11 — Changes

## Files changed

### `RustSynth/crates/rustsynth_export_template/Cargo.toml`
- Added `rustsynth_render_api = { path = "../rustsynth_render_api" }` dependency.
  Used for geometry decomposition helpers (`adapter.rs`).

### `RustSynth/crates/rustsynth_export_template/src/template.rs`
- Replaced empty stub with full XML template parser.
- `TemplatePrimitive::apply_substitutions` replaces all `{key}` tokens from a `HashMap`.
- `Template::from_xml` parses the `<template>` root element, all `<primitive>` children,
  and the optional `<description>` element using `quick-xml`.

### `RustSynth/crates/rustsynth_export_template/src/exporter.rs`
- Replaced empty stub with `TemplateExporter` and `ExportCamera`.
- `export(&mut self, scene: &Scene) -> Result<String>`:
  1. Emits `begin` block (camera, viewport, background substitutions).
  2. Iterates `scene.objects`, dispatches by `PrimitiveKind`:
     - `Box` / `Grid` / `Mesh` / `Cylinder` → `apply_standard` (matrix + colour subs).
     - `Sphere` → `apply_sphere` (centre + radius subs).
     - `Line` → `apply_line` (`{x1}`–`{z2}`).
     - `Dot` → `apply_dot` (`{x}`–`{z}`).
     - `Triangle` → `apply_triangle` (`{p1x}`–`{p3z}`).
     - Unknown → warn once, skip.
  3. Emits `end` block.
- `missing_warned: HashSet<String>` ensures each unknown primitive type is warned exactly once.
- `counter: u32` provides monotonically increasing `{uid}` values.

### `RustSynth/crates/rustsynth_export_template/src/lib.rs`
- Updated re-exports: `ExportCamera`, `TemplateExporter`, `Template`.

## Tests run
- `cargo test -p rustsynth_export_template` — 8 passed, 0 failed.
- `cargo test --workspace` — 64 passed, 0 failed.
