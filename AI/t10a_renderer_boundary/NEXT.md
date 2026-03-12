# T10A — Next

## Recommended next task: T14 — GTK4 + Relm4 desktop app shell

T10A was the blocking dependency for both T14 and T17. T14 (app shell) should come before
T17 (wgpu viewport) because T17 needs the `GtkGLArea` surface that T14 creates.

## What T14 needs from T10A

- `ViewportBackend` trait — the app shell stores a `Box<dyn ViewportBackend>` and swaps
  backends at runtime.
- `InputEvent` — the GTK4 event controller callbacks translate GDK events into
  `InputEvent` values and dispatch them through `backend.handle_input(event)`.
- `ArcballCamera` — the camera state is read back from the active backend for UI overlays
  (zoom label, reset button).

## What T17 needs from T10A

- `ViewportBackend` trait — `WgpuBackend` will implement all 9 methods.
- `ArcballCamera` — produces the `view_proj` matrix uploaded to the WGSL uniform buffer.
- `adapter.rs` — `decompose_transform` drives vertex generation for each `SceneObject`.

## Notes for the T14 agent

- The GTK4 app shell lives in `rustsynth_app_gtk`. The crate skeleton already exists.
- Use Relm4 for the component model. The top-level `AppModel` holds a
  `Box<dyn ViewportBackend>` (initially a no-op stub, replaced by wgpu in T17).
- Wire GTK4 motion/scroll/button controllers to `InputEvent` and call
  `backend.handle_input(event)`.
- The editor pane is a `GtkSourceView` (`sourceview5` crate) for EisenScript source.
- File open/save uses `gtk::FileDialog` (GTK 4.10+ async API).
