//! OBJ exporter — writes a `Scene` to Wavefront OBJ format.
//!
//! # Grouping
//!
//! Objects are partitioned into named OBJ groups.  The grouping behaviour is
//! controlled by two flags:
//!
//! - `group_by_tag` — objects with different `tag` values go to different groups.
//! - `group_by_color` — objects with different colours go to different groups.
//!
//! If neither flag is set, all objects land in a single group named `"default"`.
//!
//! # Output
//!
//! The exporter produces two strings:
//! - OBJ text (`.obj` file contents).
//! - MTL text (`.mtl` file contents) — one material per group with `Kd` set
//!   to the group's colour.
//!
//! # Primitives
//!
//! | Kind | How exported |
//! |------|-------------|
//! | Box  | 6 quads |
//! | Sphere | UV sphere with `sphere_segments` latitude/longitude bands |
//! | Cylinder | `sphere_segments` side quads + 2 caps |
//! | Mesh | 6 quads (treated like a box with separate start/end transforms) |
//! | Grid | wireframe line-quads |
//! | Line | 2-vertex edge (`l` record) |
//! | Dot | single point (`p` record) |
//! | Triangle | single triangle face |
//! | Template | skipped (no geometry) |

use std::collections::BTreeMap;
use std::fmt::Write;

use rustsynth_core::color::Rgba;
use rustsynth_core::error::Result;
use rustsynth_core::math::Vec3;
use rustsynth_scene::primitive::PrimitiveKind;
use rustsynth_scene::Scene;
use rustsynth_scene::adapter::decompose_transform;

use crate::tessellate::{
    tessellate_box, tessellate_cylinder, tessellate_grid, tessellate_sphere, ObjGroup,
    VertexNormal,
};

// ── Public configuration ──────────────────────────────────────────────────────

/// OBJ export options.
#[derive(Debug, Clone)]
pub struct ObjExporter {
    /// Number of latitude *and* longitude bands for sphere tessellation.
    pub sphere_segments: u32,
    /// Group objects by their `tag` field.
    pub group_by_tag: bool,
    /// Group objects by their colour.
    pub group_by_color: bool,
    /// Name of the MTL file referenced in the OBJ header (without directory).
    pub mtl_file_name: String,
}

impl Default for ObjExporter {
    fn default() -> Self {
        Self {
            sphere_segments: 16,
            group_by_tag: false,
            group_by_color: false,
            mtl_file_name: "output.mtl".to_string(),
        }
    }
}

/// Result of a successful OBJ export.
pub struct ObjOutput {
    /// Contents of the `.obj` file.
    pub obj: String,
    /// Contents of the `.mtl` file.
    pub mtl: String,
}

impl ObjExporter {
    /// Export the scene, returning separate OBJ and MTL text.
    pub fn export(&self, scene: &Scene) -> Result<ObjOutput> {
        // Build per-group geometry.
        let mut groups: BTreeMap<String, GroupData> = BTreeMap::new();

        for obj in &scene.objects {
            // Determine group key.
            let group_key = self.group_key(obj);
            let entry = groups.entry(group_key).or_insert_with(|| GroupData {
                group: ObjGroup::default(),
                color: obj.color,
                alpha: obj.alpha,
            });

            // Tessellate the primitive into the group.
            let (base, dir1, dir2, dir3) = decompose_transform(obj);

            match &obj.kind {
                PrimitiveKind::Box => {
                    let sub = tessellate_box(base, dir1, dir2, dir3);
                    entry.group.merge(sub);
                }
                PrimitiveKind::Grid => {
                    let sub = tessellate_grid(base, dir1, dir2, dir3);
                    entry.group.merge(sub);
                }
                PrimitiveKind::Sphere => {
                    let sub = tessellate_sphere(
                        self.sphere_segments,
                        self.sphere_segments,
                        obj.transform,
                    );
                    entry.group.merge(sub);
                }
                PrimitiveKind::Cylinder => {
                    let top = base + dir3;
                    let radius = dir1.length();
                    let sub = tessellate_cylinder(base, top, radius, self.sphere_segments);
                    entry.group.merge(sub);
                }
                PrimitiveKind::Mesh => {
                    // Treated identically to a box (same 6-face structure).
                    let sub = tessellate_box(base, dir1, dir2, dir3);
                    entry.group.merge(sub);
                }
                PrimitiveKind::Line => {
                    add_line_to_group(&mut entry.group, base, base + dir3);
                }
                PrimitiveKind::Dot => {
                    add_dot_to_group(&mut entry.group, base);
                }
                PrimitiveKind::Triangle(payload) => {
                    add_triangle_to_group(&mut entry.group, payload);
                }
                PrimitiveKind::Template => {
                    // No geometry.
                }
            }
        }

        // Serialize groups.
        let obj_text = serialize_obj(&self.mtl_file_name, &groups);
        let mtl_text = serialize_mtl(&groups);

        Ok(ObjOutput { obj: obj_text, mtl: mtl_text })
    }

