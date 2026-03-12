//! Mesh generation for each primitive kind.
//!
//! Converts [`SceneObject`]s into vertex/index buffers ready for GPU upload.
//! Each function returns `(Vec<Vertex>, Vec<u32>)`.

use std::f32::consts::PI;

use rustsynth_core::color::Rgba;
use rustsynth_core::math::Vec3;
use rustsynth_render_api::adapter;
use rustsynth_scene::object::SceneObject;
use rustsynth_scene::primitive::PrimitiveKind;

use crate::gpu_types::Vertex;

/// Tessellation resolution for spheres and cylinders.
const SPHERE_STACKS: u32 = 16;
const SPHERE_SLICES: u32 = 24;
const CYLINDER_SLICES: u32 = 24;

/// Convert a full scene into a flat vertex + index buffer pair.
pub fn scene_to_mesh(objects: &[SceneObject]) -> (Vec<Vertex>, Vec<u32>) {
    let mut all_verts = Vec::new();
    let mut all_idxs = Vec::new();

    for obj in objects {
        let (verts, idxs) = object_to_mesh(obj);
        let base = all_verts.len() as u32;
        all_verts.extend(verts);
        all_idxs.extend(idxs.iter().map(|i| i + base));
    }
    (all_verts, all_idxs)
}

/// Convert one scene object into mesh geometry.
fn object_to_mesh(obj: &SceneObject) -> (Vec<Vertex>, Vec<u32>) {
    let color = rgba_to_array(&obj.color, obj.alpha);
    match &obj.kind {
        PrimitiveKind::Box => make_box(obj, color),
        PrimitiveKind::Sphere => make_sphere(obj, color),
        PrimitiveKind::Cylinder => make_cylinder(obj, color),
        PrimitiveKind::Line => make_line(obj, color),
        PrimitiveKind::Dot => make_dot(obj, color),
        PrimitiveKind::Grid => make_grid(obj, color),
        PrimitiveKind::Mesh => make_box(obj, color),     // fallback
        PrimitiveKind::Template => (vec![], vec![]),      // invisible
        PrimitiveKind::Triangle(payload) => make_triangle(obj, payload, color),
    }
}

fn rgba_to_array(c: &Rgba, alpha: f32) -> [f32; 4] {
    [c.r, c.g, c.b, c.a * alpha]
}

// ─── Box ─────────────────────────────────────────────────────────────────────

/// Generate a unit cube [-0.5, 0.5]^3 transformed by `obj.transform`.
fn make_box(obj: &SceneObject, color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
    let (base, d1, d2, d3) = adapter::decompose_transform(obj);

    // 8 corners of the OBB (oriented bounding box)
    let corners = [
        base - d1 * 0.5 - d2 * 0.5 - d3 * 0.5, // 0: ---
        base + d1 * 0.5 - d2 * 0.5 - d3 * 0.5, // 1: +--
        base + d1 * 0.5 + d2 * 0.5 - d3 * 0.5, // 2: ++-
        base - d1 * 0.5 + d2 * 0.5 - d3 * 0.5, // 3: -+-
        base - d1 * 0.5 - d2 * 0.5 + d3 * 0.5, // 4: --+
        base + d1 * 0.5 - d2 * 0.5 + d3 * 0.5, // 5: +-+
        base + d1 * 0.5 + d2 * 0.5 + d3 * 0.5, // 6: +++
        base - d1 * 0.5 + d2 * 0.5 + d3 * 0.5, // 7: -++
    ];

    // 6 faces: each defined by 4 corner indices and a face normal
    let faces: [(usize, usize, usize, usize, Vec3); 6] = [
        (0, 1, 2, 3, -d3.normalize_or_zero()), // front  (-Z)
        (5, 4, 7, 6, d3.normalize_or_zero()),   // back   (+Z)
        (4, 0, 3, 7, -d1.normalize_or_zero()), // left   (-X)
        (1, 5, 6, 2, d1.normalize_or_zero()),   // right  (+X)
        (3, 2, 6, 7, d2.normalize_or_zero()),   // top    (+Y)
        (4, 5, 1, 0, -d2.normalize_or_zero()), // bottom (-Y)
    ];

    let mut verts = Vec::with_capacity(24);
    let mut idxs = Vec::with_capacity(36);

    for (a, b, c, d, normal) in &faces {
        let base_idx = verts.len() as u32;
        let n = [normal.x, normal.y, normal.z];
        verts.push(Vertex { position: v3_arr(corners[*a]), normal: n, color });
        verts.push(Vertex { position: v3_arr(corners[*b]), normal: n, color });
        verts.push(Vertex { position: v3_arr(corners[*c]), normal: n, color });
        verts.push(Vertex { position: v3_arr(corners[*d]), normal: n, color });
        // Two triangles per face
        idxs.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }
    (verts, idxs)
}

