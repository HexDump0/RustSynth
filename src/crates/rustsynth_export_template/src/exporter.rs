//! Template exporter — expands a `Scene` using a text template.
//!
//! # How it works
//!
//! 1. `begin` primitive is emitted with camera/viewport substitutions.
//! 2. Every `SceneObject` in the scene is dispatched to its named primitive
//!    (`box`, `sphere`, `cylinder`, `mesh`, `line`, `dot`, `grid`,
//!    `triangle`, `template`).  A `::tag` override is tried first.
//! 3. `end` primitive is emitted with the same camera/viewport substitutions.
//!
//! # Standard placeholders
//!
//! **For box / grid / mesh / cylinder:**
//! - `{matrix}` — row-major 4×4 (`dir1x dir1y dir1z 0 … bx by bz 1`)
//! - `{columnmatrix}` — column-major 4×4
//! - `{povmatrix}` — POV-Ray comma-separated 3×4
//! - `{r}`, `{g}`, `{b}` — colour (0.0–1.0)
//! - `{alpha}`, `{oneminusalpha}` — alpha and 1-alpha
//! - `{uid}` — unique counter
//!
//! **For sphere:**
//! - `{cx}`, `{cy}`, `{cz}`, `{rad}` — centre and radius
//! - `{r}`, `{g}`, `{b}`, `{alpha}`, `{oneminusalpha}`, `{uid}`
//!
//! **For line:**
//! - `{x1}`, `{y1}`, `{z1}`, `{x2}`, `{y2}`, `{z2}`, `{alpha}`, `{oneminusalpha}`, `{uid}`
//!
//! **For dot:**
//! - `{x}`, `{y}`, `{z}`, `{r}`, `{g}`, `{b}`, `{alpha}`, `{oneminusalpha}`, `{uid}`
//!
//! **For triangle:**
//! - `{p1x}`, `{p1y}`, `{p1z}`, `{p2x}`, …, `{p3z}`, `{alpha}`, `{oneminusalpha}`, `{uid}`
//!
//! **For begin / end:**
//! - `{CamPosX/Y/Z}`, `{CamUpX/Y/Z}`, `{CamDirX/Y/Z}`, `{CamRightX/Y/Z}`,
//!   `{CamTargetX/Y/Z}`, `{CamColumnMatrix}`, `{aspect}`, `{width}`,
//!   `{height}`, `{fov}`, `{BR}`, `{BG}`, `{BB}`, `{BR256}`, `{BG256}`,
//!   `{BB256}`


use rustsynth_core::error::Result;
use rustsynth_core::math::Vec3;
use rustsynth_scene::primitive::PrimitiveKind;
use rustsynth_scene::Scene;
use rustsynth_scene::adapter::{
    cam_column_matrix_str, column_matrix_str, decompose_transform, matrix_row_str, pov_matrix_str,
    sphere_center_radius,
};

use crate::template::{Template, TemplatePrimitive};

/// Camera parameters supplied to the template exporter by the app shell.
/// All vectors are in world space.
#[derive(Debug, Clone)]
pub struct ExportCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub width: u32,
    pub height: u32,
    pub fov: f32,
}

impl Default for ExportCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            right: Vec3::X,
            width: 800,
            height: 600,
            fov: 45.0,
        }
    }
}

/// Expands a `Scene` into template output text.
pub struct TemplateExporter {
    template: Template,
    camera: ExportCamera,
    /// Whether to log a warning (once) for each missing primitive type.
    missing_warned: std::collections::HashSet<String>,
    counter: usize,
}

impl TemplateExporter {
    pub fn new(template: Template) -> Self {
        Self {
            template,
            camera: ExportCamera::default(),
            missing_warned: std::collections::HashSet::new(),
            counter: 0,
        }
    }

    /// Override the camera used for `begin`/`end` substitutions.
    pub fn with_camera(mut self, camera: ExportCamera) -> Self {
        self.camera = camera;
        self
    }