    fn group_key(&self, obj: &rustsynth_scene::object::SceneObject) -> String {
        let mut key = String::new();
        if self.group_by_tag {
            if let Some(tag) = &obj.tag {
                if !tag.is_empty() {
                    key.push_str(tag);
                }
            }
        }
        if self.group_by_color {
            key.push_str(&color_hex(obj.color));
        }
        if key.is_empty() {
            key.push_str("default");
        }
        key
    }
}

// ── Group bookkeeping ─────────────────────────────────────────────────────────

struct GroupData {
    group: ObjGroup,
    color: Rgba,
    alpha: f32,
}

// ── Primitive helpers ─────────────────────────────────────────────────────────

fn add_line_to_group(group: &mut ObjGroup, from: Vec3, to: Vec3) {
    let vi = group.vertices.len() + 1;
    group.vertices.push(from);
    group.vertices.push(to);
    group.faces.push(vec![
        VertexNormal { v: vi, n: 0 },
        VertexNormal { v: vi + 1, n: 0 },
    ]);
}

fn add_dot_to_group(group: &mut ObjGroup, pos: Vec3) {
    let vi = group.vertices.len() + 1;
    group.vertices.push(pos);
    group.faces.push(vec![VertexNormal { v: vi, n: 0 }]);
}

fn add_triangle_to_group(group: &mut ObjGroup, payload: &str) {
    // Payload: "p1x p1y p1z; p2x p2y p2z; p3x p3y p3z"
    let pts: Vec<Vec3> = payload
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(';')
        .filter_map(|part| {
            let nums: Vec<f32> = part.split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if nums.len() == 3 { Some(Vec3::new(nums[0], nums[1], nums[2])) } else { None }
        })
        .collect();

    if pts.len() == 3 {
        let vi = group.vertices.len() + 1;
        for &p in &pts {
            group.vertices.push(p);
        }
        group.faces.push((0..3).map(|i| VertexNormal { v: vi + i, n: 0 }).collect());
    }
}

// ── Serialization ─────────────────────────────────────────────────────────────

fn serialize_obj(mtl_name: &str, groups: &BTreeMap<String, GroupData>) -> String {
    let mut s = String::new();
    writeln!(s, "# RustSynth OBJ export").unwrap();
    writeln!(s, "mtllib {}", mtl_name).unwrap();
    writeln!(s).unwrap();

    let mut v_offset = 0usize;
    let mut n_offset = 0usize;

    for (group_name, data) in groups {
        let g = &data.group;

        writeln!(s, "g {}", group_name).unwrap();
        writeln!(s, "usemtl {}", group_name).unwrap();

        for v in &g.vertices {
            writeln!(s, "v {} {} {}", v.x, v.y, v.z).unwrap();
        }
        for n in &g.normals {
            writeln!(s, "vn {} {} {}", n.x, n.y, n.z).unwrap();
        }

        for face in &g.faces {
            let prefix = match face.len() {
                1 => "p",
                2 => "l",
                _ => "f",
            };
            write!(s, "{}", prefix).unwrap();
            for vn in face {
                if vn.n == 0 {
                    write!(s, " {}", vn.v + v_offset).unwrap();
                } else {
                    write!(s, " {}//{}", vn.v + v_offset, vn.n + n_offset).unwrap();
                }
            }
            writeln!(s).unwrap();
        }
        writeln!(s).unwrap();

        v_offset += g.vertices.len();
        n_offset += g.normals.len();
    }

    s
}

