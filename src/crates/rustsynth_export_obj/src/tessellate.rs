//! Sphere tessellation helpers for the OBJ exporter.
//!
//! Generates a UV sphere as a list of quad and triangle faces, matching the
//! algorithm in the legacy `ObjRenderer.cpp` `CreateUnitSphere` function.
//!
//! The sphere is a **unit sphere centred at the origin** before the world
//! transform is applied.

use rustsynth_core::math::Vec3;

/// A vertex + normal index pair (1-based, as per OBJ convention).
#[derive(Debug, Clone, Copy)]
pub struct VertexNormal {
    /// 1-based vertex index.
    pub v: usize,
    /// 1-based normal index, or 0 if no normal.
    pub n: usize,
}

/// A face is a list of `VertexNormal` indices.
pub type Face = Vec<VertexNormal>;

/// An OBJ geometry group: named collection of vertices, normals, and faces.
#[derive(Debug, Default, Clone)]
pub struct ObjGroup {
    pub name: String,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Face>,
}

impl ObjGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Merge another group's geometry into this one, rebasing indices.
    pub fn merge(&mut self, other: ObjGroup) {
        let v_offset = self.vertices.len();
        let n_offset = self.normals.len();

        self.vertices.extend(other.vertices);
        self.normals.extend(other.normals);

        for face in other.faces {
            let rebased: Face = face
                .into_iter()
                .map(|vn| VertexNormal {
                    v: vn.v + v_offset,
                    n: if vn.n == 0 { 0 } else { vn.n + n_offset },
                })
                .collect();
            self.faces.push(rebased);
        }
    }

    /// Deduplicate vertices and normals (reduces output file size).
    pub fn deduplicate(&mut self) {
        // Deduplicate vertices.
        let mut new_verts: Vec<Vec3> = Vec::new();
        let mut v_remap: Vec<usize> = Vec::with_capacity(self.vertices.len());
        for v in &self.vertices {
            let pos = new_verts.iter().position(|u| vec3_approx_eq(*u, *v));
            if let Some(i) = pos {
                v_remap.push(i + 1); // 1-based
            } else {
                new_verts.push(*v);
                v_remap.push(new_verts.len()); // 1-based
            }
        }

        // Deduplicate normals.
        let mut new_normals: Vec<Vec3> = Vec::new();
        let mut n_remap: Vec<usize> = Vec::with_capacity(self.normals.len());
        for n in &self.normals {
            let pos = new_normals.iter().position(|u| vec3_approx_eq(*u, *n));
            if let Some(i) = pos {
                n_remap.push(i + 1);
            } else {
                new_normals.push(*n);
                n_remap.push(new_normals.len());
            }
        }

        // Remap face indices.
        for face in &mut self.faces {
            for vn in face.iter_mut() {
                if vn.v > 0 && vn.v <= v_remap.len() {
                    vn.v = v_remap[vn.v - 1];
                }
                if vn.n > 0 && vn.n <= n_remap.len() {
                    vn.n = n_remap[vn.n - 1];
                }
            }
        }

        self.vertices = new_verts;
        self.normals = new_normals;
    }
}

fn vec3_approx_eq(a: Vec3, b: Vec3) -> bool {
    (a - b).length_squared() < 1e-8
}

// ── Box tessellation ──────────────────────────────────────────────────────────

/// Add a quad face (4 vertices) with a shared face normal.
pub fn add_quad(group: &mut ObjGroup, v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) {
    let vi = group.vertices.len() + 1; // 1-based start index
    let ni = group.normals.len() + 1;

    group.vertices.push(v1);
    group.vertices.push(v2);
    group.vertices.push(v3);
    group.vertices.push(v4);

    let normal = (v2 - v1).cross(v4 - v1).normalize();
    group.normals.push(normal);
    group.normals.push(normal);
    group.normals.push(normal);
    group.normals.push(normal);

    let face: Face = (0..4)
        .map(|i| VertexNormal { v: vi + i, n: ni + i })
        .collect();
    group.faces.push(face);
}

