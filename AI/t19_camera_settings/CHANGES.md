# T19 — Changes

## Files changed

| File | Reason |
|---|---|
| `crates/rustsynth_render_api/Cargo.toml` | Add `serde = { workspace = true }` |
| `crates/rustsynth_render_api/src/camera.rs` | Add `Serialize, Deserialize` derive to `ArcballCamera` |
| `crates/rustsynth_app_gtk/Cargo.toml` | Add `serde` and `serde_json` workspace dependencies |
| `crates/rustsynth_app_gtk/src/camera_io.rs` | **NEW** — JSON save/load, annotation embed/extract functions |
| `crates/rustsynth_app_gtk/src/main.rs` | Add `mod camera_io;` |
| `crates/rustsynth_app_gtk/src/app.rs` | New `AppMsg` variants (ExportCamera, ImportCamera, InsertCameraIntoScript, CameraExportPicked, CameraImportPicked), new menu items, `FileOpened` camera restore, `export_camera_state`/`import_camera_state` helpers |

## Tests run

```
cargo test --workspace
# 86 total; 0 failed (4 new in camera_io::tests)
```

### New tests
- `annotation_round_trips` — serialize/deserialize preserves yaw/pitch/distance
- `extract_from_script_first_line` — annotation can be parsed after insertion
- `replace_existing_annotation` — insert_camera_annotation replaces existing line
- `no_annotation_returns_none` — non-annotated scripts return None
