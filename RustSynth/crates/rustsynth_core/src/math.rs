//! Math helpers — re-exports from `glam` and supplementary utilities.

pub use glam::{Mat4, Quat, Vec3, Vec4};

/// Degrees to radians helper.
#[inline]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

/// Build a 4×4 rotation matrix around the X axis.
pub fn rotation_x(deg: f32) -> Mat4 {
    Mat4::from_rotation_x(deg_to_rad(deg))
}

/// Build a 4×4 rotation matrix around the Y axis.
pub fn rotation_y(deg: f32) -> Mat4 {
    Mat4::from_rotation_y(deg_to_rad(deg))
}

/// Build a 4×4 rotation matrix around the Z axis.
pub fn rotation_z(deg: f32) -> Mat4 {
    Mat4::from_rotation_z(deg_to_rad(deg))
}
