//! Canonical scene — a flat list of positioned, coloured primitive instances.
//!
//! This is the boundary type between the evaluator and all output backends
//! (viewport renderers, template exporters, OBJ exporters).

use serde::{Deserialize, Serialize};

use crate::camera::CameraState;
use crate::object::SceneObject;

/// The renderer-agnostic scene produced by the evaluator.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// All emitted primitive instances in emission order.
    pub objects: Vec<SceneObject>,
    /// Camera state from `set translation/rotation/pivot/scale` commands.
    pub camera: Option<CameraState>,
    /// Background colour from `set background`.
    pub background: Option<rustsynth_core::color::Rgba>,
    /// Pass-through settings not handled by the core evaluator
    /// (e.g. `raytracer::shadows false`, `template mytemplate`).
    /// Format: `(key, value)` preserving source order.
    pub raw_settings: Vec<(String, String)>,
}
