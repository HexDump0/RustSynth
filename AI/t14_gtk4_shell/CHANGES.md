# T14 Changes

## New Files

### `RustSynth/crates/rustsynth_app_gtk/src/main.rs`
Replaced placeholder. Initializes `env_logger`, creates `RelmApp`, launches `AppModel`.

### `RustSynth/crates/rustsynth_app_gtk/src/app.rs` (~839 lines)
- `AppMsg` enum — 21 message variants covering all user actions
- `AppModel` struct — source, config, file path, status, backend, last texture, viewport dims
- `AppWidgets` struct — window, text_view, picture, status_label, toolbar widgets
- `Component` impl:
  - `init()` — builds full GTK widget tree, registers window actions + keyboard shortcut
  - `update_with_view()` — message dispatch for all UI actions including async file dialogs
- File I/O: `async` GTK4 FileDialog (open + save) via `spawn_local`
- Export: OBJ export (via FileDialog → `rustsynth_export_obj`) and Template export
  (via FileDialog → embedded XML template → `rustsynth_export_template`)
- Camera: `OrbitCamera`, `PanCamera`, `Scroll` messages mutate `WgpuBackend` camera,
  then re-render via `render_scene_to_texture_no_reload()`
- Settings: `ShowSettings` → `settings_dialog::show()`, `SettingsUpdated` → rebuild

### `RustSynth/crates/rustsynth_app_gtk/src/pipeline.rs`
- `run_pipeline(source: &str, config: &BuildConfig) -> anyhow::Result<(Scene, Vec<String>)>`
  - Calls: preprocess → parse → resolve_names → validate → build_scene
  - Collects warnings from `build_scene` diagnostics

### `RustSynth/crates/rustsynth_app_gtk/src/viewport.rs`
- `render_scene_to_texture(backend, scene, width, height) -> Result<gdk::MemoryTexture>`
  — loads scene into backend, then renders
- `render_scene_to_texture_no_reload(backend, width, height) -> Result<gdk::MemoryTexture>`
  — renders with whatever scene is currently loaded (for camera moves)
- `pixels_to_gdk_texture(backend, width, height) -> Result<gdk::MemoryTexture>`
  — calls `render_to_pixels()` then wraps bytes in `gdk::MemoryTexture::new()`
  using `gdk::MemoryFormat::R8g8b8a8`, stride = `width * 4`

### `RustSynth/crates/rustsynth_app_gtk/src/settings_dialog.rs`
- `show<F>(parent: &gtk::ApplicationWindow, current: &BuildConfig, on_apply: F)`
  — modal GTK window with SpinButtons for: max_generations, max_objects, min_dim, max_dim
  — DropDown for recursion mode (BFS / DFS), Switch for sync_random
  — Apply button closes window and calls closure with new `BuildConfig`

---

## Modified Files

### `RustSynth/Cargo.toml`
Added two workspace dependencies under `[workspace.dependencies]`:
```toml
gtk4 = { version = "0.10", features = ["v4_10"] }
relm4 = "0.10"
```

### `RustSynth/crates/rustsynth_app_gtk/Cargo.toml`
Enabled all runtime dependencies:
- `rustsynth_viewport_wgpu = { workspace = true }` (was stub)
- `gtk4 = { workspace = true }` (new)
- `relm4 = { workspace = true }` (new)
- Kept all previously-listed deps: core, eisenscript, eval, scene, semantics,
  export_obj, export_template, wgpu, anyhow, env_logger, log
- Added `[[bin]] name = "rustsynth" path = "src/main.rs"`

### `RustSynth/crates/rustsynth_viewport_wgpu/src/backend.rs`
- Added field `surface_format: wgpu::TextureFormat` to `WgpuBackend` (default `Rgba8Unorm`)
- Added `pub fn set_surface_format(&mut self, format: wgpu::TextureFormat)`
- Modified `init()` to use `self.surface_format` instead of hardcoded `Bgra8UnormSrgb`
- Added `pub fn render_to_pixels(&mut self, width: u32, height: u32) -> anyhow::Result<Vec<u8>>`
  - Uses `TexelCopyTextureInfo`, `TexelCopyBufferInfo`, `TexelCopyBufferLayout` (wgpu 24 API)
  - Handles `COPY_BYTES_PER_ROW_ALIGNMENT = 256` padding in readback stride
  - Synchronizes via mpsc channel + `map_async` + `device.poll(Wait)`

---

## Known Omissions / Deferred

- **Viewport resize tracking**: `gtk::Picture` has no `connect_resize` signal.
  The viewport dimensions are stored in `AppModel` but only updated via explicit
  `AppMsg::ViewportResized`. Wiring this properly (e.g. via a `DrawingArea` overlay
  or periodic polling) is deferred.
- **Async render**: All rendering happens synchronously on the GTK main thread.
  For large scenes this will block the UI. Moving render to a thread pool is deferred.
- **Undo/redo**: Not implemented.
- **Syntax highlighting**: Not implemented (plain `TextView`).
