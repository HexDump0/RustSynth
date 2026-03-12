//! Bevy viewport backend placeholder.
//!
//! Full implementation in T15.

use rustsynth_render_api::ViewportBackend;
use rustsynth_scene::Scene;

pub struct BevyBackend;

impl ViewportBackend for BevyBackend {
    fn init(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn load_scene(&mut self, _scene: &Scene) -> anyhow::Result<()> { Ok(()) }
    fn render_frame(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn resize(&mut self, _width: u32, _height: u32) {}
    fn shutdown(&mut self) {}
}
