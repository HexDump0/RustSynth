//! Transform application — apply `TransformOp` values to the eval `State`.
//!
//! The matrix conventions match the legacy `Transformation.cpp`:
//! - Rotations are about the centre of the unit cube `(0.5, 0.5, 0.5)`.
//! - Scales are about the same centre.
//! - Colour transforms compose multiplicatively (except absolute `color`).

use rustsynth_core::color::{Hsva, Rgba};
use rustsynth_core::math::{Mat3, Mat4, Vec3, Vec4};
use rustsynth_eisenscript::ast::TransformOp;
use rustsynth_core::rng::Rng;

use crate::state::State;

// ─── public entry points ──────────────────────────────────────────────────────

/// Apply a slice of `TransformOp`s to a state in order and return the result.
pub fn apply_transforms(ops: &[TransformOp], state: &State, rng: &mut Rng) -> State {
    let mut s = state.clone();
    for op in ops {
        s = apply_one(op, s, rng);
    }
    s
}

/// Apply a single `TransformOp` to a state and return the result.
pub fn apply_one(op: &TransformOp, mut s: State, rng: &mut Rng) -> State {
    match op {
        // ── Spatial transforms ───────────────────────────────────────────
        TransformOp::X(d) => {
            s.transform = s.transform * Mat4::from_translation(Vec3::new(*d as f32, 0.0, 0.0));
        }
        TransformOp::Y(d) => {
            s.transform = s.transform * Mat4::from_translation(Vec3::new(0.0, *d as f32, 0.0));
        }
        TransformOp::Z(d) => {
            s.transform = s.transform * Mat4::from_translation(Vec3::new(0.0, 0.0, *d as f32));
        }

        // Rotation about the centre of the unit cube (0.5, 0.5, 0.5).
        // Legacy:  T(pivot) * Rot * T(-pivot)
        TransformOp::Rx(deg) => {
            s.transform = s.transform * rotate_about_pivot(
                Vec3::new(0.0, 0.5, 0.5),
                Vec3::X,
                *deg as f32,
            );
        }
        TransformOp::Ry(deg) => {
            s.transform = s.transform * rotate_about_pivot(
                Vec3::new(0.5, 0.0, 0.5),
                Vec3::Y,
                *deg as f32,
            );
        }
        TransformOp::Rz(deg) => {
            s.transform = s.transform * rotate_about_pivot(
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::Z,
                *deg as f32,
            );
        }

        // Scale about the centre of the unit cube.
        TransformOp::S { x, y, z } => {
            s.transform = s.transform * scale_about_center(*x as f32, *y as f32, *z as f32);
        }
        TransformOp::Fx => {
            s.transform = s.transform * scale_about_center(-1.0, 1.0, 1.0);
        }
        TransformOp::Fy => {
            s.transform = s.transform * scale_about_center(1.0, -1.0, 1.0);
        }
        TransformOp::Fz => {
            s.transform = s.transform * scale_about_center(1.0, 1.0, -1.0);
        }

        TransformOp::Reflect { nx, ny, nz } => {
            let n = Vec3::new(*nx as f32, *ny as f32, *nz as f32).normalize();
            s.transform = s.transform * plane_reflection(n);
        }

        // Arbitrary 3×3 matrix embedded in 4×4, applied about centre.
        // Row-major input → column-major glam matrix.
        TransformOp::Matrix(vals) => {
            let m3 = Mat3::from_cols(
                Vec3::new(vals[0] as f32, vals[3] as f32, vals[6] as f32),
                Vec3::new(vals[1] as f32, vals[4] as f32, vals[7] as f32),
                Vec3::new(vals[2] as f32, vals[5] as f32, vals[8] as f32),
            );
            let m4 = Mat4::from_mat3(m3);
            s.transform = s.transform * scale_about_center_matrix(m4);
        }

        // ── Colour transforms ────────────────────────────────────────────
        TransformOp::Hue(dh) => {
            let mut h = s.color.h + *dh as f32;
            while h >= 360.0 { h -= 360.0; }
            while h < 0.0    { h += 360.0; }
            s.color.h = h;
        }
        TransformOp::Sat(scale) => {
            s.color.s = (s.color.s * *scale as f32).clamp(0.0, 1.0);
        }
        TransformOp::Brightness(scale) => {
            s.color.v = (s.color.v * *scale as f32).clamp(0.0, 1.0);
        }
        TransformOp::Alpha(scale) => {
            s.color.a = (s.color.a * *scale as f32).clamp(0.0, 1.0);
        }

        // Absolute colour — replaces the current colour entirely.
        TransformOp::Color(color_str) => {
            if color_str == "random" {
                // Random hue, full saturation and value.
                let h = rng.next_f64() * 360.0;
                s.color = Hsva::new(h as f32, 1.0, 1.0, 1.0);
            } else if let Some(rgba) = Rgba::from_hex(color_str) {
                s.color = rgba_to_hsva(rgba);
            } else if let Some(rgba) = named_color(color_str) {
                s.color = rgba_to_hsva(rgba);
            }
            // Unknown colour string: leave unchanged (no crash).
        }

        // Blend current colour towards target in HSV space.
        TransformOp::Blend { color: color_str, strength } => {
            let blend_hsva = if let Some(rgba) = Rgba::from_hex(color_str) {
                rgba_to_hsva(rgba)
            } else if let Some(rgba) = named_color(color_str) {
                rgba_to_hsva(rgba)
            } else {
                return s; // unknown colour
            };
            let str = *strength as f32;
            let denom = 1.0 + str;
            let mut h = (s.color.h + str * blend_hsva.h) / denom;
            let mut sat = (s.color.s + str * blend_hsva.s) / denom;
            let mut v = (s.color.v + str * blend_hsva.v) / denom;
            while h < 0.0   { h += 360.0; }
            while h > 360.0 { h -= 360.0; }
            sat = sat.clamp(0.0, 1.0);
            v   = v.clamp(0.0, 1.0);
            s.color.h = h;
            s.color.s = sat;
            s.color.v = v;
        }
    }
    s
}

