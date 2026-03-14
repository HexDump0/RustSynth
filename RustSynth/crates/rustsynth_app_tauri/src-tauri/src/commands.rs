//! Tauri commands — the IPC bridge between the React frontend and the Rust core.

use serde::Serialize;
use tauri_plugin_dialog::DialogExt;

use rustsynth_eisenscript::preprocessor::GuiParam;
use rustsynth_eval::BuildConfig;
use rustsynth_scene::Scene;

use crate::pipeline;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PipelineResult {
    pub scene: Scene,
    pub warnings: Vec<String>,
    pub gui_params: Vec<GuiParam>,
}

#[derive(Serialize)]
pub struct FileOpenResult {
    pub path: String,
    pub content: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Commands
// ─────────────────────────────────────────────────────────────────────────────

/// Run the EisenScript pipeline and return the scene as JSON.
#[tauri::command]
pub fn run_script(source: String, config: BuildConfig) -> Result<PipelineResult, String> {
    pipeline::run_pipeline(&source, &config)
        .map(|(scene, warnings, gui_params)| PipelineResult {
            scene,
            warnings,
            gui_params,
        })
        .map_err(|e| e.to_string())
}

/// Show an open-file dialog and return the file contents.
#[tauri::command]
pub async fn open_file_dialog(app: tauri::AppHandle) -> Result<FileOpenResult, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("EisenScript", &["es", "eisenscript"])
        .add_filter("All files", &["*"])
        .blocking_pick_file();

    match file {
        Some(path) => {
            let path_str = path.to_string();
            let content =
                std::fs::read_to_string(path.as_path().ok_or("Invalid path")?)
                    .map_err(|e| e.to_string())?;
            Ok(FileOpenResult {
                path: path_str,
                content,
            })
        }
        None => Err("cancelled".to_string()),
    }
}

/// Show a save-file dialog and write content to the file.
#[tauri::command]
pub async fn save_file_dialog(
    app: tauri::AppHandle,
    content: String,
    current_path: Option<String>,
) -> Result<String, String> {
    let mut dialog = app
        .dialog()
        .file()
        .add_filter("EisenScript", &["es", "eisenscript"])
        .add_filter("All files", &["*"]);

    if let Some(ref p) = current_path {
        if let Some(parent) = std::path::Path::new(p).parent() {
            dialog = dialog.set_directory(parent);
        }
        if let Some(name) = std::path::Path::new(p).file_name() {
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
    }

    let path = dialog.blocking_save_file();
    match path {
        Some(path) => {
            let path_str = path.to_string();
            std::fs::write(path.as_path().ok_or("Invalid path")?, &content)
                .map_err(|e| e.to_string())?;
            Ok(path_str)
        }
        None => Err("cancelled".to_string()),
    }
}

/// Run the pipeline and export the scene as OBJ.
#[tauri::command]
pub async fn export_obj(
    app: tauri::AppHandle,
    source: String,
    config: BuildConfig,
) -> Result<String, String> {
    let (scene, _, _) = pipeline::run_pipeline(&source, &config).map_err(|e| e.to_string())?;

    let path = app
        .dialog()
        .file()
        .add_filter("OBJ", &["obj"])
        .blocking_save_file();

    match path {
        Some(path) => {
            let path_str = path.to_string();
            let exporter = rustsynth_export_obj::ObjExporter::default();
            let output = exporter.export(&scene).map_err(|e| e.to_string())?;
            // Write both OBJ and MTL files
            let obj_path = path.as_path().ok_or("Invalid path")?;
            std::fs::write(obj_path, &output.obj)
                .map_err(|e| e.to_string())?;
            // Write MTL alongside the OBJ
            let mtl_path = obj_path.with_extension("mtl");
            std::fs::write(&mtl_path, &output.mtl)
                .map_err(|e| e.to_string())?;
            Ok(path_str)
        }
        None => Err("cancelled".to_string()),
    }
}

/// Run the pipeline and export the scene using a template.
#[tauri::command]
pub async fn export_template(
    app: tauri::AppHandle,
    source: String,
    config: BuildConfig,
    template_path: Option<String>,
) -> Result<String, String> {
    use rustsynth_export_template::{Template, TemplateExporter};

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

    let (scene, _, _) = pipeline::run_pipeline(&source, &config).map_err(|e| e.to_string())?;

    let template_xml = match template_path {
        Some(ref p) => std::fs::read_to_string(p).map_err(|e| e.to_string())?,
        None => BUILTIN_TEMPLATE_XML.to_string(),
    };

    let template = Template::from_xml(&template_xml).map_err(|e| e.to_string())?;
    let mut exporter = TemplateExporter::new(template);
    let text = exporter.export(&scene).map_err(|e| e.to_string())?;

    let path = app
        .dialog()
        .file()
        .add_filter("Text", &["txt"])
        .add_filter("All files", &["*"])
        .blocking_save_file();

    match path {
        Some(path) => {
            let path_str = path.to_string();
            std::fs::write(path.as_path().ok_or("Invalid path")?, &text)
                .map_err(|e| e.to_string())?;
            Ok(path_str)
        }
        None => Err("cancelled".to_string()),
    }
}
