# T19 — Next Steps

## Recommended next task
T20 (Template export UI flow) is also complete in this session.

## Known deferred issues

1. **Camera JSON versioning**: The JSON schema is unversioned. If `ArcballCamera`
   fields change, old `.json` files will fail to deserialize. A future schema
   version field could add forward compatibility.

2. **Camera export via drag-and-drop / clipboard**: Power users might prefer
   Ctrl+C / Ctrl+V to copy camera parameters between scripts. Could be added
   as `AppMsg::CopyCameraToClipboard` / `AppMsg::PasteCameraFromClipboard`.

3. **Camera in settings dialog**: The current camera parameters (yaw, pitch,
   distance, fov_y) could be exposed in the Settings dialog (T19 extension).
   Currently the only way to set exact values is to edit and load a JSON file.

4. **Script annotation stripping**: When exporting/saving, the
   `// @rs-camera: ...` first line could optionally be stripped if the user
   doesn't want it embedded. A toggle in Settings could control this.

## Unanswered questions
- Should `Insert Camera into Script` also auto-save the file, or just mark
  the buffer as modified?  Current behaviour: marks modified, user saves manually.
