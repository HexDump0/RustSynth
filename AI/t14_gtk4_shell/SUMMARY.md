# T14 — GTK4 + Relm4 Desktop App Shell

## Goal

Build the GTK4 + Relm4 desktop application shell for RustSynth:
- Main window with HeaderBar, toolbar, split editor/viewport, status bar
- Code editor (GtkTextView) with EisenScript source
- Viewport display using wgpu offscreen render → GDK MemoryTexture → GtkPicture
- File I/O (Open, Save, Save As) via GTK4 FileDialog
- Export (OBJ, Template) with file picker dialogs
- Settings dialog (BuildConfig fields)
- Camera interaction (orbit, pan, zoom) via GTK gesture controllers

## Approach

### Viewport rendering strategy

`WgpuBackend` already existed from T17 but used `Bgra8UnormSrgb` format (for GtkGLArea EGL
integration).  For T14 we use an **offscreen CPU-readback path**:

```
run_pipeline() → Scene → WgpuBackend::load_scene() → WgpuBackend::render_to_pixels()
                                                → Vec<u8> (Rgba8Unorm)
                                                → gdk::MemoryTexture (R8g8b8a8)
                                                → gtk::Picture::set_paintable()
```

This is correct, deterministic, and requires no GL context sharing.  Camera
interaction re-renders via the same path.

### WgpuBackend changes (T17 crate)

- Added `surface_format: wgpu::TextureFormat` field (default: `Rgba8Unorm`)
- Added `set_surface_format()` to allow override before `init()`
- `init()` now uses `self.surface_format` instead of hardcoded `Bgra8UnormSrgb`
- Added `render_to_pixels(width, height) -> anyhow::Result<Vec<u8>>`
  - Creates an offscreen texture, calls `render_to_view`, copies to CPU via
    `queue.submit` + `device.poll(Wait)` + `map_async`
  - Handles row-stride alignment (`COPY_BYTES_PER_ROW_ALIGNMENT = 256`)

### Crate structure (`rustsynth_app_gtk`)

| File | Purpose |
|---|---|
| `src/main.rs` | Entry point: `RelmApp::new("io.rustsynth.app").run::<AppModel>(())` |
| `src/app.rs` | `AppModel` (Relm4 `Component`), `AppMsg`, `AppWidgets`, full message loop |
| `src/pipeline.rs` | Headless compile pipeline: preprocess → parse → resolve → build |
| `src/viewport.rs` | `render_scene_to_texture()` and `render_scene_to_texture_no_reload()` |
| `src/settings_dialog.rs` | Settings window with BuildConfig fields |

### Relm4 architecture

Uses `Component` (not `SimpleComponent`) to get access to both widgets and
root window in `update_with_view`.  This is needed for:
- File dialogs (`gtk::FileDialog::open/save` needs `&Window`)
- Settings dialog (`transient_for` needs `&Window`)
- Widget updates after rendering

### Window actions

Seven `gio::SimpleAction` instances are registered on the `ApplicationWindow`:
`new-file`, `open-file`, `save-file`, `save-file-as`, `export-obj`,
`export-template`, `run`.  These are wired to menu items.  F5 triggers `run`
via a `ShortcutController`.

### Camera interaction

Three GTK gesture/event controllers on the `gtk::Picture`:
- `GestureDrag` (primary button) → `AppMsg::OrbitCamera(dx, dy)` 
- `GestureDrag` (secondary button) → `AppMsg::PanCamera(dx, dy)` 
- `EventControllerScroll` → `AppMsg::Scroll(dy)` 

Each delta is computed from the cumulative drag offset to get per-event deltas.

## Result

Complete. All workspace tests pass (78 total, 0 new — no regression).

```
$ cargo build -p rustsynth_app_gtk
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
$ cargo test --workspace
... ok. 78 tests, 0 failures
```

The `rustsynth` binary can be run once a display is available.

### Pre-migration Polish (HexDump0)
Before the Tauri migration, the following features were added to `rustsynth_app_gtk` by HexDump0:
- **Syntax Highlighting**: Added syntax highlighting into `gtk::TextView` implementing custom tag colorization rules covering primitives, transforms, state rules, block symbols, and `#` preprocessor defines.
- **Variables UI**: Added a settings panel allowing UI modification of script variables parameterised using the `#define` pattern (`T18`).
- **Camera Settings**: Integrated the complete camera IO save/restore options natively into UI bindings (`T19`).
- **Completed GUI Controls**: Fully fleshed-out options on the menubar for object counts & parameters in the header UI (`T20`).
- Now obsolete due to the complete migration towards the React application scope via `T27`.