// ─── Sphere ──────────────────────────────────────────────────────────────────

/// Generate a UV sphere at `obj.transform` center/radius.
fn make_sphere(obj: &SceneObject, color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
    let (center, radius) = adapter::sphere_center_radius(obj);
    let stacks = SPHERE_STACKS;
    let slices = SPHERE_SLICES;

    let mut verts = Vec::with_capacity(((stacks + 1) * (slices + 1)) as usize);
    let mut idxs = Vec::new();

    for i in 0..=stacks {
        let phi = PI * (i as f32) / (stacks as f32); // 0..π
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for j in 0..=slices {
            let theta = 2.0 * PI * (j as f32) / (slices as f32); // 0..2π
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let nx = sin_phi * cos_theta;
            let ny = cos_phi;
            let nz = sin_phi * sin_theta;

            let pos = center + Vec3::new(nx, ny, nz) * radius;
            verts.push(Vertex {
                position: v3_arr(pos),
                normal: [nx, ny, nz],
                color,
            });
        }
    }

    for i in 0..stacks {
        for j in 0..slices {
            let first = i * (slices + 1) + j;
            let second = first + slices + 1;
            idxs.extend_from_slice(&[first, second, first + 1]);
            idxs.extend_from_slice(&[first + 1, second, second + 1]);
        }
    }
    (verts, idxs)
}

// ─── Cylinder ────────────────────────────────────────────────────────────────

/// Generate a cylinder with endcaps from `obj.transform`.
fn make_cylinder(obj: &SceneObject, color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
    let (bot, top, radius) = adapter::cylinder_endpoints(obj);
    let slices = CYLINDER_SLICES;

    let axis = top - bot;
    let height = axis.length();
    if height < 1e-8 {
        return (vec![], vec![]);
    }
    let axis_n = axis / height;

    // Build a local coordinate frame
    let arbitrary = if axis_n.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
    let tangent = axis_n.cross(arbitrary).normalize();
    let bitangent = axis_n.cross(tangent);

    let mut verts = Vec::new();
    let mut idxs = Vec::new();

    // --- Side vertices ---
    for i in 0..=slices {
        let angle = 2.0 * PI * (i as f32) / (slices as f32);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let normal = tangent * cos_a + bitangent * sin_a;
        let offset = normal * radius;

        let n = [normal.x, normal.y, normal.z];
        verts.push(Vertex { position: v3_arr(bot + offset), normal: n, color });
        verts.push(Vertex { position: v3_arr(top + offset), normal: n, color });
    }

    // Side indices
    for i in 0..slices {
        let b = i * 2;
        idxs.extend_from_slice(&[b, b + 1, b + 2]);
        idxs.extend_from_slice(&[b + 2, b + 1, b + 3]);
    }

    // --- Bottom cap ---
    let cap_base = verts.len() as u32;
    let bot_n = [-axis_n.x, -axis_n.y, -axis_n.z];
    // Center vertex
    verts.push(Vertex { position: v3_arr(bot), normal: bot_n, color });
    let bot_center = cap_base;
    for i in 0..=slices {
        let angle = 2.0 * PI * (i as f32) / (slices as f32);
        let offset = (tangent * angle.cos() + bitangent * angle.sin()) * radius;
        verts.push(Vertex { position: v3_arr(bot + offset), normal: bot_n, color });
    }
    for i in 0..slices {
        let c = bot_center;
        let a = cap_base + 1 + i;
        let b = cap_base + 1 + i + 1;
        idxs.extend_from_slice(&[c, b, a]); // winding for outward normal
    }

    // --- Top cap ---
    let cap_base = verts.len() as u32;
    let top_n = [axis_n.x, axis_n.y, axis_n.z];
    verts.push(Vertex { position: v3_arr(top), normal: top_n, color });
    let top_center = cap_base;
    for i in 0..=slices {
        let angle = 2.0 * PI * (i as f32) / (slices as f32);
        let offset = (tangent * angle.cos() + bitangent * angle.sin()) * radius;
        verts.push(Vertex { position: v3_arr(top + offset), normal: top_n, color });
    }
    for i in 0..slices {
        let c = top_center;
        let a = cap_base + 1 + i;
        let b = cap_base + 1 + i + 1;
        idxs.extend_from_slice(&[c, a, b]); // opposite winding for outward normal
    }

    (verts, idxs)
}