    /// Export the scene, returning the rendered output as a string.
    pub fn export(&mut self, scene: &Scene) -> Result<String> {
        let mut output = String::new();
        self.counter = 0;

        // --- begin ---
        let bg_color = scene.background.unwrap_or(rustsynth_core::color::Rgba::BLACK);
        if let Some(prim) = self.template.get("begin").cloned() {
            let text = self.apply_begin_end_with_bg(&prim, bg_color);
            output.push_str(&text);
        }

        // --- objects ---
        let objects = scene.objects.clone();
        for obj in &objects {
            let base_name = primitive_name(&obj.kind);
            let tag_name = obj
                .tag
                .as_deref()
                .filter(|t| !t.is_empty())
                .map(|t| format!("{}::{}", base_name, t));

            // Try tagged variant first.
            let key = tag_name
                .as_deref()
                .filter(|k| self.template.has(k))
                .unwrap_or(base_name);

            if !self.template.has(key) {
                let msg = key.to_string();
                if self.missing_warned.insert(msg.clone()) {
                    log::warn!(
                        "template exporter: primitive '{}' not defined in template '{}'",
                        msg,
                        self.template.name
                    );
                }
                continue;
            }

            let prim = self.template.get(key).unwrap().clone();
            let text = match &obj.kind {
                PrimitiveKind::Sphere => self.apply_sphere(&prim, obj),
                PrimitiveKind::Line => self.apply_line(&prim, obj),
                PrimitiveKind::Dot => self.apply_dot(&prim, obj),
                PrimitiveKind::Triangle(payload) => {
                    self.apply_triangle(&prim, obj, payload.clone())
                }
                PrimitiveKind::Template => {
                    // Template primitives have no extra substitutions.
                    prim.text().to_string()
                }
                // Box, Cylinder, Grid, Mesh all use the standard matrix substitutions.
                _ => self.apply_standard(&prim, obj),
            };
            output.push_str(&text);
        }

        // --- end ---
        if let Some(prim) = self.template.get("end").cloned() {
            let text = self.apply_begin_end_with_bg(&prim, bg_color);
            output.push_str(&text);
        }

        // Normalise line endings (legacy behaviour).
        Ok(output.replace('\r', ""))
    }

    // ── Substitution helpers ──────────────────────────────────────────────────

    fn next_uid(&mut self, prefix: &str) -> String {
        let id = self.counter;
        self.counter += 1;
        format!("{}{}", prefix, id)
    }

    #[allow(dead_code)]
    fn apply_begin_end(&mut self, prim: &TemplatePrimitive) -> String {
        self.apply_begin_end_with_bg(prim, rustsynth_core::color::Rgba::BLACK)
    }

    fn apply_begin_end_with_bg(
        &mut self,
        prim: &TemplatePrimitive,
        bg: rustsynth_core::color::Rgba,
    ) -> String {
        let cam = &self.camera;
        let dir = (cam.target - cam.position).normalize();
        let aspect = cam.width as f32 / cam.height as f32;

        let mut subs: Vec<(&str, String)> = vec![
            ("{CamPosX}", fmt(cam.position.x)),
            ("{CamPosY}", fmt(cam.position.y)),
            ("{CamPosZ}", fmt(cam.position.z)),
            ("{CamUpX}", fmt(cam.up.x)),
            ("{CamUpY}", fmt(cam.up.y)),
            ("{CamUpZ}", fmt(cam.up.z)),
            ("{CamDirX}", fmt(dir.x)),
            ("{CamDirY}", fmt(dir.y)),
            ("{CamDirZ}", fmt(dir.z)),
            ("{CamRightX}", fmt(cam.right.x)),
            ("{CamRightY}", fmt(cam.right.y)),
            ("{CamRightZ}", fmt(cam.right.z)),
            ("{CamTargetX}", fmt(cam.target.x)),
            ("{CamTargetY}", fmt(cam.target.y)),
            ("{CamTargetZ}", fmt(cam.target.z)),
            ("{aspect}", fmt(aspect)),
            ("{width}", cam.width.to_string()),
            ("{height}", cam.height.to_string()),
            ("{fov}", fmt(cam.fov)),
            ("{BR}", fmt(bg.r)),
            ("{BG}", fmt(bg.g)),
            ("{BB}", fmt(bg.b)),
            ("{BR256}", fmt(bg.r * 255.0)),
            ("{BG256}", fmt(bg.g * 255.0)),
            ("{BB256}", fmt(bg.b * 255.0)),
        ];

        if prim.contains("{CamColumnMatrix}") {
            subs.push((
                "{CamColumnMatrix}",
                cam_column_matrix_str(cam.position, cam.right, cam.up, dir),
            ));
        }

        prim.apply_substitutions(subs)
    }

    fn apply_standard(&mut self, prim: &TemplatePrimitive, obj: &rustsynth_scene::object::SceneObject) -> String {
        let (base, dir1, dir2, dir3) = decompose_transform(obj);
        let uid = if prim.contains("{uid}") {
            self.next_uid("Box")
        } else {
            String::new()
        };

        let mut subs: Vec<(&str, String)> = vec![
            ("{r}", fmt(obj.color.r)),
            ("{g}", fmt(obj.color.g)),
            ("{b}", fmt(obj.color.b)),
            ("{alpha}", fmt(obj.alpha)),
            ("{oneminusalpha}", fmt(1.0 - obj.alpha)),
        ];

        if prim.contains("{matrix}") {
            subs.push(("{matrix}", matrix_row_str(base, dir1, dir2, dir3)));
        }
        if prim.contains("{columnmatrix}") {
            subs.push(("{columnmatrix}", column_matrix_str(base, dir1, dir2, dir3)));
        }
        if prim.contains("{povmatrix}") {
            subs.push(("{povmatrix}", pov_matrix_str(base, dir1, dir2, dir3)));
        }
        if !uid.is_empty() {
            subs.push(("{uid}", uid));
        }

        prim.apply_substitutions(subs)
    }