// ─── matrix helpers ───────────────────────────────────────────────────────────

/// Rotation about an arbitrary pivot point and axis.
fn rotate_about_pivot(pivot: Vec3, axis: Vec3, deg: f32) -> Mat4 {
    let rad = deg.to_radians();
    let to_origin = Mat4::from_translation(-pivot);
    let rot = Mat4::from_axis_angle(axis, rad);
    let from_origin = Mat4::from_translation(pivot);
    from_origin * rot * to_origin
}

/// Uniform or non-uniform scale about the centre of the unit cube (0.5, 0.5, 0.5).
fn scale_about_center(sx: f32, sy: f32, sz: f32) -> Mat4 {
    let pivot = Vec3::splat(0.5);
    let to_origin = Mat4::from_translation(-pivot);
    let scale = Mat4::from_scale(Vec3::new(sx, sy, sz));
    let from_origin = Mat4::from_translation(pivot);
    from_origin * scale * to_origin
}

/// Apply an arbitrary 4×4 matrix about the centre of the unit cube.
fn scale_about_center_matrix(m: Mat4) -> Mat4 {
    let pivot = Vec3::splat(0.5);
    let to_origin = Mat4::from_translation(-pivot);
    let from_origin = Mat4::from_translation(pivot);
    from_origin * m * to_origin
}

