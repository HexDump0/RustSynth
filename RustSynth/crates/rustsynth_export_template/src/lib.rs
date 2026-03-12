//! `rustsynth_export_template` — template-based exporter.
//!
//! Expands a canonical `Scene` into text output using renderer templates
//! (Sunflow, POV-Ray, RenderMan, etc.). Mirrors legacy `TemplateRenderer`.

pub mod exporter;
pub mod template;

pub use exporter::{ExportCamera, TemplateExporter};
pub use template::Template;
