//! WebAssembly bindings for the RustSynth EisenScript pipeline.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use rustsynth_eisenscript::parser;
use rustsynth_eisenscript::preprocessor::{self, GuiParam};
use rustsynth_eval::{build, BuildConfig};
use rustsynth_export_obj::ObjExporter;
use rustsynth_export_template::{Template, TemplateExporter};
use rustsynth_scene::Scene;
use rustsynth_semantics::{resolve, validate};

// ── Builtin template (same as CLI / Tauri) ────────────────────────────────────

const BUILTIN_TEMPLATE_XML: &str = r#"<template name="TextDump"
    defaultExtension="Text file (*.txt)">
    <primitive name="begin">// RustSynth scene export\n// Objects:\n</primitive>
    <primitive name="box">box {matrix}\n</primitive>
    <primitive name="sphere">sphere cx={cx} cy={cy} cz={cz} r={rad} rgb={r},{g},{b}\n</primitive>
    <primitive name="cylinder">cylinder {matrix}\n</primitive>
    <primitive name="line">line {x1},{y1},{z1} {x2},{y2},{z2}\n</primitive>
    <primitive name="dot">dot {x},{y},{z}\n</primitive>
    <primitive name="grid">grid {matrix}\n</primitive>
    <primitive name="end">// end\n</primitive>
</template>"#;

// ── Result structs ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PipelineResult {
    scene: Scene,
    warnings: Vec<String>,
    gui_params: Vec<GuiParam>,
}

#[derive(Serialize)]
struct ObjResult {
    obj: String,
    mtl: String,
}

// ── Pipeline helper ───────────────────────────────────────────────────────────

fn run_pipeline(
    source: &str,
    config: &BuildConfig,
) -> Result<(Scene, Vec<String>, Vec<GuiParam>), JsError> {
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

// ── Exported WASM functions ───────────────────────────────────────────────────

/// Run the full EisenScript pipeline and return the scene, warnings, and GUI
/// parameters as a JS object.
#[wasm_bindgen]
pub fn run_script(source: &str, config_json: &str) -> Result<JsValue, JsError> {
    let config: BuildConfig =
        serde_json::from_str(config_json).map_err(|e| JsError::new(&e.to_string()))?;

    let (scene, warnings, gui_params) = run_pipeline(source, &config)?;

    let result = PipelineResult {
        scene,
        warnings,
        gui_params,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Run the pipeline and export the scene to OBJ format.
///
/// Returns a JS object with `obj` and `mtl` string fields.
#[wasm_bindgen]
pub fn export_obj(source: &str, config_json: &str) -> Result<JsValue, JsError> {
    let config: BuildConfig =
        serde_json::from_str(config_json).map_err(|e| JsError::new(&e.to_string()))?;

    let (scene, _warnings, _gui_params) = run_pipeline(source, &config)?;

    let exporter = ObjExporter::default();
    let output = exporter
        .export(&scene)
        .map_err(|e| JsError::new(&e.to_string()))?;

    let result = ObjResult {
        obj: output.obj,
        mtl: output.mtl,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Run the pipeline and export using a template.
///
/// If `template_xml` is empty, the built-in TextDump template is used.
#[wasm_bindgen]
pub fn export_template(
    source: &str,
    config_json: &str,
    template_xml: &str,
) -> Result<String, JsError> {
    let config: BuildConfig =
        serde_json::from_str(config_json).map_err(|e| JsError::new(&e.to_string()))?;

    let (scene, _warnings, _gui_params) = run_pipeline(source, &config)?;

    let xml = if template_xml.is_empty() {
        BUILTIN_TEMPLATE_XML
    } else {
        template_xml
    };

    let tmpl = Template::from_xml(xml).map_err(|e| JsError::new(&e.to_string()))?;
    let mut exporter = TemplateExporter::new(tmpl);
    let text = exporter
        .export(&scene)
        .map_err(|e| JsError::new(&e.to_string()))?;

    Ok(text)
}
