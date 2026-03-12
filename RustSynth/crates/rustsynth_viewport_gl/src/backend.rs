//! Custom OpenGL viewport backend placeholder.
//!
//! Full implementation in T16.

use rustsynth_render_api::{ArcballCamera, InputEvent, ViewportBackend};
use rustsynth_scene::Scene;

pub struct GlBackend {
    camera: ArcballCamera,
}

impl Default for GlBackend {
    fn default() -> Self {
        Self { camera: ArcballCamera::default() }
    }
}

impl ViewportBackend for GlBackend {
    fn init(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn load_scene(&mut self, _scene: &Scene) -> anyhow::Result<()> { Ok(()) }
    fn render_frame(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn resize(&mut self, _width: u32, _height: u32) {}
    fn shutdown(&mut self) {}
    fn camera(&self) -> &ArcballCamera { &self.camera }
    fn camera_mut(&mut self) -> &mut ArcballCamera { &mut self.camera }
    fn handle_input(&mut self, _event: InputEvent) -> bool { false }
    fn backend_name(&self) -> &'static str { "opengl-stub" }
}
