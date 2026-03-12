# T12 — Next

## Recommended next task: T13 — Scripting compatibility decision

T13 is the only remaining `READY` task. It requires no code — only a decision document.

## After T13

**T14** (GTK4 + Relm4 desktop app shell) becomes the main focus. T14 is the largest single
remaining task before v1 is usable.

## Notes for the T14 agent (after T13)

The exporter crates (`rustsynth_export_template`, `rustsynth_export_obj`) are now complete.
T14 should wire them up through an `ExportDialog` component that:
1. Lets the user pick a template file (T11) or choose OBJ (T12).
2. Passes the current `Scene` to the chosen exporter.
3. Writes the output to a file chosen via `gtk::FileDialog`.

The export flow is tracked as T20 (template) and T21 (OBJ), both blocked on T14.

## OBJ exporter configuration surface

`ObjExporter` exposes these fields that the T20/T21 UI should let the user control:
- `sphere_segments: u32` — sphere tessellation quality (default 16).
- `group_by_tag: bool` — group objects by EisenScript class tag.
- `group_by_color: bool` — group objects by RGBA colour (default true).
- `mtl_file_name: String` — base name for the companion `.mtl` file.