/// Add a line edge (2 vertices, no normals) — used for `grid` primitive.
pub fn add_line_quad(group: &mut ObjGroup, v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) {
    let vi = group.vertices.len() + 1;

    group.vertices.push(v1);
    group.vertices.push(v2);
    group.vertices.push(v3);
    group.vertices.push(v4);

    for i in 0..4 {
        let face: Face = vec![
            VertexNormal { v: vi + i, n: 0 },
            VertexNormal { v: vi + (i + 1) % 4, n: 0 },
        ];
        group.faces.push(face);
    }
}

/// Tessellate a box primitive into 6 quads.
///
/// `origin` is the corner, `dir1/dir2/dir3` are the three edge vectors (may
/// be non-unit — they carry the scale).
pub fn tessellate_box(origin: Vec3, dir1: Vec3, dir2: Vec3, dir3: Vec3) -> ObjGroup {
    let mut g = ObjGroup::default();
    let o = origin;
    // 6 faces of the box.
    add_quad(&mut g, o, o + dir2, o + dir2 + dir1, o + dir1);
    add_quad(&mut g, o + dir3, o + dir1 + dir3, o + dir2 + dir1 + dir3, o + dir2 + dir3);
    add_quad(&mut g, o, o + dir3, o + dir3 + dir2, o + dir2);
    add_quad(&mut g, o + dir1, o + dir2 + dir1, o + dir3 + dir2 + dir1, o + dir3 + dir1);
    add_quad(&mut g, o, o + dir1, o + dir3 + dir1, o + dir3);
    add_quad(&mut g, o + dir2, o + dir3 + dir2, o + dir3 + dir2 + dir1, o + dir1 + dir2);
    g.deduplicate();
    g
}

/// Tessellate a grid primitive into line-quads (wireframe).
pub fn tessellate_grid(origin: Vec3, dir1: Vec3, dir2: Vec3, dir3: Vec3) -> ObjGroup {
    let mut g = ObjGroup::default();
    let o = origin;
    add_line_quad(&mut g, o, o + dir2, o + dir2 + dir1, o + dir1);
    add_line_quad(&mut g, o + dir3, o + dir1 + dir3, o + dir2 + dir1 + dir3, o + dir2 + dir3);
    add_line_quad(&mut g, o, o + dir3, o + dir3 + dir2, o + dir2);
    add_line_quad(&mut g, o + dir1, o + dir2 + dir1, o + dir3 + dir2 + dir1, o + dir3 + dir1);
    add_line_quad(&mut g, o, o + dir1, o + dir3 + dir1, o + dir3);
    add_line_quad(&mut g, o + dir2, o + dir3 + dir2, o + dir3 + dir2 + dir1, o + dir1 + dir2);
    g.deduplicate();
    g
}

// ── Sphere tessellation ───────────────────────────────────────────────────────

/// Tessellate a unit sphere using a UV grid (`dt` latitude bands, `dp`
/// longitude bands), then apply `transform` to each vertex.
///
/// Matches the legacy `CreateUnitSphere(dt, dp, …, m)` algorithm.
pub fn tessellate_sphere(dt: u32, dp: u32, transform: rustsynth_core::math::Mat4) -> ObjGroup {
    let mut g = ObjGroup::default();
    let dtheta = 180.0_f32 / dt as f32;
    let dphi = 360.0_f32 / dp as f32;

    for i in 0..dt {
        let theta = -90.0 + (i as f32 * 180.0 / dt as f32);
        for j in 0..dp {
            let phi = j as f32 * 360.0 / dp as f32;
            let theta_r = theta.to_radians();
            let phi_r = phi.to_radians();
            let theta2_r = (theta + dtheta).to_radians();
            let phi2_r = (phi + dphi).to_radians();

            // Raw unit-sphere positions (pre-transform).
            let p0 = Vec3::new(
                theta_r.cos() * phi_r.cos(),
                theta_r.cos() * phi_r.sin(),
                theta_r.sin(),
            );
            let p1 = Vec3::new(
                theta_r.cos() * phi2_r.cos(),
                theta_r.cos() * phi2_r.sin(),
                theta_r.sin(),
            );
            let p2 = Vec3::new(
                theta2_r.cos() * phi2_r.cos(),
                theta2_r.cos() * phi2_r.sin(),
                theta2_r.sin(),
            );
            let p3 = Vec3::new(
                theta2_r.cos() * phi_r.cos(),
                theta2_r.cos() * phi_r.sin(),
                theta2_r.sin(),
            );

            // Apply transform.
            let tp0 = (transform * p0.extend(1.0)).truncate();
            let tp1 = (transform * p1.extend(1.0)).truncate();
            let tp2 = (transform * p2.extend(1.0)).truncate();
            let tp3 = (transform * p3.extend(1.0)).truncate();

            let vi = g.vertices.len() + 1;
            let ni = g.normals.len() + 1;

            if theta > -90.0 && theta < 90.0 - dtheta + 0.001 {
                // Interior: quad face.
                g.vertices.push(tp0);
                g.vertices.push(tp1);
                g.vertices.push(tp2);
                g.vertices.push(tp3);
                // Normals equal to the raw position (unit sphere normals).
                g.normals.push(p0);
                g.normals.push(p1);
                g.normals.push(p2);
                g.normals.push(p3);
                let face: Face = (0..4).map(|k| VertexNormal { v: vi + k, n: ni + k }).collect();
                g.faces.push(face);
            } else {
                // Pole: triangle face (3 unique vertices).
                g.vertices.push(tp0);
                g.vertices.push(tp2);
                g.vertices.push(tp3);
                g.normals.push(p0);
                g.normals.push(p2);
                g.normals.push(p3);
                let face: Face = (0..3).map(|k| VertexNormal { v: vi + k, n: ni + k }).collect();
                g.faces.push(face);
            }
        }
    }

    g.deduplicate();
    g
}

