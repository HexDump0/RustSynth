# T27 — Migrate from GTK4+Relm4+wgpu to Tauri+React+Three.js

## What was done

Replaced the GTK4+Relm4 desktop shell (`rustsynth_app_gtk`) and wgpu viewport (`rustsynth_viewport_wgpu`) with a Tauri v2 + React + Three.js stack for cross-platform support and full design control.

### Changes

1. **Added serde derives** to all scene types (`Scene`, `SceneObject`, `PrimitiveKind`), `BuildConfig`, `RecursionMode`, and `GuiParam` so the Rust core can serialize scenes to JSON for the frontend.

2. **Created `rustsynth_app_tauri`** — a new crate containing:
   - `src-tauri/` — Tauri v2 Rust backend with commands: `run_script`, `open_file_dialog`, `save_file_dialog`, `export_obj`, `export_template`
   - `src/` — React + TypeScript frontend with Three.js viewport
   - Pipeline code mirrors `rustsynth_app_gtk::pipeline` but has no GTK dependencies

3. **Frontend components:**
   - `App.tsx` — main app with editor, toolbar, status bar, console
   - `Viewport.tsx` — Three.js scene renderer using @react-three/fiber, renders all primitive types (Box, Sphere, Cylinder, Line, Dot, Grid, Triangle)
   - `VariablePanel.tsx` — slider/input controls for `#define` GUI parameters
   - `styles.css` — Catppuccin-inspired dark theme

4. **Added to workspace** — `crates/rustsynth_app_tauri/src-tauri` added to `Cargo.toml` workspace members.

### What was NOT changed

- All headless core crates remain untouched: `rustsynth_core`, `rustsynth_eisenscript`, `rustsynth_semantics`, `rustsynth_eval`, `rustsynth_scene`, `rustsynth_render_api`, `rustsynth_export_template`, `rustsynth_export_obj`
- The GTK app (`rustsynth_app_gtk`) and wgpu viewport (`rustsynth_viewport_wgpu`) remain in the workspace but are now deprecated in favor of the Tauri app
- All existing tests still pass

## Why

- GTK4 limits cross-platform reach (no web, no Android, difficult macOS/Windows)
- wgpu+GtkGLArea EGL is complex plumbing; Three.js provides the same rendering with less code
- React gives full control over UI design
- Tauri v2 supports desktop + mobile; same frontend runs in a browser via WASM
