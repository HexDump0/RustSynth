//! EisenScript preprocessor.
//!
//! Placeholder skeleton — full implementation in T04.
//!
//! The real preprocessor handles:
//! - `#define name value` substitutions
//! - `#define name value (float:lo-hi)` GUI float parameters
//! - `#define name value (int:lo-hi)` GUI int parameters
//! - `random[lo,hi]` replacement using seeded RNG

use crate::diagnostics::Diagnostic;

/// A GUI-controllable parameter extracted from preprocessor directives.
#[derive(Debug, Clone)]
pub enum GuiParam {
    Float { name: String, default: f64, min: f64, max: f64 },
    Int   { name: String, default: i64, min: i64, max: i64 },
}

/// Result of preprocessing.
pub struct PreprocessResult {
    pub output: String,
    pub gui_params: Vec<GuiParam>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Preprocess EisenScript source text.
///
/// `seed` is used for `random[lo,hi]` substitutions.
///
/// This is a stub — the real implementation will be built in T04.
pub fn preprocess(_source: &str, _seed: u64) -> PreprocessResult {
    PreprocessResult {
        output: _source.to_owned(),
        gui_params: vec![],
        diagnostics: vec![],
    }
}
