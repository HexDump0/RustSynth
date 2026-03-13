//! Headless EisenScript compilation pipeline.
//!
//! Converts source text into a [`rustsynth_scene::Scene`] via the standard
//! preprocessing → lexing → parsing → resolution → validation → evaluation
//! chain, collecting all diagnostics into a string list.

use anyhow::Result;
use rustsynth_eisenscript::parser;
use rustsynth_eisenscript::preprocessor;
use rustsynth_eval::{build, BuildConfig};
use rustsynth_scene::Scene;
use rustsynth_semantics::{resolve, validate};

pub use rustsynth_eisenscript::preprocessor::GuiParam;

/// Run the full pipeline on `source` with the given `config`.
///
/// Returns the produced `Scene`, a list of warning/info strings, and any GUI
/// parameters extracted from `#define` directives with type annotations.
/// Hard parse errors are folded into the warning list; the evaluator still
/// runs in best-effort mode on a partial graph.
pub fn run_pipeline(source: &str, config: &BuildConfig) -> Result<(Scene, Vec<String>, Vec<GuiParam>)> {
    let mut warnings: Vec<String> = Vec::new();

    // ── 1. Preprocess ─────────────────────────────────────────────────────────
    let pre = preprocessor::preprocess(source, config.seed);
    let gui_params = pre.gui_params.clone();
    for d in &pre.diagnostics {
        warnings.push(format!(
            "[{}] line {}: {}",
            if matches!(d.severity, rustsynth_eisenscript::diagnostics::Severity::Error) {
                "ERROR"
            } else {
                "WARN"
            },
            d.line,
            d.message
        ));
    }

    // ── 2. Parse ──────────────────────────────────────────────────────────────
    let parse = parser::parse(&pre.output);
    for d in &parse.diagnostics {
        warnings.push(format!(
            "[{}] line {}: {}",
            if matches!(d.severity, rustsynth_eisenscript::diagnostics::Severity::Error) {
                "ERROR"
            } else {
                "WARN"
            },
            d.line,
            d.message
        ));
    }

    // ── 3. Resolve ────────────────────────────────────────────────────────────
    let (graph, resolve_diags) = resolve(&parse.script);
    for d in &resolve_diags {
        warnings.push(format!(
            "[{}] line {}: {}",
            if matches!(d.severity, rustsynth_eisenscript::diagnostics::Severity::Error) {
                "ERROR"
            } else {
                "WARN"
            },
            d.line,
            d.message
        ));
    }

    // ── 4. Validate ───────────────────────────────────────────────────────────
    let val_diags = validate(&graph);
    for d in &val_diags {
        warnings.push(format!(
            "[{}] line {}: {}",
            if matches!(d.severity, rustsynth_eisenscript::diagnostics::Severity::Error) {
                "ERROR"
            } else {
                "WARN"
            },
            d.line,
            d.message
        ));
    }

    // ── 5. Evaluate ───────────────────────────────────────────────────────────
    let scene = build(&graph, config);

    Ok((scene, warnings, gui_params))
}
