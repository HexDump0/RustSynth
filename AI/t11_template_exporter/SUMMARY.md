# T11 — Implement template exporter

## Task goal
Port the legacy `TemplateRenderer.cpp` (489 lines) to a fully tested Rust exporter that
reads a template XML file, substitutes per-object placeholders, and emits the final text output.

## Approach
- Read `TemplateRenderer.cpp` in full to catalogue all placeholder tokens and the XML schema.
- Template XML format: `<template name="…" defaultExtension="…"><primitive name="box">…</primitive></template>`.
- Each primitive block is a text template with `{placeholder}` tokens that are replaced
  per-object with matrix, colour, camera, and geometry data.
- The exporter iterates `Scene::objects`, looks up the template primitive by `PrimitiveKind`,
  applies all substitutions, and concatenates the output.

## Result

### `crates/rustsynth_export_template/src/template.rs`
```rust
pub struct TemplatePrimitive { text: String }
// apply_substitutions(vars: &HashMap<&str, String>) -> String
pub struct Template {
    name, default_extension, run_after, description,
    full_text, primitives: HashMap<String, TemplatePrimitive>
}
impl Template { pub fn from_xml(xml: &str) -> Result<Self> }
```

### `crates/rustsynth_export_template/src/exporter.rs`
```rust
pub struct ExportCamera { position, target, up, right, width: u32, height: u32, fov }
pub struct TemplateExporter { template, camera, missing_warned: HashSet<String>, counter: u32 }
impl TemplateExporter { pub fn export(&mut self, scene: &Scene) -> Result<String> }
```

Placeholders fully covered:
`{matrix}`, `{columnmatrix}`, `{povmatrix}`, `{r}`, `{g}`, `{b}`, `{alpha}`,
`{oneminusalpha}`, `{uid}`, `{cx}`, `{cy}`, `{cz}`, `{rad}`,
`{x1}`–`{z2}` (cylinder endpoints), `{x}`–`{z}` (dot/line),
`{p1x}`–`{p3z}` (triangle vertices),
camera/viewport vars: `{cx}`, `{cy}`, `{cz}`, `{lx}`–`{lz}`, `{ux}`–`{uz}`,
`{rx}`–`{rz}`, `{width}`, `{height}`, `{fov}`, `{aspect}`,
`{background_r}`, `{background_g}`, `{background_b}`.

## Tests (8 passing)
- `parse_name` — Template `name` attribute parsed from XML.
- `parse_description` — `<description>` child element extracted.
- `parse_primitives_present` — `box`, `sphere`, `begin`, `end` primitives all present.
- `parse_primitive_text` — primitive body text round-trips correctly.
- `substitute_works` — `{r}` and `{g}` replaced with correct colour values.
- `export_box` — full export of a single-box scene matches expected fragment.
- `missing_primitive_skipped` — unknown primitive kind silently skipped (warn once).
- `uid_increments` — `{uid}` increases monotonically across objects.

## Status
Complete.
