# T29 — Changes

## Files created
- `crates/rustsynth_cli/Cargo.toml` — new CLI binary crate
- `crates/rustsynth_cli/src/main.rs` — CLI with build/export-obj/export-template subcommands
- `crates/rustsynth_scene/src/adapter.rs` — geometry helpers moved from render_api

## Files modified
- `Cargo.toml` — added `rustsynth_cli` to workspace, added `clap` dep, removed `rustsynth_render_api`
- `crates/rustsynth_scene/src/lib.rs` — added `pub mod adapter`
- `crates/rustsynth_export_template/src/exporter.rs` — import from `rustsynth_scene::adapter` instead of `rustsynth_render_api::adapter`
- `crates/rustsynth_export_template/Cargo.toml` — removed `rustsynth_render_api` dep
- `crates/rustsynth_export_obj/src/exporter.rs` — import from `rustsynth_scene::adapter` instead of `rustsynth_render_api::adapter`
- `crates/rustsynth_export_obj/Cargo.toml` — removed `rustsynth_render_api` dep
- `crates/rustsynth_app_tauri/src-tauri/Cargo.toml` — renamed binary from `rustsynth` to `rustsynth-app`
- `AI/AGENT_INSTRUCTIONS.md` — rewritten for simplified architecture
- `AI/REWRITE_ROADMAP.md` — rewritten for simplified architecture
- `AI/MASTER_TODO.md` — updated task statuses and notes

## Files deleted
- `crates/rustsynth_render_api/` — entire crate removed (helpers merged into rustsynth_scene)

## Tests
- `cargo check` passes cleanly
