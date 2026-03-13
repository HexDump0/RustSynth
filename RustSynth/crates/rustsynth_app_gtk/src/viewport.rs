//! Viewport rendering helpers.
//!
//! Wraps the `WgpuBackend` offscreen render path and converts the resulting
//! pixel buffer into a `gdk::MemoryTexture` ready for display in a
//! `gtk::Picture` widget.
//!
//! The offscreen→CPU readback approach is used here for simplicity.  For
//! realtime performance, replace this with direct GtkGLArea EGL integration
//! (see T17 NEXT.md).

use anyhow::{Context, Result};
use relm4::gtk;
use rustsynth_render_api::backend::ViewportBackend;
use rustsynth_scene::Scene;
use rustsynth_viewport_wgpu::WgpuBackend;

/// Load `scene` into `backend`, render offscreen at `width × height` pixels,
/// and return a GDK texture ready for `gtk::Picture::set_paintable`.
///
/// The texture uses `gdk::MemoryFormat::R8g8b8a8` which matches the
/// `Rgba8Unorm` wgpu surface format set in [`WgpuBackend::new`].
pub fn render_scene_to_texture(
    backend: &mut WgpuBackend,
    scene: &Scene,
    width: u32,
    height: u32,
) -> Result<gtk::gdk::MemoryTexture> {
    // Upload scene geometry to the GPU.
    backend
        .load_scene(scene)
        .context("Failed to upload scene to GPU")?;

    pixels_to_gdk_texture(backend, width, height)
}

/// Re-render the currently loaded scene (no geometry reload) and return a
/// GDK texture.  Used after camera moves where the scene hasn't changed.
pub fn render_scene_to_texture_no_reload(
    backend: &mut WgpuBackend,
    width: u32,
    height: u32,
) -> Result<gtk::gdk::MemoryTexture> {
    pixels_to_gdk_texture(backend, width, height)
}

fn pixels_to_gdk_texture(
    backend: &mut WgpuBackend,
    width: u32,
    height: u32,
) -> Result<gtk::gdk::MemoryTexture> {
    // Render offscreen and read pixels back to CPU.
    let pixels = backend
        .render_to_pixels(width, height)
        .context("Failed to render scene offscreen")?;

    // Wrap pixels in a GDK texture.
    let bytes = gtk::glib::Bytes::from(&pixels);
    let texture = gtk::gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (width * 4) as usize,
    );

    Ok(texture)
}
