//! `rustsynth_viewport_wgpu` — wgpu viewport backend.
//!
//! Implements [`rustsynth_render_api::ViewportBackend`] using `wgpu`, targeting
//! a `GtkGLArea` EGL surface or any wgpu-compatible surface.
//!
//! # Crate layout
//!
//! - [`backend`] — [`WgpuBackend`] struct implementing `ViewportBackend`.
//! - [`geometry`] — mesh generation for boxes, spheres, cylinders and other primitives.
//! - [`gpu_types`] — GPU-side vertex and uniform types (`bytemuck`-compatible).
//! - [`pipeline`] — render pipeline and bind group setup.
//! - [`shader`] — embedded WGSL shader source.

pub mod backend;
pub mod geometry;
pub mod gpu_types;
pub mod pipeline;
pub mod shader;

pub use backend::WgpuBackend;
