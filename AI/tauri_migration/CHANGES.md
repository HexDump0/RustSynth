# Files created/edited

## Created
- `crates/rustsynth_app_tauri/package.json` — npm project with React, Three.js, Tauri deps
- `crates/rustsynth_app_tauri/vite.config.ts` — Vite config for Tauri dev
- `crates/rustsynth_app_tauri/tsconfig.json` — TypeScript config
- `crates/rustsynth_app_tauri/index.html` — HTML entry point
- `crates/rustsynth_app_tauri/src/main.tsx` — React entry
- `crates/rustsynth_app_tauri/src/App.tsx` — main application component
- `crates/rustsynth_app_tauri/src/types.ts` — TypeScript types mirroring Rust scene types
- `crates/rustsynth_app_tauri/src/styles.css` — dark theme styles
- `crates/rustsynth_app_tauri/src/components/Viewport.tsx` — Three.js viewport
- `crates/rustsynth_app_tauri/src/components/VariablePanel.tsx` — GUI parameter controls
- `crates/rustsynth_app_tauri/src-tauri/Cargo.toml` — Tauri backend crate
- `crates/rustsynth_app_tauri/src-tauri/build.rs` — Tauri build script
- `crates/rustsynth_app_tauri/src-tauri/tauri.conf.json` — Tauri config
- `crates/rustsynth_app_tauri/src-tauri/capabilities/default.json` — Tauri permissions
- `crates/rustsynth_app_tauri/src-tauri/src/main.rs` — binary entry
- `crates/rustsynth_app_tauri/src-tauri/src/lib.rs` — Tauri app setup
- `crates/rustsynth_app_tauri/src-tauri/src/pipeline.rs` — headless pipeline
- `crates/rustsynth_app_tauri/src-tauri/src/commands.rs` — Tauri IPC commands
- `crates/rustsynth_app_tauri/src-tauri/icons/icon.png` — placeholder app icon

## Edited
- `RustSynth/Cargo.toml` — added `crates/rustsynth_app_tauri/src-tauri` to workspace members
- `crates/rustsynth_scene/src/scene.rs` — added Serialize, Deserialize
- `crates/rustsynth_scene/src/object.rs` — added Serialize, Deserialize
- `crates/rustsynth_scene/src/primitive.rs` — added Serialize, Deserialize
- `crates/rustsynth_eval/src/builder.rs` — added Serialize, Deserialize to BuildConfig
- `crates/rustsynth_eval/src/recursion.rs` — added Serialize, Deserialize to RecursionMode
- `crates/rustsynth_eisenscript/src/preprocessor.rs` — added Serialize, Deserialize to GuiParam
- `crates/rustsynth_eisenscript/Cargo.toml` — added serde dependency
- `crates/rustsynth_eval/Cargo.toml` — added serde dependency