    fn apply_sphere(&mut self, prim: &TemplatePrimitive, obj: &rustsynth_scene::object::SceneObject) -> String {
        let (center, radius) = sphere_center_radius(obj);
        let uid = if prim.contains("{uid}") {
            self.next_uid("Sphere")
        } else {
            String::new()
        };

        let mut subs: Vec<(&str, String)> = vec![
            ("{cx}", fmt(center.x)),
            ("{cy}", fmt(center.y)),
            ("{cz}", fmt(center.z)),
            ("{rad}", fmt(radius)),
            ("{r}", fmt(obj.color.r)),
            ("{g}", fmt(obj.color.g)),
            ("{b}", fmt(obj.color.b)),
            ("{alpha}", fmt(obj.alpha)),
            ("{oneminusalpha}", fmt(1.0 - obj.alpha)),
        ];
        if !uid.is_empty() {
            subs.push(("{uid}", uid));
        }
        prim.apply_substitutions(subs)
    }

    fn apply_line(&mut self, prim: &TemplatePrimitive, obj: &rustsynth_scene::object::SceneObject) -> String {
        // Legacy Structure Synth convention:
        // line is along local X, centered on the YZ plane.
        let (base, dir1, _dir2, _dir3) = decompose_transform(obj);
        let from = base - dir1 * 0.5;
        let to = base + dir1 * 0.5;
        let uid = if prim.contains("{uid}") {
            self.next_uid("Line")
        } else {
            String::new()
        };

        let mut subs: Vec<(&str, String)> = vec![
            ("{x1}", fmt(from.x)),
            ("{y1}", fmt(from.y)),
            ("{z1}", fmt(from.z)),
            ("{x2}", fmt(to.x)),
            ("{y2}", fmt(to.y)),
            ("{z2}", fmt(to.z)),
            ("{alpha}", fmt(obj.alpha)),
            ("{oneminusalpha}", fmt(1.0 - obj.alpha)),
        ];
        if !uid.is_empty() {
            subs.push(("{uid}", uid));
        }
        prim.apply_substitutions(subs)
    }

    fn apply_dot(&mut self, prim: &TemplatePrimitive, obj: &rustsynth_scene::object::SceneObject) -> String {
        let (base, _, _, _) = decompose_transform(obj);
        let uid = if prim.contains("{uid}") {
            self.next_uid("Dot")
        } else {
            String::new()
        };

        let mut subs: Vec<(&str, String)> = vec![
            ("{x}", fmt(base.x)),
            ("{y}", fmt(base.y)),
            ("{z}", fmt(base.z)),
            ("{r}", fmt(obj.color.r)),
            ("{g}", fmt(obj.color.g)),
            ("{b}", fmt(obj.color.b)),
            ("{alpha}", fmt(obj.alpha)),
            ("{oneminusalpha}", fmt(1.0 - obj.alpha)),
        ];
        if !uid.is_empty() {
            subs.push(("{uid}", uid));
        }
        prim.apply_substitutions(subs)
    }

