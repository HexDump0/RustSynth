# T29 — Simplify architecture: CLI + remove render_api

## Goal

Refactor the codebase into a clear two-layer architecture:
- **Rust**: core compute (parsing, evaluation, export) + standalone CLI
- **Web**: UI wrapper (Tauri + React + Three.js) — rendering only, no compute

## Approach

1. Moved `adapter.rs` geometry helpers from `rustsynth_render_api` into `rustsynth_scene` (where they belong — they operate on `SceneObject`)
2. Removed the `rustsynth_render_api` crate entirely
3. Created `rustsynth_cli` binary crate with `clap` subcommands: `build`, `export-obj`, `export-template`
4. Updated all documentation (AGENT_INSTRUCTIONS, REWRITE_ROADMAP, MASTER_TODO) to reflect the simplified architecture
5. Removed all references to deprecated viewport backends (Bevy, wgpu, OpenGL, GTK4+Relm4)

## Result

✅ Complete. Workspace compiles cleanly. The Rust core can now be used three ways:
- As library crates (by any Rust project)
- Via CLI (`rustsynth build`, `rustsynth export-obj`, `rustsynth export-template`)
- Via Tauri IPC (from the React web UI)
