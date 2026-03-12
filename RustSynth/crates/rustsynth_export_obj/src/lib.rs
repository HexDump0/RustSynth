//! `rustsynth_export_obj` — OBJ exporter.
//!
//! Writes a canonical `Scene` to Wavefront OBJ format.
//! Supports grouping options and sphere tessellation settings.

pub mod exporter;
pub mod tessellate;

pub use exporter::ObjExporter;