    fn apply_triangle(
        &mut self,
        prim: &TemplatePrimitive,
        obj: &rustsynth_scene::object::SceneObject,
        payload: String,
    ) -> String {
        // Triangle payload format: "p1x p1y p1z; p2x p2y p2z; p3x p3y p3z"
        let points = parse_triangle_payload(&payload);
        let (p1, p2, p3) = if points.len() == 3 {
            (points[0], points[1], points[2])
        } else {
            // Fallback if parsing fails — emit zero triangle.
            (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO)
        };

        let uid = if prim.contains("{uid}") {
            self.next_uid("Triangle")
        } else {
            String::new()
        };

        let mut subs: Vec<(&str, String)> = vec![
            ("{p1x}", fmt(p1.x)),
            ("{p1y}", fmt(p1.y)),
            ("{p1z}", fmt(p1.z)),
            ("{p2x}", fmt(p2.x)),
            ("{p2y}", fmt(p2.y)),
            ("{p2z}", fmt(p2.z)),
            ("{p3x}", fmt(p3.x)),
            ("{p3y}", fmt(p3.y)),
            ("{p3z}", fmt(p3.z)),
            ("{alpha}", fmt(obj.alpha)),
            ("{oneminusalpha}", fmt(1.0 - obj.alpha)),
        ];
        if !uid.is_empty() {
            subs.push(("{uid}", uid));
        }
        prim.apply_substitutions(subs)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format a float for template output (matches legacy `QString::number` default
/// precision — at most 6 significant digits, no trailing zeros).
fn fmt(v: f32) -> String {
    // Use shortest representation with enough precision.
    format!("{}", v)
}

/// Map a `PrimitiveKind` to its template key name.
fn primitive_name(kind: &PrimitiveKind) -> &'static str {
    match kind {
        PrimitiveKind::Box => "box",
        PrimitiveKind::Sphere => "sphere",
        PrimitiveKind::Cylinder => "cylinder",
        PrimitiveKind::Mesh => "mesh",
        PrimitiveKind::Line => "line",
        PrimitiveKind::Dot => "dot",
        PrimitiveKind::Grid => "grid",
        PrimitiveKind::Template => "template",
        PrimitiveKind::Triangle(_) => "triangle",
    }
}

/// Parse the triangle payload `"p1x p1y p1z; p2x p2y p2z; p3x p3y p3z"`.
fn parse_triangle_payload(payload: &str) -> Vec<Vec3> {
    // Strip surrounding brackets if present.
    let payload = payload.trim().trim_start_matches('[').trim_end_matches(']');
    payload
        .split(';')
        .filter_map(|part| {
            let nums: Vec<f32> = part
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if nums.len() == 3 {
                Some(Vec3::new(nums[0], nums[1], nums[2]))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::Template;
    use rustsynth_scene::{Scene, object::SceneObject, primitive::PrimitiveKind};
    use rustsynth_core::{color::Rgba, math::Mat4};

    fn box_scene() -> Scene {
        let mut scene = Scene::default();
        scene.objects.push(SceneObject {
            kind: PrimitiveKind::Box,
            transform: Mat4::IDENTITY,
            color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            alpha: 1.0,
            tag: None,
        });
        scene
    }

    const BOX_TEMPLATE_XML: &str = r#"
<template name="BoxTest" defaultExtension="*.txt">
  <primitive name="begin">BEGIN</primitive>
  <primitive name="box">BOX {r} {g} {b}</primitive>
  <primitive name="end">END</primitive>
</template>
"#;

    #[test]
    fn export_box() {
        let t = Template::from_xml(BOX_TEMPLATE_XML).unwrap();
        let mut exporter = TemplateExporter::new(t);
        let scene = box_scene();
        let out = exporter.export(&scene).unwrap();
        assert!(out.contains("BEGIN"));
        assert!(out.contains("BOX 1 0 0"));
        assert!(out.contains("END"));
    }

    #[test]
    fn missing_primitive_skipped() {
        // Template has no sphere entry.
        const XML: &str = r#"<template name="T"><primitive name="begin">B</primitive><primitive name="end">E</primitive></template>"#;
        let t = Template::from_xml(XML).unwrap();
        let mut exporter = TemplateExporter::new(t);
        let mut scene = Scene::default();
        scene.objects.push(SceneObject {
            kind: PrimitiveKind::Sphere,
            transform: Mat4::IDENTITY,
            color: Rgba::WHITE,
            alpha: 1.0,
            tag: None,
        });
        let out = exporter.export(&scene).unwrap();
        // Should still emit begin/end, just no sphere content.
        assert_eq!(out, "BE");
    }

    #[test]
    fn uid_increments() {
        const XML: &str = r#"<template name="T"><primitive name="begin"></primitive><primitive name="box">B{uid}</primitive><primitive name="end"></primitive></template>"#;
        let t = Template::from_xml(XML).unwrap();
        let mut exporter = TemplateExporter::new(t);
        let mut scene = Scene::default();
        for _ in 0..3 {
            scene.objects.push(SceneObject {
                kind: PrimitiveKind::Box,
                transform: Mat4::IDENTITY,
                color: Rgba::WHITE,
                alpha: 1.0,
                tag: None,
            });
        }
        let out = exporter.export(&scene).unwrap();
        assert!(out.contains("BBox0"));
        assert!(out.contains("BBox1"));
        assert!(out.contains("BBox2"));
    }

    #[test]
    fn export_line_uses_local_x_centered_endpoints() {
        const XML: &str = r#"<template name="T"><primitive name="line">{x1},{y1},{z1}|{x2},{y2},{z2}</primitive></template>"#;
        let t = Template::from_xml(XML).unwrap();
        let mut exporter = TemplateExporter::new(t);
        let mut scene = Scene::default();
        scene.objects.push(SceneObject {
            kind: PrimitiveKind::Line,
            transform: Mat4::IDENTITY,
            color: Rgba::WHITE,
            alpha: 1.0,
            tag: None,
        });

        let out = exporter.export(&scene).unwrap();
        assert_eq!(out, "-0.5,0,0|0.5,0,0");
    }
}

