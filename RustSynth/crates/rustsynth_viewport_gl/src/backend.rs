//! Custom OpenGL viewport backend placeholder.
//!
//! Full implementation in T16.

use rustsynth_render_api::ViewportBackend;
use rustsynth_scene::Scene;

pub struct GlBackend;

impl ViewportBackend for GlBackend {
    fn init(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn load_scene(&mut self, _scene: &Scene) -> anyhow::Result<()> { Ok(()) }
    fn render_frame(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn resize(&mut self, _width: u32, _height: u32) {}
    fn shutdown(&mut self) {}
}
