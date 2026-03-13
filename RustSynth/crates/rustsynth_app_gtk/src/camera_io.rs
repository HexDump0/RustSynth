//! Camera state I/O — serialize / deserialize [`ArcballCamera`] to/from JSON files.
//!
//! Also handles the `// @rs-camera: {...}` annotation that can be embedded as the
//! first line of an EisenScript file so the camera is restored on open.

use std::path::Path;

use anyhow::Result;
use rustsynth_render_api::camera::ArcballCamera;

// ─────────────────────────────────────────────────────────────────────────────
// File I/O
// ─────────────────────────────────────────────────────────────────────────────

/// Serialize the camera to a JSON file at `path`.
pub fn save_camera(camera: &ArcballCamera, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(camera)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Deserialize a camera from a JSON file at `path`.
pub fn load_camera(path: &Path) -> Result<ArcballCamera> {
    let content = std::fs::read_to_string(path)?;
    let camera: ArcballCamera = serde_json::from_str(&content)?;
    Ok(camera)
}

// ─────────────────────────────────────────────────────────────────────────────
// Script annotation
// ─────────────────────────────────────────────────────────────────────────────

const ANNOTATION_PREFIX: &str = "// @rs-camera: ";

/// Produce a `// @rs-camera: {...}` annotation string for the given camera.
pub fn camera_to_annotation(camera: &ArcballCamera) -> Result<String> {
    let json = serde_json::to_string(camera)?;
    Ok(format!("{}{}", ANNOTATION_PREFIX, json))
}

/// Insert or replace a `// @rs-camera:` annotation at the top of `source`.
///
/// If the first line already contains a camera annotation it is replaced;
/// otherwise a new line is prepended.
pub fn insert_camera_annotation(source: &str, camera: &ArcballCamera) -> Result<String> {
    let annotation = camera_to_annotation(camera)?;
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.starts_with(ANNOTATION_PREFIX)).unwrap_or(false) {
        let mut new_lines = vec![annotation.as_str()];
        new_lines.extend_from_slice(&lines[1..]);
        Ok(new_lines.join("\n"))
    } else if source.is_empty() {
        Ok(annotation)
    } else {
        Ok(format!("{}\n{}", annotation, source))
    }
}

/// Try to parse a camera annotation from the first line of `source`.
///
/// Returns `None` if the first line is not a camera annotation or fails to
/// deserialize.
pub fn extract_camera_annotation(source: &str) -> Option<ArcballCamera> {
    let first = source.lines().next()?;
    let json = first.strip_prefix(ANNOTATION_PREFIX)?;
    serde_json::from_str(json).ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_camera() -> ArcballCamera {
        ArcballCamera::default()
    }

    #[test]
    fn annotation_round_trips() {
        let cam = default_camera();
        let annotation = camera_to_annotation(&cam).unwrap();
        assert!(annotation.starts_with(ANNOTATION_PREFIX));
        let restored: ArcballCamera = serde_json::from_str(
            annotation.strip_prefix(ANNOTATION_PREFIX).unwrap()
        ).unwrap();
        assert!((restored.yaw - cam.yaw).abs() < 1e-4);
        assert!((restored.pitch - cam.pitch).abs() < 1e-4);
        assert!((restored.distance - cam.distance).abs() < 1e-4);
    }

    #[test]
    fn extract_from_script_first_line() {
        let cam = default_camera();
        let script = "set maxdepth 200\nrule r0 { }";
        let with_annotation = insert_camera_annotation(script, &cam).unwrap();
        assert!(with_annotation.starts_with(ANNOTATION_PREFIX));

        let extracted = extract_camera_annotation(&with_annotation).unwrap();
        assert!((extracted.yaw - cam.yaw).abs() < 1e-4);
    }

    #[test]
    fn replace_existing_annotation() {
        let cam1 = default_camera();
        let mut cam2 = default_camera();
        cam2.yaw = 90.0;

        let script = "// @rs-camera: {\"yaw\":30.0,\"pitch\":20.0,\"distance\":5.0,\"pivot\":[0.0,0.0,0.0],\"fov_y\":45.0,\"aspect\":1.0,\"near\":0.01,\"far\":1000.0}\nset maxdepth 200";
        let updated = insert_camera_annotation(script, &cam2).unwrap();
        // The first line should now reflect cam2's yaw of 90.0
        let first_line = updated.lines().next().unwrap();
        assert!(first_line.contains("90.0") || first_line.contains("\"yaw\":90"));
        // Should still have the script body on the second line
        assert!(updated.contains("set maxdepth 200"));
        let _ = cam1; // suppress unused warning
    }

    #[test]
    fn no_annotation_returns_none() {
        let script = "set maxdepth 200\nrule r0 { }";
        assert!(extract_camera_annotation(script).is_none());
    }
}
