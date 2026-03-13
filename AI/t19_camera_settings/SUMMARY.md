# T19 — Camera/settings import-export

## Task goal
Persist the active viewport camera state to/from JSON files, and support
embedding the camera as a `// @rs-camera: {...}` annotation in EisenScript
source files so the view is automatically restored on file open.

## Approach

### `rustsynth_render_api` crate
- Added `serde = { workspace = true }` to `Cargo.toml`.
- Added `#[derive(Serialize, Deserialize)]` to `ArcballCamera` in `camera.rs`.
  `glam`'s `Vec3` already has serde support via the workspace `glam` feature.

### New crate `rustsynth_app_gtk/src/camera_io.rs`
Contains four public functions:

| Function | Description |
|---|---|
| `save_camera(camera, path)` | Serialize to pretty JSON |
| `load_camera(path)` | Deserialize from JSON |
| `camera_to_annotation(camera)` | `"// @rs-camera: {...}"` string |
| `insert_camera_annotation(source, camera)` | Prepend or replace annotation in source |
| `extract_camera_annotation(source)` | Parse annotation from first line |

### `rustsynth_app_gtk/Cargo.toml`
Added `serde` and `serde_json` workspace dependencies.

### App shell changes (`app.rs`)
**New `AppMsg` variants:**
- `ExportCamera` / `CameraExportPicked(PathBuf)` — save camera to JSON
- `ImportCamera` / `CameraImportPicked(PathBuf)` — load camera from JSON
- `InsertCameraIntoScript` — embed `// @rs-camera:` as first line of current script

**File menu additions** (under a "Camera" submenu section):
- Export Camera…
- Import Camera…
- Insert Camera into Script

**`AppMsg::FileOpened` handler**: now calls `camera_io::extract_camera_annotation`
on the loaded content; if present the camera is restored before rendering.

**New helpers:**
- `export_camera_state(path)` — calls `camera_io::save_camera`
- `import_camera_state(path)` — calls `camera_io::load_camera`, sets `backend.camera_mut()`

After `ImportCamera` the `needs_rerender` flag is set to re-render the viewport
at the new camera position without rebuilding scene geometry.

### JSON schema
The camera JSON uses `serde`'s default field mapping for `ArcballCamera`:
```json
{
  "pivot": [0.0, 0.0, 0.0],
  "yaw": 30.0,
  "pitch": 20.0,
  "distance": 5.0,
  "fov_y": 45.0,
  "aspect": 1.0,
  "near": 0.01,
  "far": 1000.0
}
```

## Result
Complete. Camera can be exported/imported as JSON and embedded in scripts.
Scripts saved with `Insert Camera into Script` restore the camera on open.

## Status
Complete. 4 new tests in `camera_io::tests`.
