//! Camera state serialized from/to EisenScript `set` commands.

use serde::{Deserialize, Serialize};
use rustsynth_core::math::{Mat4, Vec3};

/// Camera state matching the legacy `set translation/rotation/pivot/scale` format.
///
/// Stored as plain arrays for serde compatibility. Use the conversion helpers
/// to get `glam` types for computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraState {
    /// [x, y, z]
    pub translation: [f32; 3],
    /// Column-major 4×4 matrix as 16 floats
    pub rotation: [f32; 16],
    /// [x, y, z]
    pub pivot: [f32; 3],
    pub scale: f32,
}

impl CameraState {
    pub fn translation_vec3(&self) -> Vec3 {
        Vec3::from_array(self.translation)
    }

    pub fn rotation_mat4(&self) -> Mat4 {
        Mat4::from_cols_array(&self.rotation)
    }

    pub fn pivot_vec3(&self) -> Vec3 {
        Vec3::from_array(self.pivot)
    }
}