/// Householder reflection about a plane through the origin with the given normal.
fn plane_reflection(n: Vec3) -> Mat4 {
    // R = I - 2 * n * nᵀ
    let xx = 1.0 - 2.0 * n.x * n.x;
    let yy = 1.0 - 2.0 * n.y * n.y;
    let zz = 1.0 - 2.0 * n.z * n.z;
    let xy = -2.0 * n.x * n.y;
    let xz = -2.0 * n.x * n.z;
    let yz = -2.0 * n.y * n.z;
    // glam Mat4::from_cols is column-major
    Mat4::from_cols(
        Vec4::new(xx, xy, xz, 0.0),
        Vec4::new(xy, yy, yz, 0.0),
        Vec4::new(xz, yz, zz, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    )
}

// ─── colour helpers ───────────────────────────────────────────────────────────

fn rgba_to_hsva(rgba: Rgba) -> Hsva {
    let r = rgba.r;
    let g = rgba.g;
    let b = rgba.b;
    let a = rgba.a;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max < 1e-6 { 0.0 } else { delta / max };
    let h = if delta < 1e-6 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    Hsva::new(h, s, v, a)
}

/// A minimal lookup for a handful of common SVG/CSS named colours.
fn named_color(name: &str) -> Option<Rgba> {
    match name {
        "white"   => Some(Rgba::new(1.0, 1.0, 1.0, 1.0)),
        "black"   => Some(Rgba::new(0.0, 0.0, 0.0, 1.0)),
        "red"     => Some(Rgba::new(1.0, 0.0, 0.0, 1.0)),
        "green"   => Some(Rgba::new(0.0, 0.502, 0.0, 1.0)),
        "blue"    => Some(Rgba::new(0.0, 0.0, 1.0, 1.0)),
        "yellow"  => Some(Rgba::new(1.0, 1.0, 0.0, 1.0)),
        "cyan"    => Some(Rgba::new(0.0, 1.0, 1.0, 1.0)),
        "magenta" => Some(Rgba::new(1.0, 0.0, 1.0, 1.0)),
        "orange"  => Some(Rgba::new(1.0, 0.647, 0.0, 1.0)),
        _ => None,
    }
}

// ─── tests (T09) ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rustsynth_core::math::Vec3;

    fn default_state() -> State {
        State::default()
    }

    fn rng() -> Rng {
        Rng::new(42)
    }

    // ── Translation ───────────────────────────────────────────────────────

    #[test]
    fn translate_x_moves_origin() {
        let s = apply_one(&TransformOp::X(3.0), default_state(), &mut rng());
        let translated = s.transform.transform_point3(Vec3::ZERO);
        assert!((translated.x - 3.0).abs() < 1e-5);
        assert!(translated.y.abs() < 1e-5);
        assert!(translated.z.abs() < 1e-5);
    }

    #[test]
    fn translate_y_moves_origin() {
        let s = apply_one(&TransformOp::Y(2.5), default_state(), &mut rng());
        let translated = s.transform.transform_point3(Vec3::ZERO);
        assert!(translated.x.abs() < 1e-5);
        assert!((translated.y - 2.5).abs() < 1e-5);
    }

    #[test]
    fn translate_z_moves_origin() {
        let s = apply_one(&TransformOp::Z(-1.0), default_state(), &mut rng());
        let translated = s.transform.transform_point3(Vec3::ZERO);
        assert!((translated.z - -1.0).abs() < 1e-5);
    }

    // ── Scale ─────────────────────────────────────────────────────────────

    #[test]
    fn uniform_scale_halves_unit_cube_centre() {
        // The unit cube centre at (0.5,0.5,0.5) must stay fixed under any
        // scale-about-centre transform.
        let s = apply_one(&TransformOp::S { x: 0.5, y: 0.5, z: 0.5 }, default_state(), &mut rng());
        let centre = s.transform.transform_point3(Vec3::splat(0.5));
        assert!((centre.x - 0.5).abs() < 1e-5);
        assert!((centre.y - 0.5).abs() < 1e-5);
        assert!((centre.z - 0.5).abs() < 1e-5);
    }

    #[test]
    fn flip_x_reflects_about_yz_plane_through_centre() {
        let s = apply_one(&TransformOp::Fx, default_state(), &mut rng());
        // Point (0, 0.5, 0.5) should map to (1, 0.5, 0.5) after fx (mirror through x=0.5).
        let p = s.transform.transform_point3(Vec3::new(0.0, 0.5, 0.5));
        assert!((p.x - 1.0).abs() < 1e-5, "x should be mirrored to 1.0, got {}", p.x);
        assert!((p.y - 0.5).abs() < 1e-5);
        assert!((p.z - 0.5).abs() < 1e-5);
    }

    // ── Rotation ─────────────────────────────────────────────────────────

    #[test]
    fn rz_90_rotates_corner_about_centre() {
        // With a 90° Z rotation about (0.5, 0.5, 0):
        // point (1.0, 0.5, 0.0) → (0.5, 1.0, 0.0)  [approximately]
        let s = apply_one(&TransformOp::Rz(90.0), default_state(), &mut rng());
        let p = s.transform.transform_point3(Vec3::new(1.0, 0.5, 0.0));
        assert!((p.x - 0.5).abs() < 1e-5, "x after rz90 = {}", p.x);
        assert!((p.y - 1.0).abs() < 1e-5, "y after rz90 = {}", p.y);
    }

    #[test]
    fn rz_identity_after_360() {
        let s = apply_one(&TransformOp::Rz(360.0), default_state(), &mut rng());
        let p = s.transform.transform_point3(Vec3::new(0.3, 0.7, 0.2));
        assert!((p.x - 0.3).abs() < 1e-4);
        assert!((p.y - 0.7).abs() < 1e-4);
        assert!((p.z - 0.2).abs() < 1e-4);
    }

    // ── HSV colour ────────────────────────────────────────────────────────

    #[test]
    fn hue_shift_wraps_around_360() {
        let s = apply_one(&TransformOp::Hue(350.0), default_state(), &mut rng());
        // default h=0 + 350 = 350
        assert!((s.color.h - 350.0).abs() < 1e-4);
        // Shift by another 20 → should wrap to 10
        let s2 = apply_one(&TransformOp::Hue(20.0), s, &mut rng());
        assert!((s2.color.h - 10.0).abs() < 1e-4, "wrapped hue = {}", s2.color.h);
    }

    #[test]
    fn saturation_scale_clamps_to_zero() {
        let s = apply_one(&TransformOp::Sat(0.0), default_state(), &mut rng());
        assert!(s.color.s.abs() < 1e-6);
    }

    #[test]
    fn brightness_scale_halves() {
        let s = apply_one(&TransformOp::Brightness(0.5), default_state(), &mut rng());
        assert!((s.color.v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn alpha_scale_reduces() {
        let s = apply_one(&TransformOp::Alpha(0.9), default_state(), &mut rng());
        assert!((s.color.a - 0.9).abs() < 1e-5);
    }

    #[test]
    fn absolute_color_hex_sets_hsva() {
        // #ff0000 is pure red → h≈0, s=1, v=1
        let s = apply_one(
            &TransformOp::Color("#ff0000".to_owned()),
            default_state(),
            &mut rng(),
        );
        assert!(s.color.s > 0.99, "saturation should be 1.0, got {}", s.color.s);
        assert!(s.color.v > 0.99, "value should be 1.0, got {}", s.color.v);
    }

    #[test]
    fn random_color_changes_state() {
        // Two consecutive `Color("random")` calls on the same advancing RNG must
        // produce different hues.  xorshift64 never emits two identical consecutive
        // values from a non-zero state, so this assertion always holds.
        let mut r = Rng::new(1234);
        let s1 = apply_one(&TransformOp::Color("random".to_owned()), default_state(), &mut r);
        let s2 = apply_one(&TransformOp::Color("random".to_owned()), default_state(), &mut r);
        assert!(
            (s1.color.h - s2.color.h).abs() > 1e-3,
            "consecutive random colours should differ: {} vs {}",
            s1.color.h,
            s2.color.h
        );
    }

    // ── Matrix ────────────────────────────────────────────────────────────

    #[test]
    fn identity_matrix_is_noop() {
        #[rustfmt::skip]
        let identity = TransformOp::Matrix([
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ]);
        let s = apply_one(&identity, default_state(), &mut rng());
        let p = s.transform.transform_point3(Vec3::new(0.3, 0.6, 0.1));
        assert!((p.x - 0.3).abs() < 1e-5);
        assert!((p.y - 0.6).abs() < 1e-5);
        assert!((p.z - 0.1).abs() < 1e-5);
    }
}