fn serialize_mtl(groups: &BTreeMap<String, GroupData>) -> String {
    let mut s = String::new();
    writeln!(s, "# RustSynth MTL export").unwrap();

    for (group_name, data) in groups {
        writeln!(s, "newmtl {}", group_name).unwrap();
        writeln!(s, "Kd {} {} {}", data.color.r, data.color.g, data.color.b).unwrap();
        if data.alpha < 1.0 {
            writeln!(s, "d {}", data.alpha).unwrap();
        }
        writeln!(s).unwrap();
    }

    s
}

fn color_hex(c: Rgba) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustsynth_core::{color::Rgba, math::Mat4};
    use rustsynth_scene::{object::SceneObject, primitive::PrimitiveKind, Scene};

    fn make_scene(kind: PrimitiveKind) -> Scene {
        let mut scene = Scene::default();
        scene.objects.push(SceneObject {
            kind,
            transform: Mat4::IDENTITY,
            color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            alpha: 1.0,
            tag: None,
        });
        scene
    }

    #[test]
    fn export_box_has_faces() {
        let exp = ObjExporter::default();
        let out = exp.export(&make_scene(PrimitiveKind::Box)).unwrap();
        assert!(out.obj.contains("f "), "expected face records in OBJ");
        assert!(out.mtl.contains("Kd"), "expected material in MTL");
    }

    #[test]
    fn export_sphere_has_faces() {
        let exp = ObjExporter { sphere_segments: 4, ..Default::default() };
        let out = exp.export(&make_scene(PrimitiveKind::Sphere)).unwrap();
        assert!(out.obj.contains("f ") || out.obj.contains("f\n"));
    }

    #[test]
    fn export_line_has_l_record() {
        let exp = ObjExporter::default();
        let out = exp.export(&make_scene(PrimitiveKind::Line)).unwrap();
        assert!(out.obj.contains("\nl "), "expected line record");
    }

    #[test]
    fn export_dot_has_p_record() {
        let exp = ObjExporter::default();
        let out = exp.export(&make_scene(PrimitiveKind::Dot)).unwrap();
        assert!(out.obj.contains("\np "), "expected point record");
    }

    #[test]
    fn grouping_by_color() {
        let mut scene = Scene::default();
        scene.objects.push(SceneObject {
            kind: PrimitiveKind::Box,
            transform: Mat4::IDENTITY,
            color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            alpha: 1.0,
            tag: None,
        });
        scene.objects.push(SceneObject {
            kind: PrimitiveKind::Box,
            transform: Mat4::IDENTITY,
            color: Rgba::new(0.0, 1.0, 0.0, 1.0),
            alpha: 1.0,
            tag: None,
        });
        let exp = ObjExporter { group_by_color: true, ..Default::default() };
        let out = exp.export(&scene).unwrap();
        // Two different colors → two groups.
        let group_count = out.obj.matches("\ng ").count();
        assert_eq!(group_count, 2);
    }

    #[test]
    fn mtl_has_material_for_each_group() {
        let exp = ObjExporter::default();
        let out = exp.export(&make_scene(PrimitiveKind::Box)).unwrap();
        assert!(out.mtl.contains("newmtl default"));
        assert!(out.mtl.contains("Kd 1 0 0"));
    }
}