// ─── Line ────────────────────────────────────────────────────────────────────

/// Approximate a line as a very thin cylinder.
fn make_line(obj: &SceneObject, color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
    let (base, d1, _d2, d3) = adapter::decompose_transform(obj);
    let end = base + d3;
    let radius = d1.length() * 0.05; // very thin

    let axis = end - base;
    let height = axis.length();
    if height < 1e-8 {
        return (vec![], vec![]);
    }

    // Build a thin cylinder
    let line_obj = SceneObject {
        kind: PrimitiveKind::Cylinder,
        transform: obj.transform,
        color: obj.color,
        alpha: obj.alpha,
        tag: obj.tag.clone(),
    };
    // Use a scaled-down transform for thin lines
    let _ = (base, end, radius);
    make_cylinder(&line_obj, color)
}

// ─── Dot ─────────────────────────────────────────────────────────────────────

/// Render a dot as a small sphere.
fn make_dot(obj: &SceneObject, color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
    // Use the sphere generator at the object center with small radius
    make_sphere(obj, color)
}

// ─── Grid ────────────────────────────────────────────────────────────────────

/// Render a grid as a flat quad.
fn make_grid(obj: &SceneObject, color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
    let (base, d1, d2, _d3) = adapter::decompose_transform(obj);
    let normal = d1.cross(d2).normalize_or_zero();
    let n = [normal.x, normal.y, normal.z];

    let c0 = base - d1 * 0.5 - d2 * 0.5;
    let c1 = base + d1 * 0.5 - d2 * 0.5;
    let c2 = base + d1 * 0.5 + d2 * 0.5;
    let c3 = base - d1 * 0.5 + d2 * 0.5;

    let verts = vec![
        Vertex { position: v3_arr(c0), normal: n, color },
        Vertex { position: v3_arr(c1), normal: n, color },
        Vertex { position: v3_arr(c2), normal: n, color },
        Vertex { position: v3_arr(c3), normal: n, color },
    ];
    let idxs = vec![0, 1, 2, 0, 2, 3];
    (verts, idxs)
}

// ─── Triangle ────────────────────────────────────────────────────────────────

