//! `rustsynth_render_api` — renderer boundary traits and viewport-facing contracts.
//!
//! Any viewport backend (Bevy, OpenGL, wgpu) must implement the traits defined
//! here. The app shell drives all backends through this API.

pub mod adapter;
pub mod backend;
pub mod camera;

pub use backend::ViewportBackend;
