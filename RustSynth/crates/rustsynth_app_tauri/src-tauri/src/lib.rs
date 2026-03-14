//! Tauri v2 backend for RustSynth.
//!
//! Exposes the headless EisenScript pipeline as Tauri commands, consumed by
//! the React + Three.js frontend via `invoke()`.

mod commands;
mod pipeline;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::run_script,
            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::export_obj,
            commands::export_template,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running RustSynth");
}
