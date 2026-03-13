//! RustSynth — GTK4 + Relm4 desktop application.
//!
//! Entry point: creates a `RelmApp` and launches the `AppModel` component.

mod app;
mod camera_io;
mod pipeline;
mod settings_dialog;
mod viewport;

fn main() {
    env_logger::init();
    log::info!("RustSynth starting");
    let app = relm4::RelmApp::new("io.rustsynth.app");
    app.run::<app::AppModel>(());
}
