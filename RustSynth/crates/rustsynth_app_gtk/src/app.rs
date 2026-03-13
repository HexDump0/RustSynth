//! Main application model — GTK4 + Relm4 desktop shell.
//!
//! Layout:
//! ```text
//! ApplicationWindow
//!   ├── HeaderBar  (title, File menu, Run button)
//!   ├── Toolbar    (seed, max-objects, recursion mode, settings)
//!   ├── Paned      (horizontal)
//!   │    ├── LEFT  ScrolledWindow → TextView  (code editor)
//!   │    └── RIGHT ScrolledWindow → Picture   (rendered viewport)
//!   └── Label      (status bar)
//! ```

use std::path::PathBuf;

use anyhow::Result;
use relm4::gtk::gio;
use relm4::gtk::prelude::*;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use rustsynth_eval::{BuildConfig, RecursionMode};
use rustsynth_render_api::backend::{InputEvent, PointerButton, ViewportBackend};
use rustsynth_viewport_wgpu::WgpuBackend;

use crate::{pipeline, settings_dialog, viewport};

// ─────────────────────────────────────────────────────────────────────────────
// Default script shown on first launch
// ─────────────────────────────────────────────────────────────────────────────

const DEFAULT_SCRIPT: &str = r#"set background #111
set maxdepth 200

r0

rule r0 {
  3 * { rz 120 } R1
  3 * { rz 120 } R2
}

rule R1 {
  { x 1.3 rx 1.57 rz 6 ry 3 s 0.99 hue 0.4 sat 0.99 } R1
  { s 4 } sphere
}

