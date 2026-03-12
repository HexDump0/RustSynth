//! Canonical scene — a flat list of positioned, colored primitive instances.
//!
//! Placeholder — full scene model defined in T10.

use crate::object::SceneObject;
use crate::camera::CameraState;

/// The renderer-agnostic scene produced by the evaluator.
#[derive(Debug, Default, Clone)]
pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub camera: Option<CameraState>,
    pub background: Option<rustsynth_core::color::Rgba>,
}
