//! `ViewportBackend` — the trait every viewport backend must implement.
//!
//! Placeholder — full interface defined in T10A.

use rustsynth_scene::Scene;

/// Trait for a realtime viewport backend.
///
/// The app shell creates a backend, feeds it scenes, and calls update
/// in a render loop. The backend owns its own rendering resources.
pub trait ViewportBackend {
    /// Initialize the backend, creating rendering resources.
    fn init(&mut self) -> anyhow::Result<()>;

    /// Load a new scene, replacing whatever was previously displayed.
    fn load_scene(&mut self, scene: &Scene) -> anyhow::Result<()>;

    /// Render a single frame.
    fn render_frame(&mut self) -> anyhow::Result<()>;

    /// Called when the viewport is resized.
    fn resize(&mut self, width: u32, height: u32);

    /// Clean up all rendering resources.
    fn shutdown(&mut self);
}