// ── Cylinder tessellation ─────────────────────────────────────────────────────

/// Tessellate a cylinder as `segments` quads around its circumference plus
/// two cap polygons.
///
/// `base` is the bottom-centre, `top` is the top-centre, `radius` is the
/// radius (already in world units after being extracted from the transform).
pub fn tessellate_cylinder(base: Vec3, top: Vec3, radius: f32, segments: u32) -> ObjGroup {
    let mut g = ObjGroup::default();
    let axis = (top - base).normalize();

    // Build a local frame perpendicular to the axis.
    let up = if axis.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let right = axis.cross(up).normalize();
    let fwd = axis.cross(right).normalize();

    let mut rim_base: Vec<Vec3> = Vec::new();
    let mut rim_top: Vec<Vec3> = Vec::new();

    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let r = right * angle.cos() * radius + fwd * angle.sin() * radius;
        rim_base.push(base + r);
        rim_top.push(top + r);
    }

    let n = segments as usize;

    // Side quads.
    for i in 0..n {
        let j = (i + 1) % n;
        add_quad(&mut g, rim_base[i], rim_base[j], rim_top[j], rim_top[i]);
    }

    // Bottom cap.
    {
        let vi = g.vertices.len() + 1;
        let ni = g.normals.len() + 1;
        for &v in &rim_base {
            g.vertices.push(v);
            g.normals.push(-axis);
        }
        let face: Face = (0..n).map(|k| VertexNormal { v: vi + k, n: ni + k }).collect();
        g.faces.push(face);
    }

    // Top cap.
    {
        let vi = g.vertices.len() + 1;
        let ni = g.normals.len() + 1;
        for &v in &rim_top {
            g.vertices.push(v);
            g.normals.push(axis);
        }
        let face: Face = (0..n).map(|k| VertexNormal { v: vi + k, n: ni + k }).collect();
        g.faces.push(face);
    }

    g.deduplicate();
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustsynth_core::math::Vec3;

    #[test]
    fn box_has_six_faces() {
        let g = tessellate_box(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);
        assert_eq!(g.faces.len(), 6);
        // Each face should be a quad (4 indices).
        for f in &g.faces {
            assert_eq!(f.len(), 4, "expected quad faces");
        }
    }

    #[test]
    fn sphere_face_count() {
        let g = tessellate_sphere(8, 16, rustsynth_core::math::Mat4::IDENTITY);
        // Should have 8*16 faces, but polar rows emit triangles.
        assert!(g.faces.len() > 0);
    }

    #[test]
    fn cylinder_side_faces() {
        let g = tessellate_cylinder(Vec3::ZERO, Vec3::Z, 1.0, 8);
        // 8 side quads + 2 cap polygons = 10 faces.
        assert_eq!(g.faces.len(), 10);
    }
}

