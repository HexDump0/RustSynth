//! `rustsynth_render_api` — renderer boundary traits and viewport-facing contracts.
//!
//! Any viewport backend (wgpu, OpenGL, Bevy) must implement the traits defined
//! here. The app shell drives all backends through this API.
//!
//! # Crate layout
//!
//! - [`backend`] — [`ViewportBackend`] trait and [`InputEvent`] types.
//! - [`camera`] — [`ArcballCamera`] orbit camera model.
//! - [`adapter`] — geometry helpers for unpacking `SceneObject` transforms.

pub mod adapter;
pub mod backend;
pub mod camera;

pub use backend::{InputEvent, PointerButton, ViewportBackend};
pub use camera::ArcballCamera;
