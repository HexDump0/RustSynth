//! A single object instance in the canonical scene.

use rustsynth_core::color::Rgba;
use rustsynth_core::math::Mat4;
use crate::primitive::PrimitiveKind;

/// One instantiated primitive with full world transform and material info.
#[derive(Debug, Clone)]
pub struct SceneObject {
    pub kind: PrimitiveKind,
    pub transform: Mat4,
    pub color: Rgba,
    pub alpha: f32,
    /// Optional tag (e.g. `box::metal` → tag = "metal").
    pub tag: Option<String>,
}
