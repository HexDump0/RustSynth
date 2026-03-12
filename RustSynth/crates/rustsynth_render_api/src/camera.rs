//! Camera interface for viewport backends.
//!
//! Defines the arcball-style orbit camera model used by the `wgpu` viewport
//! backend and shared across any future backends.

use rustsynth_core::math::{Mat4, Vec3};

/// Arcball orbit camera that wraps around a pivot point.
///
/// All angles are in **degrees**. Distance is in world units.
#[derive(Debug, Clone)]
pub struct ArcballCamera {
    /// Point the camera orbits around.
    pub pivot: Vec3,
    /// Horizontal orbit angle around Y axis (degrees).
    pub yaw: f32,
    /// Vertical orbit angle (degrees, clamped ±89°).
    pub pitch: f32,
    /// Distance from pivot to camera eye.
    pub distance: f32,
    /// Vertical field-of-view in degrees.
    pub fov_y: f32,
    /// Viewport aspect ratio (width / height).
    pub aspect: f32,
    /// Near clip plane.
    pub near: f32,
    /// Far clip plane.
    pub far: f32,
}

impl Default for ArcballCamera {
    fn default() -> Self {
        Self {
            pivot: Vec3::ZERO,
            yaw: 30.0,
            pitch: 20.0,
            distance: 5.0,
            fov_y: 45.0,
            aspect: 1.0,
            near: 0.01,
            far: 1000.0,
        }
    }
}

impl ArcballCamera {
    /// Compute the world-space eye position from the orbit parameters.
    pub fn eye(&self) -> Vec3 {
        let pitch_rad = self.pitch.to_radians();
        let yaw_rad = self.yaw.to_radians();
        let r = self.distance;
        let x = r * pitch_rad.cos() * yaw_rad.sin();
        let y = r * pitch_rad.sin();
        let z = r * pitch_rad.cos() * yaw_rad.cos();
        self.pivot + Vec3::new(x, y, z)
    }

    /// Build the view matrix (world-to-camera).
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.pivot, Vec3::Y)
    }

    /// Build the perspective projection matrix.
    pub fn proj_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y.to_radians(), self.aspect, self.near, self.far)
    }

    /// Combined view-projection matrix ready for shaders.
    pub fn view_proj(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    /// Orbit by a delta in yaw/pitch degrees.
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw = (self.yaw + delta_yaw) % 360.0;
        self.pitch = (self.pitch + delta_pitch).clamp(-89.0, 89.0);
    }

    /// Zoom by changing the camera distance (multiplicative).
    pub fn zoom(&mut self, factor: f32) {
        self.distance = (self.distance * factor).max(0.01);
    }

    /// Pan the pivot point in the camera-local XY plane.
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let view = self.view_matrix();
        let right = Vec3::new(view.x_axis.x, view.y_axis.x, view.z_axis.x);
        let up = Vec3::new(view.x_axis.y, view.y_axis.y, view.z_axis.y);
        self.pivot += right * delta_x + up * delta_y;
    }

    /// Reset the camera to its default pose.
    pub fn reset(&mut self) {
        *self = Self {
            aspect: self.aspect,
            ..Self::default()
        };
    }
}