rule R2 {
  { x -1.3 rz 6 ry 3 s 0.99 hue 0.4 sat 0.99 } R2
  { s 4 } box
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// Messages
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppMsg {
    // ── Script management ─────────────────────────────────────────────────────
    Run,
    ScriptChanged(String),

    // ── File I/O ──────────────────────────────────────────────────────────────
    NewFile,
    OpenFile,
    FileOpened(PathBuf),
    SaveFile,
    SaveFileAs,
    FileSaved(PathBuf),

    // ── Export ────────────────────────────────────────────────────────────────
    ExportObj,
    ExportObjPicked(PathBuf),
    ExportTemplate,
    ExportTemplatePicked(PathBuf),

    // ── Configuration ─────────────────────────────────────────────────────────
    ShowSettings,
    SettingsUpdated(BuildConfig),
    SeedChanged(String),
    MaxObjectsChanged(f64),
    RecursionModeChanged(u32),

    // ── Camera ────────────────────────────────────────────────────────────────
    OrbitCamera(f64, f64),
    PanCamera(f64, f64),
    Scroll(f64),
    ResetCamera,

    // ── Viewport size ─────────────────────────────────────────────────────────
    ViewportResized(u32, u32),
}

// ─────────────────────────────────────────────────────────────────────────────
// Model
// ─────────────────────────────────────────────────────────────────────────────

pub struct AppModel {
    /// Current EisenScript source text (kept in sync with the text buffer).
    source: String,
    /// Build configuration (seed, limits, recursion mode).
    config: BuildConfig,
    /// Currently open file path (None for unsaved scripts).
    file_path: Option<PathBuf>,
    /// Message shown in the status bar.
    status: String,
    /// Number of objects in the last successfully rendered scene.
    object_count: usize,
    /// wgpu renderer backend.
    backend: WgpuBackend,
    /// Last rendered GDK texture for the viewport picture widget.
    last_texture: Option<gtk::gdk::MemoryTexture>,
    /// Current viewport pixel dimensions.
    viewport_width: u32,
    viewport_height: u32,
    /// Whether we need to re-render on the next update_view pass.
    needs_rerender: bool,
}

impl AppModel {
    fn render_viewport(&mut self) -> Result<gtk::gdk::MemoryTexture> {
        let w = self.viewport_width.max(64);
        let h = self.viewport_height.max(64);
        viewport::render_scene_to_texture_no_reload(&mut self.backend, w, h)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Widgets
// ─────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct AppWidgets {
    window: gtk::ApplicationWindow,
    text_view: gtk::TextView,
    picture: gtk::Picture,
    status_label: gtk::Label,
    seed_entry: gtk::Entry,
    max_obj_spin: gtk::SpinButton,
    recursion_combo: gtk::DropDown,
}

// ─────────────────────────────────────────────────────────────────────────────
// Component implementation
// ─────────────────────────────────────────────────────────────────────────────

impl Component for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = AppWidgets;

    fn init_root() -> gtk::ApplicationWindow {
        gtk::ApplicationWindow::builder()
            .title("RustSynth")
            .default_width(1200)
            .default_height(720)
            .build()
    }

    fn init(
        _init: (),
        root: gtk::ApplicationWindow,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // ── Outer container ───────────────────────────────────────────────────
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        root.set_child(Some(&vbox));

        // ── Header bar ────────────────────────────────────────────────────────
        let header = gtk::HeaderBar::new();
        root.set_titlebar(Some(&header));

        // File menu
        let file_menu = gio::Menu::new();
        file_menu.append(Some("New"), Some("win.new-file"));
        file_menu.append(Some("Open…"), Some("win.open-file"));
        file_menu.append(Some("Save"), Some("win.save-file"));
        file_menu.append(Some("Save As…"), Some("win.save-file-as"));
        let export_section = gio::Menu::new();
        export_section.append(Some("Export OBJ…"), Some("win.export-obj"));
        export_section.append(Some("Export Template…"), Some("win.export-template"));
        file_menu.append_section(None, &export_section);

        let menu_btn = gtk::MenuButton::builder()
            .label("File")
            .menu_model(&file_menu)
            .build();
        header.pack_start(&menu_btn);

        // Run button
        let run_btn = gtk::Button::builder()
            .label("▶  Run")
            .css_classes(["suggested-action"])
            .tooltip_text("Run script (F5)")
            .build();
        let sender_run = sender.input_sender().clone();
        run_btn.connect_clicked(move |_| sender_run.emit(AppMsg::Run));
        header.pack_end(&run_btn);

        // Reset camera button
        let reset_btn = gtk::Button::builder()
            .label("⟳ Camera")
            .tooltip_text("Reset camera")
            .build();
        let sender_reset = sender.input_sender().clone();
        reset_btn.connect_clicked(move |_| sender_reset.emit(AppMsg::ResetCamera));
        header.pack_end(&reset_btn);

        // ── Toolbar ───────────────────────────────────────────────────────────
        let toolbar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .spacing(8)
            .build();
        vbox.append(&toolbar);

        // Seed
        toolbar.append(
            &gtk::Label::builder()
                .label("Seed:")
                .css_classes(["dim-label"])
                .build(),
        );
        let seed_entry = gtk::Entry::builder()
            .text("0")
            .width_chars(8)
            .tooltip_text("RNG seed — change to get different variants")
            .build();
        let sender_seed = sender.input_sender().clone();
        seed_entry.connect_activate(move |entry| {
            sender_seed.emit(AppMsg::SeedChanged(entry.text().to_string()));
        });
        toolbar.append(&seed_entry);

        // Separator
        toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        // Max objects
        toolbar.append(
            &gtk::Label::builder()
                .label("Max objects:")
                .css_classes(["dim-label"])
                .build(),
        );
        let obj_adj = gtk::Adjustment::new(100_000.0, 1.0, 10_000_000.0, 1000.0, 10_000.0, 0.0);
        let max_obj_spin = gtk::SpinButton::new(Some(&obj_adj), 1000.0, 0);
        let sender_obj = sender.input_sender().clone();
        max_obj_spin.connect_value_changed(move |spin| {
            sender_obj.emit(AppMsg::MaxObjectsChanged(spin.value()));
        });
        toolbar.append(&max_obj_spin);

        // Separator
        toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        // Recursion mode
        toolbar.append(
            &gtk::Label::builder()
                .label("Mode:")
                .css_classes(["dim-label"])
                .build(),
        );
        let modes = gtk::StringList::new(&["BFS", "DFS"]);
        let recursion_combo = gtk::DropDown::new(Some(modes), gtk::Expression::NONE);
        recursion_combo.set_selected(0);
        let sender_mode = sender.input_sender().clone();
        recursion_combo.connect_selected_notify(move |combo| {
            sender_mode.emit(AppMsg::RecursionModeChanged(combo.selected()));
        });
        toolbar.append(&recursion_combo);

        // Separator
        toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        // Settings
        let settings_btn = gtk::Button::builder()
            .label("⚙ Settings")
            .tooltip_text("Advanced build settings")
            .build();
        let sender_settings = sender.input_sender().clone();
        settings_btn.connect_clicked(move |_| sender_settings.emit(AppMsg::ShowSettings));
        toolbar.append(&settings_btn);

        // ── Main paned view ───────────────────────────────────────────────────
        let paned = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .position(440)
            .vexpand(true)
            .build();
        vbox.append(&paned);

        // Left: code editor
        let editor_scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .min_content_width(300)
            .build();
        let text_view = gtk::TextView::builder()
            .monospace(true)
            .left_margin(8)
            .right_margin(8)
            .top_margin(8)
            .bottom_margin(8)
            .tooltip_text("EisenScript editor")
            .build();
        text_view.buffer().set_text(DEFAULT_SCRIPT);
        editor_scroll.set_child(Some(&text_view));
        paned.set_start_child(Some(&editor_scroll));

        // Keep model.source in sync with the buffer
        let sender_text = sender.input_sender().clone();
        text_view.buffer().connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let text = buf.text(&start, &end, false).to_string();
            sender_text.emit(AppMsg::ScriptChanged(text));
        });

        // Right: viewport picture
        let viewport_scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .min_content_width(300)
            .build();
        let picture = gtk::Picture::builder()
            .hexpand(true)
            .vexpand(true)
            .can_shrink(true)
            .tooltip_text("Viewport — drag to orbit, right-drag to pan, scroll to zoom")
            .build();

        // Camera gesture controllers
        // Primary drag → orbit
        {
            use std::cell::Cell;
            use std::rc::Rc;
            let prev = Rc::new(Cell::new((0.0f64, 0.0f64)));
            let drag = gtk::GestureDrag::new();
            drag.set_button(gtk::gdk::BUTTON_PRIMARY);
            let prev_begin = prev.clone();
            drag.connect_drag_begin(move |_, _, _| prev_begin.set((0.0, 0.0)));
            let prev_update = prev.clone();
            let sender_orbit = sender.input_sender().clone();
            drag.connect_drag_update(move |_, ox, oy| {
                let (px, py) = prev_update.get();
                sender_orbit.emit(AppMsg::OrbitCamera(ox - px, oy - py));
                prev_update.set((ox, oy));
            });
            picture.add_controller(drag);
        }

        // Secondary drag → pan
        {
            use std::cell::Cell;
            use std::rc::Rc;
            let prev = Rc::new(Cell::new((0.0f64, 0.0f64)));
            let drag = gtk::GestureDrag::new();
            drag.set_button(gtk::gdk::BUTTON_SECONDARY);
            let prev_begin = prev.clone();
            drag.connect_drag_begin(move |_, _, _| prev_begin.set((0.0, 0.0)));
            let prev_update = prev.clone();
            let sender_pan = sender.input_sender().clone();
            drag.connect_drag_update(move |_, ox, oy| {
                let (px, py) = prev_update.get();
                sender_pan.emit(AppMsg::PanCamera(ox - px, oy - py));
                prev_update.set((ox, oy));
            });
            picture.add_controller(drag);
        }

        // Scroll → zoom
        {
            let scroll = gtk::EventControllerScroll::new(
                gtk::EventControllerScrollFlags::VERTICAL,
            );
            let sender_scroll = sender.input_sender().clone();
            scroll.connect_scroll(move |_, _dx, dy| {
                sender_scroll.emit(AppMsg::Scroll(dy));
                gtk::glib::Propagation::Proceed
            });
            picture.add_controller(scroll);
        }

        viewport_scroll.set_child(Some(&picture));
        paned.set_end_child(Some(&viewport_scroll));

        // ── Status bar ────────────────────────────────────────────────────────
        let status_label = gtk::Label::builder()
            .label("Ready — press ▶ Run (or F5) to render the script.")
            .xalign(0.0)
            .margin_start(8)
            .margin_end(8)
            .margin_top(3)
            .margin_bottom(3)
            .css_classes(["dim-label"])
            .build();
        vbox.append(&status_label);

        // ── Window actions (for menu items and keyboard shortcuts) ────────────
        Self::register_actions(&root, sender.input_sender());

        // F5 shortcut → Run
        let shortcut_run = gtk::Shortcut::new(
            Some(gtk::ShortcutTrigger::parse_string("F5").unwrap()),
            Some(gtk::NamedAction::new("win.run")),
        );
        let shortcut_ctrl = gtk::ShortcutController::new();
        shortcut_ctrl.set_scope(gtk::ShortcutScope::Managed);
        shortcut_ctrl.add_shortcut(shortcut_run);
        root.add_controller(shortcut_ctrl);

        // ── Initialise backend ────────────────────────────────────────────────
        let mut backend = WgpuBackend::new();
        let status = match backend.init() {
            Ok(()) => "GPU initialized. Press ▶ Run to render.".to_string(),
            Err(e) => {
                log::error!("GPU init failed: {e}");
                format!("GPU init failed: {e}")
            }
        };

        let model = AppModel {
            source: DEFAULT_SCRIPT.to_string(),
            config: BuildConfig::default(),
            file_path: None,
            status,
            object_count: 0,
            backend,
            last_texture: None,
            viewport_width: 600,
            viewport_height: 500,
            needs_rerender: false,
        };

        ComponentParts {
            model,
            widgets: AppWidgets {
                window: root,
                text_view,
                picture,
                status_label,
                seed_entry,
                max_obj_spin,
                recursion_combo,
            },
        }
    }

    // ── Main message handler (with widget access) ─────────────────────────────

    fn update_with_view(
        &mut self,
        widgets: &mut AppWidgets,
        message: AppMsg,
        sender: ComponentSender<Self>,
        root: &gtk::ApplicationWindow,
    ) {
        match message {
            // ── Script ────────────────────────────────────────────────────────
            AppMsg::ScriptChanged(text) => {
                self.source = text;
            }

            AppMsg::Run => {
                self.run_script();
            }

            // ── File I/O ──────────────────────────────────────────────────────
            AppMsg::NewFile => {
                widgets.text_view.buffer().set_text(DEFAULT_SCRIPT);
                self.source = DEFAULT_SCRIPT.to_string();
                self.file_path = None;
                self.last_texture = None;
                self.status = "New file.".into();
            }

            AppMsg::OpenFile => {
                let dialog = gtk::FileDialog::builder()
                    .title("Open EisenScript")
                    .build();
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.es");
                filter.set_name(Some("EisenScript (*.es)"));
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));

                let input = sender.input_sender().clone();
                dialog.open(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::FileOpened(path));
                        }
                    }
                });
            }

            AppMsg::FileOpened(path) => {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        widgets.text_view.buffer().set_text(&content);
                        self.source = content;
                        self.file_path = Some(path.clone());
                        self.status = format!(
                            "Opened: {}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                    }
                    Err(e) => self.status = format!("Error opening file: {e}"),
                }
            }

            AppMsg::SaveFile => {
                if let Some(path) = self.file_path.clone() {
                    self.save_to_path(&path);
                } else {
                    sender.input(AppMsg::SaveFileAs);
                }
            }

            AppMsg::SaveFileAs => {
                let dialog = gtk::FileDialog::builder()
                    .title("Save EisenScript")
                    .initial_name("script.es")
                    .build();
                let input = sender.input_sender().clone();
                dialog.save(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::FileSaved(path));
                        }
                    }
                });
            }

            AppMsg::FileSaved(path) => {
                self.file_path = Some(path.clone());
                self.save_to_path(&path);
            }

            // ── Export ────────────────────────────────────────────────────────
            AppMsg::ExportObj => {
                let dialog = gtk::FileDialog::builder()
                    .title("Export OBJ")
                    .initial_name("scene.obj")
                    .build();
                let input = sender.input_sender().clone();
                dialog.save(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::ExportObjPicked(path));
                        }
                    }
                });
            }

            AppMsg::ExportObjPicked(path) => {
                self.export_obj(&path);
            }

            AppMsg::ExportTemplate => {
                let dialog = gtk::FileDialog::builder()
                    .title("Export Template")
                    .initial_name("scene.txt")
                    .build();
                let input = sender.input_sender().clone();
                dialog.save(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::ExportTemplatePicked(path));
                        }
                    }
                });
            }

            AppMsg::ExportTemplatePicked(path) => {
                self.export_template(&path);
            }

            // ── Config ────────────────────────────────────────────────────────
            AppMsg::ShowSettings => {
                let config = self.config.clone();
                let input = sender.input_sender().clone();
                settings_dialog::show(root, &config, move |new_cfg| {
                    input.emit(AppMsg::SettingsUpdated(new_cfg));
                });
            }

            AppMsg::SettingsUpdated(mut new_cfg) => {
                // Preserve seed from toolbar
                new_cfg.seed = self.config.seed;
                self.config = new_cfg;
                self.status = "Settings updated.".into();
            }

            AppMsg::SeedChanged(text) => {
                if let Ok(seed) = text.trim().parse::<u64>() {
                    self.config.seed = seed;
                }
            }

            AppMsg::MaxObjectsChanged(v) => {
                self.config.max_objects = v as usize;
            }

            AppMsg::RecursionModeChanged(idx) => {
                self.config.mode = if idx == 0 {
                    RecursionMode::BreadthFirst
                } else {
                    RecursionMode::DepthFirst
                };
            }

            // ── Camera ────────────────────────────────────────────────────────
            AppMsg::OrbitCamera(dx, dy) => {
                self.backend.handle_input(InputEvent::PointerDrag {
                    button: PointerButton::Primary,
                    dx: dx as f32,
                    dy: dy as f32,
                });
                self.needs_rerender = true;
            }

            AppMsg::PanCamera(dx, dy) => {
                self.backend.handle_input(InputEvent::Pan {
                    dx: dx as f32,
                    dy: dy as f32,
                });
                self.needs_rerender = true;
            }

            AppMsg::Scroll(dy) => {
                self.backend.handle_input(InputEvent::Scroll {
                    delta: -dy as f32,
                });
                self.needs_rerender = true;
            }

            AppMsg::ResetCamera => {
                self.backend.handle_input(InputEvent::ResetCamera);
                self.needs_rerender = true;
            }

            // ── Viewport size ─────────────────────────────────────────────────
            AppMsg::ViewportResized(w, h) => {
                self.viewport_width = w;
                self.viewport_height = h;
            }
        }

        // Re-render if camera moved (and we have a loaded scene)
        if self.needs_rerender && self.last_texture.is_some() {
            self.needs_rerender = false;
            match self.render_viewport() {
                Ok(tex) => self.last_texture = Some(tex),
                Err(e) => log::warn!("Re-render after camera move failed: {e}"),
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_view(&self, widgets: &mut AppWidgets, _sender: ComponentSender<Self>) {
        // Update viewport picture
        if let Some(tex) = &self.last_texture {
            widgets.picture.set_paintable(Some(tex));
        }
        // Update status bar
        widgets.status_label.set_text(&self.status);
        // Sync seed entry (only if the value differs to avoid caret reset)
        let seed_text = self.config.seed.to_string();
        if widgets.seed_entry.text().as_str() != seed_text {
            widgets.seed_entry.set_text(&seed_text);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

impl AppModel {
    /// Register window actions (for menu items & keyboard shortcuts).
    fn register_actions(
        root: &gtk::ApplicationWindow,
        input: &relm4::Sender<AppMsg>,
    ) {
        let actions: &[(&str, AppMsg)] = &[
            ("new-file", AppMsg::NewFile),
            ("open-file", AppMsg::OpenFile),
            ("save-file", AppMsg::SaveFile),
            ("save-file-as", AppMsg::SaveFileAs),
            ("export-obj", AppMsg::ExportObj),
            ("export-template", AppMsg::ExportTemplate),
            ("run", AppMsg::Run),
        ];
        for (name, msg_template) in actions {
            let action = gio::SimpleAction::new(name, None);
            let input_clone = input.clone();
            // Each arm must clone because we can't move out of a reference pattern
            let name_owned = name.to_string();
            action.connect_activate(move |_, _| {
                let msg = match name_owned.as_str() {
                    "new-file" => AppMsg::NewFile,
                    "open-file" => AppMsg::OpenFile,
                    "save-file" => AppMsg::SaveFile,
                    "save-file-as" => AppMsg::SaveFileAs,
                    "export-obj" => AppMsg::ExportObj,
                    "export-template" => AppMsg::ExportTemplate,
                    "run" => AppMsg::Run,
                    _ => return,
                };
                input_clone.emit(msg);
            });
            root.add_action(&action);
            let _ = msg_template; // suppress unused warning
        }
    }

    /// Run the current script and render the result.
    fn run_script(&mut self) {
        match pipeline::run_pipeline(&self.source, &self.config) {
            Ok((scene, warnings)) => {
                self.object_count = scene.objects.len();

                match viewport::render_scene_to_texture(
                    &mut self.backend,
                    &scene,
                    self.viewport_width.max(64),
                    self.viewport_height.max(64),
                ) {
                    Ok(tex) => {
                        self.last_texture = Some(tex);
                        let warn_str = if warnings.is_empty() {
                            String::new()
                        } else {
                            format!("  ({} warnings)", warnings.len())
                        };
                        self.status = format!(
                            "✓ {} objects rendered{}",
                            self.object_count, warn_str
                        );
                        log::info!("Run complete: {} objects", self.object_count);
                        for w in &warnings {
                            log::warn!("{}", w);
                        }
                    }
                    Err(e) => {
                        self.status = format!("Render error: {e}");
                        log::error!("{e}");
                    }
                }
            }
            Err(e) => {
                self.status = format!("Script error: {e}");
                log::error!("{e}");
            }
        }
    }

    /// Write `self.source` to `path` and update status.
    fn save_to_path(&mut self, path: &PathBuf) {
        match std::fs::write(path, &self.source) {
            Ok(()) => {
                self.status = format!(
                    "Saved: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            Err(e) => self.status = format!("Save error: {e}"),
        }
    }

    /// Export the current scene as OBJ to `path`.
    fn export_obj(&mut self, path: &PathBuf) {
        match pipeline::run_pipeline(&self.source, &self.config) {
            Ok((scene, _)) => {
                use rustsynth_export_obj::ObjExporter;
                let exporter = ObjExporter::default();
                match exporter.export(&scene) {
                    Ok(output) => {
                        let result = std::fs::write(path, &output.obj).and_then(|_| {
                            let mtl_path = path.with_extension("mtl");
                            std::fs::write(mtl_path, &output.mtl)
                        });
                        match result {
                            Ok(()) => {
                                self.status = format!(
                                    "OBJ exported: {}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                );
                            }
                            Err(e) => self.status = format!("OBJ write error: {e}"),
                        }
                    }
                    Err(e) => self.status = format!("OBJ export error: {e}"),
                }
            }
            Err(e) => self.status = format!("Script error during export: {e}"),
        }
    }

    /// Export the current scene as a template to `path`.
    ///
    /// Uses a minimal built-in plain-text template.  For production use, the
    /// user would select a template XML file first (a future enhancement).
    fn export_template(&mut self, path: &PathBuf) {
        use rustsynth_export_template::{Template, TemplateExporter};
        // Minimal built-in template that outputs a human-readable object list.
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
        match pipeline::run_pipeline(&self.source, &self.config) {
            Ok((scene, _)) => {
                match Template::from_xml(BUILTIN_TEMPLATE_XML) {
                    Ok(template) => {
                        let mut exporter = TemplateExporter::new(template);
                        match exporter.export(&scene) {
                            Ok(text) => match std::fs::write(path, text) {
                                Ok(()) => {
                                    self.status = format!(
                                        "Template exported: {}",
                                        path.file_name().unwrap_or_default().to_string_lossy()
                                    );
                                }
                                Err(e) => self.status = format!("Template write error: {e}"),
                            },
                            Err(e) => self.status = format!("Template export error: {e}"),
                        }
                    }
                    Err(e) => self.status = format!("Internal template error: {e}"),
                }
            }
            Err(e) => self.status = format!("Script error during export: {e}"),
        }
    }
}