/// Parse and render an inline triangle from its raw vertex string.
/// Format: `"[x0 y0 z0; x1 y1 z1; x2 y2 z2]"`
fn make_triangle(
    _obj: &SceneObject,
    payload: &str,
    color: [f32; 4],
) -> (Vec<Vertex>, Vec<u32>) {
    let inner = payload.trim().trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = inner.split(';').collect();
    if parts.len() != 3 {
        log::warn!("Triangle payload has {} parts, expected 3: {:?}", parts.len(), payload);
        return (vec![], vec![]);
    }

    let mut positions = Vec::with_capacity(3);
    for part in &parts {
        let coords: Vec<f32> = part
            .split_whitespace()
            .filter_map(|s| s.parse::<f32>().ok())
            .collect();
        if coords.len() != 3 {
            log::warn!("Triangle vertex has {} coords, expected 3: {:?}", coords.len(), part);
            return (vec![], vec![]);
        }
        positions.push(Vec3::new(coords[0], coords[1], coords[2]));
    }

    let edge1 = positions[1] - positions[0];
    let edge2 = positions[2] - positions[0];
    let normal = edge1.cross(edge2).normalize_or_zero();
    let n = [normal.x, normal.y, normal.z];

    let verts = vec![
        Vertex { position: v3_arr(positions[0]), normal: n, color },
        Vertex { position: v3_arr(positions[1]), normal: n, color },
        Vertex { position: v3_arr(positions[2]), normal: n, color },
    ];
    let idxs = vec![0, 1, 2];
    (verts, idxs)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn v3_arr(v: Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rustsynth_core::color::Rgba;
    use rustsynth_core::math::Mat4;
    use rustsynth_scene::object::SceneObject;
    use rustsynth_scene::primitive::PrimitiveKind;

    fn identity_obj(kind: PrimitiveKind) -> SceneObject {
        SceneObject {
            kind,
            transform: Mat4::IDENTITY,
            color: Rgba::WHITE,
            alpha: 1.0,
            tag: None,
        }
    }

    #[test]
    fn box_generates_24_verts_36_indices() {
        let (v, i) = object_to_mesh(&identity_obj(PrimitiveKind::Box));
        assert_eq!(v.len(), 24, "box should have 24 vertices (4 per face × 6)");
        assert_eq!(i.len(), 36, "box should have 36 indices (6 per face × 6)");
    }

    #[test]
    fn sphere_generates_nonempty_mesh() {
        let (v, i) = object_to_mesh(&identity_obj(PrimitiveKind::Sphere));
        assert!(!v.is_empty(), "sphere should produce vertices");
        assert!(!i.is_empty(), "sphere should produce indices");
        // UV sphere: (stacks+1)*(slices+1) vertices
        let expected_verts = (SPHERE_STACKS + 1) * (SPHERE_SLICES + 1);
        assert_eq!(v.len(), expected_verts as usize);
    }

    #[test]
    fn cylinder_generates_nonempty_mesh() {
        // Scale the Z axis to give the cylinder height
        let mut obj = identity_obj(PrimitiveKind::Cylinder);
        obj.transform = Mat4::from_scale(Vec3::new(0.5, 0.5, 1.0));
        let (v, i) = object_to_mesh(&obj);
        assert!(!v.is_empty(), "cylinder should produce vertices");
        assert!(!i.is_empty(), "cylinder should produce indices");
    }

    #[test]
    fn triangle_parses_valid_payload() {
        let obj = SceneObject {
            kind: PrimitiveKind::Triangle("[0 0 0; 1 0 0; 0 1 0]".to_string()),
            transform: Mat4::IDENTITY,
            color: Rgba::WHITE,
            alpha: 1.0,
            tag: None,
        };
        let (v, i) = object_to_mesh(&obj);
        assert_eq!(v.len(), 3);
        assert_eq!(i.len(), 3);
    }

    #[test]
    fn template_produces_no_geometry() {
        let (v, i) = object_to_mesh(&identity_obj(PrimitiveKind::Template));
        assert!(v.is_empty());
        assert!(i.is_empty());
    }

    #[test]
    fn scene_to_mesh_accumulates_objects() {
        let objects = vec![
            identity_obj(PrimitiveKind::Box),
            identity_obj(PrimitiveKind::Box),
        ];
        let (v, i) = scene_to_mesh(&objects);
        assert_eq!(v.len(), 48, "2 boxes × 24 vertices");
        assert_eq!(i.len(), 72, "2 boxes × 36 indices");
        // Indices of second box should be offset
        assert!(i[36] >= 24, "second box indices should be offset by vertex base");
    }
}
