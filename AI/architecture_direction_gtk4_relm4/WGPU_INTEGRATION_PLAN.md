# Improved wgpu Integration Plan

The current implementation uses `render_scene_to_texture` (offscreen rendering + CPU readback). This is robust but inefficient for high-resolution real-time rendering.

## The "Better Way": GtkGLArea + Shared Texture

To achieve zero-copy rendering with `wgpu` inside GTK4, we should use a `GtkGLArea` and export the `wgpu` render result as a GL texture, or allow `wgpu` to render directly to the GL context.

### Strategy A: EGL/GL Context Sharing (Recommended)

1. **Backend Update**: Modify `rustsynth_viewport_wgpu::WgpuBackend` to accept an `Option<wgpu::Instance>`.
2. **GLArea Widget**:
   - Create a `gtk::GLArea`.
   - In `realize` signal:
     - Get the `gdk::GLContext`.
     - Initialize `wgpu::Instance` with `Backends::GL`.
     - Create a `wgpu::Surface` targeting the generic window? No, `wgpu` on GL usually checks for current context.
   - In `render` signal:
     - `wgpu` performs render pass.
     - `wgpu` presents.

### Strategy B: Shared Texture (DMA-BUF)

1. **Offscreen Render**: `WgpuBackend` renders to a texture.
2. **Export**: Use `wgpu` extension or `hal` to export the texture handle (DMABUF on Linux).
3. **Import in GTK**: Use `gdk::GLTexture::new_from_builder` or `gdk::DmabufTextureBuilder` (GTK 4.14+).
4. **Display**: Set this texture on a `gtk::Picture`.

## Immediate UI Improvements (Implemented)

- **HeaderBar Controls**: Moved rendering parameters (Seed, Max Objects, Mode) to the HeaderBar to save vertical space.
- **Viewport Overlay**: Removed `ScrolledWindow` from the viewport. The 3D view now fills the specialized `gtk::Overlay`, allowing for future HUD/status indicators on top of the render.
- **Content Fit**: Used `ContentFit::Fill` to ensure the render matches the widget size without scrollbars.

## Future Code Changes

1. Create `crates/rustsynth_app_gtk/src/widgets/wgpu_area.rs`.
2. Implement `WgpuArea` struct wrapping `gtk::GLArea`.
3. Use `pollster::block_on` if async `wgpu` features are needed (though `wgpu` core is sync on native).
