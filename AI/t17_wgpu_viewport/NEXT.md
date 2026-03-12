# T17 — Next steps

## Immediate follow-ups

1. **T14 — GTK4 app shell**: Wire `WgpuBackend` into `GtkGLArea`:
   - On `realize`: call `backend.init()`
   - On `render`: call `backend.render_to_view(&view)`
   - On `resize`: call `backend.resize(w, h)`
   - On `unrealize`: call `backend.shutdown()`
   - Route GTK4 event controller signals through `backend.handle_input()`

2. **Scene loading flow**: After evaluator produces a `Scene`, call `backend.load_scene(&scene)` then `gtk_gl_area.queue_render()`.

3. **Background colour**: The backend reads `scene.background` in `load_scene`. The GTK shell should also allow UI-driven background changes via `set_clear_color`.

## Future enhancements (not blocking v1)

- **Anti-aliasing**: Enable MSAA (multisample) in the pipeline
- **Transparency sorting**: Currently uses alpha blending but no depth-sorted draw order for translucent objects
- **Wireframe mode**: Add a secondary pipeline with `PolygonMode::Line`
- **Screenshot**: Render to an offscreen texture and read back pixels
- **Per-object selection / picking**: Ray-cast or colour-ID pass for interactive selection
