# Next steps

## Immediate

1. **Run and test the Tauri app** — `cd crates/rustsynth_app_tauri && npx tauri dev`
2. **Replace the placeholder icon** — generate a proper app icon
3. **Add syntax highlighting to the editor** — consider CodeMirror or Monaco for a proper code editor with EisenScript highlighting
4. **Improve Three.js rendering fidelity** — match primitive geometry sizes/conventions to the legacy renderer exactly
5. **Camera state persistence** — port camera import/export and `// @rs-camera:` annotation support to the Tauri backend

## Medium term

6. **WASM target** — compile core crates to WASM + deploy the React frontend as a standalone web app (no Tauri needed)
7. **Mobile (Android/iOS)** — Tauri v2 mobile support
8. **Deprecate/remove GTK app** — once the Tauri app has parity, remove `rustsynth_app_gtk`, `rustsynth_viewport_wgpu`, and related crates
9. **Instanced rendering** — use Three.js `InstancedMesh` for scenes with many identical primitives (major perf win)
10. **Template file selection UI** — add template file picker to the export flow

## Architecture notes

- The Tauri app is at `crates/rustsynth_app_tauri/`
- Rust backend: `crates/rustsynth_app_tauri/src-tauri/`
- React frontend: `crates/rustsynth_app_tauri/src/`
- The headless pipeline is duplicated from the GTK app (same logic, no GTK deps)
- Scene serialization uses serde JSON: Rust Scene → JSON → TypeScript types → Three.js
