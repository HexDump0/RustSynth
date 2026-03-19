//! `rustsynth_scene` — canonical renderer-agnostic scene representation.
//!
//! This is the boundary between the evaluation core and all output backends
//! (viewport renderers, template exporters, OBJ exporters).

pub mod adapter;
pub mod camera;
pub mod object;
pub mod primitive;
pub mod scene;

pub use scene::Scene;
