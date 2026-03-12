//! Template exporter — expands a `Scene` using a text template.
//!
//! Placeholder — full implementation in T11.

use rustsynth_scene::Scene;
use rustsynth_core::error::Result;
use crate::template::Template;

/// Expands a `Scene` into template output text.
pub struct TemplateExporter {
    template: Template,
}

impl TemplateExporter {
    pub fn new(template: Template) -> Self {
        Self { template }
    }

    /// Export the scene, returning the rendered output as a string.
    pub fn export(&self, _scene: &Scene) -> Result<String> {
        // Placeholder
        Ok(String::new())
    }
}
