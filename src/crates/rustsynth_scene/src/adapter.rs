//! Adapter helpers shared by multiple viewport backends.
//!
//! These utilities sit between the renderer-agnostic `rustsynth_scene` types
//! and the raw data formats that GPU backends consume.

use rustsynth_core::math::{Mat4, Vec3};
use crate::object::SceneObject;

/// Decompose a `SceneObject` transform into its three local-axis vectors and
/// origin, mirroring how the legacy C++ renderers unpacked the matrix.
///
/// The transform is a column-major 4×4 (glam convention):
/// ```text
/// [ col0  col1  col2  col3 ]
///   dir1  dir2  dir3  base
/// ```
/// where `dir1/dir2/dir3` are the three axis columns (unit vectors times
/// scale) and `base` is the translation column.
pub fn decompose_transform(obj: &SceneObject) -> (Vec3, Vec3, Vec3, Vec3) {
    let m = obj.transform;
    let dir1 = Vec3::new(m.x_axis.x, m.x_axis.y, m.x_axis.z);
    let dir2 = Vec3::new(m.y_axis.x, m.y_axis.y, m.y_axis.z);
    let dir3 = Vec3::new(m.z_axis.x, m.z_axis.y, m.z_axis.z);
    let base = Vec3::new(m.w_axis.x, m.w_axis.y, m.w_axis.z);
    (base, dir1, dir2, dir3)
}

/// Pack a `Mat4` into a flat `[f32; 16]` array in column-major order,
/// suitable for uploading to a GPU uniform buffer.
pub fn mat4_to_array(m: Mat4) -> [f32; 16] {
    m.to_cols_array()
}

/// Build a row-major string representation of a 4×4 matrix for template
/// substitution (legacy `{matrix}` placeholder format):
/// `"d1x d1y d1z 0 d2x d2y d2z 0 d3x d3y d3z 0 bx by bz 1"`
pub fn matrix_row_str(base: Vec3, dir1: Vec3, dir2: Vec3, dir3: Vec3) -> String {
    format!(
        "{} {} {} 0 {} {} {} 0 {} {} {} 0 {} {} {} 1",
        dir1.x, dir1.y, dir1.z,
        dir2.x, dir2.y, dir2.z,
        dir3.x, dir3.y, dir3.z,
        base.x, base.y, base.z,
    )
}

/// Build a column-major string representation (`{columnmatrix}` placeholder):
/// `"d1x d2x d3x bx d1y d2y d3y by d1z d2z d3z bz 0 0 0 1"`
pub fn column_matrix_str(base: Vec3, dir1: Vec3, dir2: Vec3, dir3: Vec3) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {} {} {} 0 0 0 1",
        dir1.x, dir2.x, dir3.x, base.x,
        dir1.y, dir2.y, dir3.y, base.y,
        dir1.z, dir2.z, dir3.z, base.z,
    )
}

/// Build a POV-Ray compatible matrix string (`{povmatrix}` placeholder):
/// `"d1x, d1y, d1z, d2x, d2y, d2z, d3x, d3y, d3z, bx, by, bz"`
pub fn pov_matrix_str(base: Vec3, dir1: Vec3, dir2: Vec3, dir3: Vec3) -> String {
    format!(
        "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
        dir1.x, dir1.y, dir1.z,
        dir2.x, dir2.y, dir2.z,
        dir3.x, dir3.y, dir3.z,
        base.x, base.y, base.z,
    )
}

/// Build the camera column matrix string used in `{CamColumnMatrix}`.
///
/// u = -right, v = up, w = -dir (right-handed, into-scene).
pub fn cam_column_matrix_str(
    position: Vec3,
    right: Vec3,
    up: Vec3,
    dir: Vec3,
) -> String {
    let u = -right;
    let v = up;
    let w = -dir;
    format!(
        "{} {} {} {} {} {} {} {} {} {} {} {} 0.0 0.0 0.0 1.0",
        u.x, v.x, w.x, position.x,
        u.y, v.y, w.y, position.y,
        u.z, v.z, w.z, position.z,
    )
}

/// Extract the center and radius of a sphere from a `SceneObject` transform.
///
/// In the legacy pipeline the sphere is a unit sphere scaled and translated
/// via the same 4×4 transform used for all primitives. The radius is the
/// average of the three axis lengths (the transform should be isotropic for
/// spheres, but we average defensively).
pub fn sphere_center_radius(obj: &SceneObject) -> (Vec3, f32) {
    let m = &obj.transform;
    let center = Vec3::new(m.w_axis.x, m.w_axis.y, m.w_axis.z);
    let sx = Vec3::new(m.x_axis.x, m.x_axis.y, m.x_axis.z).length();
    let sy = Vec3::new(m.y_axis.x, m.y_axis.y, m.y_axis.z).length();
    let sz = Vec3::new(m.z_axis.x, m.z_axis.y, m.z_axis.z).length();
    let radius = (sx + sy + sz) / 3.0;
    (center, radius)
}

/// Extract the two end points and radius of a cylinder from a `SceneObject`
/// transform.
///
/// Legacy convention: the cylinder runs from `base` to `base + dir3`, with
/// radius = length of dir1 (= dir2, assumed equal).
pub fn cylinder_endpoints(obj: &SceneObject) -> (Vec3, Vec3, f32) {
    let (_base, dir1, _dir2, dir3) = decompose_transform(obj);
    let (base, _, _, _) = decompose_transform(obj);
    let top = base + dir3;
    let radius = dir1.length();
    (base, top, radius)
}
