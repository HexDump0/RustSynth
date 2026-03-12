# T01 — Changes

## Files created

| File | Reason |
|---|---|
| `RustSynth/Cargo.toml` | Workspace root: members, shared deps, release profile |
| `RustSynth/.cargo/config.toml` | Build aliases (`ck`, `t`) |
| `RustSynth/Clippy.toml` | Workspace-wide lint policy |
| `RustSynth/tests/README.md` | Placeholder for integration test directory |
| `RustSynth/tests/fixtures/README.md` | Placeholder for golden fixture directory |
| `crates/rustsynth_core/Cargo.toml` + `src/` | Core types, errors, math, color, RNG |
| `crates/rustsynth_eisenscript/Cargo.toml` + `src/` | Preprocessor, lexer, parser, AST, diagnostics skeletons |
| `crates/rustsynth_semantics/Cargo.toml` + `src/` | Rule graph, resolution, primitive list, validation skeletons |
| `crates/rustsynth_eval/Cargo.toml` + `src/` | Builder, state, transform, recursion mode skeletons |
| `crates/rustsynth_scene/Cargo.toml` + `src/` | Scene, object, primitive, camera skeletons |
| `crates/rustsynth_render_api/Cargo.toml` + `src/` | `ViewportBackend` trait, adapter, camera skeletons |
| `crates/rustsynth_export_template/Cargo.toml` + `src/` | `TemplateExporter` and `Template` skeletons |
| `crates/rustsynth_export_obj/Cargo.toml` + `src/` | `ObjExporter` and tessellate skeletons |
| `crates/rustsynth_app_gtk/Cargo.toml` + `src/main.rs` | App shell skeleton (GTK4/Relm4 deps commented, added in T14) |
| `crates/rustsynth_viewport_bevy/Cargo.toml` + `src/` | Bevy backend skeleton (Bevy dep commented, added in T15) |
| `crates/rustsynth_viewport_gl/Cargo.toml` + `src/` | OpenGL backend skeleton (gl dep commented, added in T16) |

## Tests run
- `cargo check --workspace` — clean
- `cargo test --workspace` — 4 pass, 0 fail
