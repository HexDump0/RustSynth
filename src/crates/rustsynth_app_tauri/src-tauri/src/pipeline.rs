//! Headless EisenScript compilation pipeline.
//!
//! Identical to the logic in `rustsynth_app_gtk::pipeline` but without any
//! GTK dependencies. Converts source text into a serializable `Scene`.

use anyhow::Result;
use rustsynth_eisenscript::parser;
use rustsynth_eisenscript::preprocessor::{self, GuiParam};
use rustsynth_eval::{build, BuildConfig};
use rustsynth_scene::Scene;
use rustsynth_semantics::{resolve, validate};

/// Run the full pipeline on `source` with the given `config`.
///
/// Returns the produced `Scene`, a list of warning/info strings, and any GUI
/// parameters extracted from `#define` directives with type annotations.
pub fn run_pipeline(
    source: &str,
    config: &BuildConfig,
) -> Result<(Scene, Vec<String>, Vec<GuiParam>)> {
    let mut warnings: Vec<String> = Vec::new();

    // 1. Preprocess
    let pre = preprocessor::preprocess(source, config.seed);
    let gui_params = pre.gui_params.clone();
    for d in &pre.diagnostics {
        warnings.push(format!(
            "[{}] line {}: {}",
            severity_label(&d.severity),
            d.line,
            d.message
        ));
    }

    // 2. Parse
    let parse = parser::parse(&pre.output);
    for d in &parse.diagnostics {
        warnings.push(format!(
            "[{}] line {}: {}",
            severity_label(&d.severity),
            d.line,
            d.message
        ));
    }

    // 3. Resolve
    let (graph, resolve_diags) = resolve(&parse.script);
    for d in &resolve_diags {
        warnings.push(format!(
            "[{}] line {}: {}",
            severity_label(&d.severity),
            d.line,
            d.message
        ));
    }

    // 4. Validate
    let val_diags = validate(&graph);
    for d in &val_diags {
        warnings.push(format!(
            "[{}] line {}: {}",
            severity_label(&d.severity),
            d.line,
            d.message
        ));
    }

    // 5. Evaluate
    let scene = build(&graph, config);

    Ok((scene, warnings, gui_params))
}

fn severity_label(sev: &rustsynth_eisenscript::diagnostics::Severity) -> &'static str {
    match sev {
        rustsynth_eisenscript::diagnostics::Severity::Error => "ERROR",
        _ => "WARN",
    }
}
