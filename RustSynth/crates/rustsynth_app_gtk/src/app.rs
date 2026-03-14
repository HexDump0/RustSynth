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
use regex::Regex;
use relm4::gtk::gio;
use relm4::gtk::prelude::*;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use rustsynth_eval::{BuildConfig, RecursionMode};
use rustsynth_render_api::backend::{InputEvent, PointerButton, ViewportBackend};
use rustsynth_viewport_wgpu::WgpuBackend;

use crate::{camera_io, pipeline, settings_dialog, viewport};
use rustsynth_eisenscript::preprocessor::GuiParam;

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

const AUTOCOMPLETE_KEYWORDS: &[&str] = &[
    "set", "rule", "box", "sphere", "cylinder", "line", "dot", "grid", "triangle",
    "x", "y", "z", "s", "rx", "ry", "rz", "hue", "sat", "b", "a", "alpha",
    "minsize", "maxdepth", "maxobjects", "background", "seed", "translation", "rotation",
    "pivot", "scale", "camera", "if", "else",
];

// ─────────────────────────────────────────────────────────────────────────────
// Messages
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppMsg {
    // ── Script management ─────────────────────────────────────────────────────
    Run,
    ScriptChanged(String),
    TriggerAutocomplete,
    FrameTick,

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
    SetTemplateFile,
    TemplateFileSet(PathBuf),
    ClearTemplateFile,
    ExportCamera,
    CameraExportPicked(PathBuf),
    ImportCamera,
    CameraImportPicked(PathBuf),
    InsertCameraIntoScript,

    // ── GUI parameter panel ───────────────────────────────────────────────────
    GuiParamChanged { name: String, value: String },

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
    /// GUI parameters extracted from preprocessor `#define` directives.
    gui_params: Vec<GuiParam>,
    /// Custom template XML file path (None = use built-in template).
    template_path: Option<PathBuf>,
    /// Console/log output shown in the right-side Console tab.
    console_output: String,
}

impl AppModel {
    fn render_viewport(&mut self) -> Result<gtk::gdk::MemoryTexture> {
        let w = self.viewport_width.max(64);
        let h = self.viewport_height.max(64);
        viewport::render_scene_to_texture_no_reload(&mut self.backend, w, h)
    }

    fn apply_editor_highlighting(buffer: &gtk::TextBuffer) {
        let table = buffer.tag_table();
        if table.lookup("es-keyword").is_none() {
            let tag = gtk::TextTag::builder()
                .name("es-keyword")
                .foreground("#8aadf4")
                .weight(700)
                .build();
            table.add(&tag);
        }
        if table.lookup("es-number").is_none() {
            let tag = gtk::TextTag::builder()
                .name("es-number")
                .foreground("#f5a97f")
                .build();
            table.add(&tag);
        }
        if table.lookup("es-comment").is_none() {
            let tag = gtk::TextTag::builder()
                .name("es-comment")
                .foreground("#6e738d")
                .style(gtk::pango::Style::Italic)
                .build();
            table.add(&tag);
        }

        let (start, end) = buffer.bounds();
        let text = buffer.text(&start, &end, false).to_string();
        buffer.remove_all_tags(&start, &end);

        let keyword_re = Regex::new(r"\b(set|rule|box|sphere|cylinder|line|dot|grid|triangle|if|else)\b").ok();
        let number_re = Regex::new(r"(?m)(?<![A-Za-z_])[-+]?[0-9]*\.?[0-9]+(?![A-Za-z_])").ok();
        let comment_re = Regex::new(r"(?m)//.*$").ok();

        let apply_matches = |re: &Regex, tag_name: &str| {
            for m in re.find_iter(&text) {
                let start_char = text[..m.start()].chars().count() as i32;
                let end_char = text[..m.end()].chars().count() as i32;
                let it_start = buffer.iter_at_offset(start_char);
                let it_end = buffer.iter_at_offset(end_char);
                buffer.apply_tag_by_name(tag_name, &it_start, &it_end);
            }
        };

        if let Some(re) = keyword_re.as_ref() {
            apply_matches(re, "es-keyword");
        }
        if let Some(re) = number_re.as_ref() {
            apply_matches(re, "es-number");
        }
        if let Some(re) = comment_re.as_ref() {
            apply_matches(re, "es-comment");
        }
    }

    fn autocomplete_current_word(buffer: &gtk::TextBuffer) -> bool {
        let cursor_offset = buffer.cursor_position();
        let end = buffer.iter_at_offset(cursor_offset);
        let mut start = end;
        if !start.starts_word() {
            start.backward_word_start();
        }
        let prefix = buffer.text(&start, &end, false).to_string();
        if prefix.trim().is_empty() {
            return false;
        }

        let lower = prefix.to_lowercase();
        let mut matches: Vec<&str> = AUTOCOMPLETE_KEYWORDS
            .iter()
            .copied()
            .filter(|kw| kw.starts_with(&lower) && *kw != lower)
            .collect();

        if matches.is_empty() {
            return false;
        }
        matches.sort_unstable();
        let replacement = matches[0];

        let mut del_start = start;
        let mut del_end = end;
        buffer.delete(&mut del_start, &mut del_end);
        let mut insert_at = buffer.iter_at_offset(cursor_offset - prefix.chars().count() as i32);
        buffer.insert(&mut insert_at, replacement);
        true
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
    console_view: gtk::TextView,
    status_label: gtk::Label,
    seed_entry: gtk::Entry,
    max_obj_spin: gtk::SpinButton,
    recursion_combo: gtk::DropDown,
    var_panel: gtk::Box,
    var_expander: gtk::Expander,
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
        export_section.append(Some("Set Template File…"), Some("win.set-template-file"));
        export_section.append(Some("Clear Template File"), Some("win.clear-template-file"));
        file_menu.append_section(None, &export_section);
        let camera_section = gio::Menu::new();
        camera_section.append(Some("Export Camera…"), Some("win.export-camera"));
        camera_section.append(Some("Import Camera…"), Some("win.import-camera"));
        camera_section.append(Some("Insert Camera into Script"), Some("win.insert-camera"));
        file_menu.append_section(None, &camera_section);

        let menu_btn = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("File operations")
            .menu_model(&file_menu)
            .build();
        header.pack_start(&menu_btn);

        // ── Controls in Header (formerly Toolbar) ─────────────────────────────
        
        let controls_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .css_classes(["linked"])
            .build();
        header.pack_start(&controls_box);

        // Seed
        let seed_entry = gtk::Entry::builder()
            .text("0")
            .width_chars(6)
            .placeholder_text("Seed")
            .tooltip_text("RNG seed")
            .build();
        let sender_seed = sender.input_sender().clone();
        seed_entry.connect_activate(move |entry| {
            sender_seed.emit(AppMsg::SeedChanged(entry.text().to_string()));
        });
        controls_box.append(&seed_entry);

        // Max objects
        let obj_adj = gtk::Adjustment::new(100_000.0, 1.0, 10_000_000.0, 1000.0, 10_000.0, 0.0);
        let max_obj_spin = gtk::SpinButton::new(Some(&obj_adj), 1000.0, 0);
        max_obj_spin.set_tooltip_text(Some("Max objects"));
        let sender_obj = sender.input_sender().clone();
        max_obj_spin.connect_value_changed(move |spin| {
            sender_obj.emit(AppMsg::MaxObjectsChanged(spin.value()));
        });
        controls_box.append(&max_obj_spin);

        // Recursion mode
        let modes = gtk::StringList::new(&["BFS", "DFS"]);
        let recursion_combo = gtk::DropDown::new(Some(modes), gtk::Expression::NONE);
        recursion_combo.set_selected(0);
        recursion_combo.set_tooltip_text(Some("Recursion mode"));
        let sender_mode = sender.input_sender().clone();
        recursion_combo.connect_selected_notify(move |combo| {
            sender_mode.emit(AppMsg::RecursionModeChanged(combo.selected()));
        });
        controls_box.append(&recursion_combo);

        // Run button
        let run_btn = gtk::Button::builder()
            .label("Run")
            .icon_name("media-playback-start-symbolic")
            .css_classes(["suggested-action"])
            .tooltip_text("Run script (F5)")
            .build();
        let sender_run = sender.input_sender().clone();
        run_btn.connect_clicked(move |_| sender_run.emit(AppMsg::Run));
        header.pack_end(&run_btn);

        // Reset camera button
        let reset_btn = gtk::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Reset camera")
            .build();
        let sender_reset = sender.input_sender().clone();
        reset_btn.connect_clicked(move |_| sender_reset.emit(AppMsg::ResetCamera));
        header.pack_end(&reset_btn);

        // Settings
        let settings_btn = gtk::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Advanced settings")
            .build();
        let sender_settings = sender.input_sender().clone();
        settings_btn.connect_clicked(move |_| sender_settings.emit(AppMsg::ShowSettings));
        header.pack_end(&settings_btn);

        // ── Main paned view ───────────────────────────────────────────────────
        let paned = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .position(440)
            .vexpand(true)
            .build();
        vbox.append(&paned);

        // Left: code editor + variables panel
        let left_vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
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
            .tooltip_text("EisenScript editor — Ctrl+Space or Tab for autocomplete")
            .build();
        text_view.buffer().set_text(DEFAULT_SCRIPT);
        Self::apply_editor_highlighting(&text_view.buffer());
        editor_scroll.set_child(Some(&text_view));
        left_vbox.append(&editor_scroll);

        // Variables panel — shown automatically when the script has GUI #define params
        let var_expander = gtk::Expander::builder()
            .label("Variables")
            .expanded(true)
            .visible(false)
            .margin_start(4)
            .margin_end(4)
            .margin_bottom(4)
            .build();
        let var_panel = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .margin_start(4)
            .margin_end(4)
            .margin_top(2)
            .margin_bottom(4)
            .build();
        var_expander.set_child(Some(&var_panel));
        left_vbox.append(&var_expander);

        paned.set_start_child(Some(&left_vbox));

        // Keep model.source in sync with the buffer
        let sender_text = sender.input_sender().clone();
        text_view.buffer().connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let text = buf.text(&start, &end, false).to_string();
            AppModel::apply_editor_highlighting(buf);
            sender_text.emit(AppMsg::ScriptChanged(text));
        });

        // Ctrl+Space or Tab → autocomplete current token.
        {
            let key = gtk::EventControllerKey::new();
            let sender_auto = sender.input_sender().clone();
            key.connect_key_pressed(move |_, key, _code, state| {
                if key == gtk::gdk::Key::space
                    && state.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                {
                    sender_auto.emit(AppMsg::TriggerAutocomplete);
                    return gtk::glib::Propagation::Stop;
                }
                if key == gtk::gdk::Key::Tab {
                    sender_auto.emit(AppMsg::TriggerAutocomplete);
                    return gtk::glib::Propagation::Stop;
                }
                gtk::glib::Propagation::Proceed
            });
            text_view.add_controller(key);
        }

        // Right: split nav + content stack (Viewport / Console)
        let right_vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .build();
        let right_navbar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(4)
            .build();

        let right_stack = gtk::Stack::builder()
            .hexpand(true)
            .vexpand(true)
            .transition_type(gtk::StackTransitionType::Crossfade)
            .build();

        let switcher = gtk::StackSwitcher::builder()
            .stack(&right_stack)
            .build();
        right_navbar.append(&switcher);
        right_vbox.append(&right_navbar);

        // Viewport page
        let viewport_overlay = gtk::Overlay::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
            
        let picture = gtk::Picture::builder()
            .hexpand(true)
            .vexpand(true)
            .can_shrink(true)
            .content_fit(gtk::ContentFit::Fill) // Ensure texture fills the widget
            .tooltip_text("Viewport — drag to orbit, right-drag to pan, scroll to zoom")
            .build();
        viewport_overlay.set_child(Some(&picture));
        right_stack.add_titled(&viewport_overlay, Some("viewport"), "Viewport");

        // Console page
        let console_scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        let console_view = gtk::TextView::builder()
            .editable(false)
            .monospace(true)
            .left_margin(8)
            .right_margin(8)
            .top_margin(8)
            .bottom_margin(8)
            .cursor_visible(false)
            .build();
        console_view.buffer().set_text("RustSynth console ready.\n");
        console_scroll.set_child(Some(&console_view));
        right_stack.add_titled(&console_scroll, Some("console"), "Console");

        right_vbox.append(&right_stack);
        paned.set_end_child(Some(&right_vbox));

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

        // Frame tick (about 60Hz): coalesces camera/resize rerenders so
        // viewport interaction feels smoother and more native.
        {
            let tick_sender = sender.input_sender().clone();
            root.add_tick_callback(move |_, _| {
                tick_sender.emit(AppMsg::FrameTick);
                gtk::glib::ControlFlow::Continue
            });
        }

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
            gui_params: Vec::new(),
            template_path: None,
            console_output: "RustSynth console ready.\n".to_string(),
        };

        ComponentParts {
            model,
            widgets: AppWidgets {
                window: root,
                text_view,
                picture,
                console_view,
                status_label,
                seed_entry,
                max_obj_spin,
                recursion_combo,
                var_panel,
                var_expander,
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

            AppMsg::TriggerAutocomplete => {
                if Self::autocomplete_current_word(&widgets.text_view.buffer()) {
                    let buf = widgets.text_view.buffer();
                    let (start, end) = buf.bounds();
                    self.source = buf.text(&start, &end, false).to_string();
                    self.status = "Autocomplete applied".into();
                } else {
                    self.status = "No autocomplete suggestion".into();
                }
            }

            AppMsg::FrameTick => {
                let w = widgets.picture.allocated_width().max(1) as u32;
                let h = widgets.picture.allocated_height().max(1) as u32;
                if w != self.viewport_width || h != self.viewport_height {
                    self.viewport_width = w;
                    self.viewport_height = h;
                    if self.last_texture.is_some() {
                        self.needs_rerender = true;
                    }
                }

                if self.needs_rerender && self.last_texture.is_some() {
                    self.needs_rerender = false;
                    match self.render_viewport() {
                        Ok(tex) => self.last_texture = Some(tex),
                        Err(e) => log::warn!("Re-render after camera move failed: {e}"),
                    }
                }
            }

            AppMsg::Run => {
                self.run_script();
                Self::rebuild_var_panel(
                    &widgets.var_expander,
                    &widgets.var_panel,
                    &self.gui_params,
                    sender.input_sender(),
                );
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
                        // Restore embedded camera annotation if present
                        if let Some(cam) = camera_io::extract_camera_annotation(&content) {
                            *self.backend.camera_mut() = cam;
                        }
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
            // ── Camera I/O ────────────────────────────────────────────────────────────
            AppMsg::ExportCamera => {
                let dialog = gtk::FileDialog::builder()
                    .title("Export Camera")
                    .initial_name("camera.json")
                    .build();
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.json");
                filter.set_name(Some("Camera files (*.json)"));
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));
                let input = sender.input_sender().clone();
                dialog.save(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::CameraExportPicked(path));
                        }
                    }
                });
            }

            AppMsg::CameraExportPicked(path) => {
                self.export_camera_state(&path);
            }

            AppMsg::ImportCamera => {
                let dialog = gtk::FileDialog::builder()
                    .title("Import Camera")
                    .build();
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.json");
                filter.set_name(Some("Camera files (*.json)"));
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));
                let input = sender.input_sender().clone();
                dialog.open(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::CameraImportPicked(path));
                        }
                    }
                });
            }

            AppMsg::CameraImportPicked(path) => {
                self.import_camera_state(&path);
                self.needs_rerender = true;
            }

            AppMsg::InsertCameraIntoScript => {
                match camera_io::insert_camera_annotation(&self.source, self.backend.camera()) {
                    Ok(new_source) => {
                        self.source = new_source.clone();
                        widgets.text_view.buffer().set_text(&new_source);
                        self.status = "Camera state inserted into script.".into();
                    }
                    Err(e) => self.status = format!("Camera insert error: {e}"),
                }
            }

            // ── GUI parameter panel ───────────────────────────────────────────────────
            AppMsg::GuiParamChanged { name, value } => {
                let new_source = Self::rewrite_define_value(&self.source, &name, &value);
                if new_source != self.source {
                    self.source = new_source.clone();
                    let buf = widgets.text_view.buffer();
                    let (start, end) = buf.bounds();
                    if buf.text(&start, &end, false).as_str() != new_source {
                        buf.set_text(&new_source);
                    }
                    self.status = "Variables updated — press ▶ Run (F5) to re-render.".into();
                }
            }

            // ── Template file selection ─────────────────────────────────────────────
            AppMsg::SetTemplateFile => {
                let dialog = gtk::FileDialog::builder()
                    .title("Select Template XML")
                    .build();
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.xml");
                filter.set_name(Some("Template files (*.xml)"));
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));
                let input = sender.input_sender().clone();
                dialog.open(Some(root), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            input.emit(AppMsg::TemplateFileSet(path));
                        }
                    }
                });
            }

            AppMsg::TemplateFileSet(path) => {
                self.status = format!(
                    "Template: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                self.template_path = Some(path);
            }

            AppMsg::ClearTemplateFile => {
                self.template_path = None;
                self.status = "Template cleared — using built-in.".into();
            }
            // ── Viewport size ─────────────────────────────────────────────────
            AppMsg::ViewportResized(w, h) => {
                self.viewport_width = w;
                self.viewport_height = h;
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

        // Sync console output
        let console_buf = widgets.console_view.buffer();
        let (start, end) = console_buf.bounds();
        if console_buf.text(&start, &end, false).as_str() != self.console_output {
            console_buf.set_text(&self.console_output);
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
            ("set-template-file", AppMsg::SetTemplateFile),
            ("clear-template-file", AppMsg::ClearTemplateFile),
            ("export-camera", AppMsg::ExportCamera),
            ("import-camera", AppMsg::ImportCamera),
            ("insert-camera", AppMsg::InsertCameraIntoScript),
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
                    "set-template-file" => AppMsg::SetTemplateFile,
                    "clear-template-file" => AppMsg::ClearTemplateFile,
                    "export-camera" => AppMsg::ExportCamera,
                    "import-camera" => AppMsg::ImportCamera,
                    "insert-camera" => AppMsg::InsertCameraIntoScript,
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
            Ok((scene, warnings, gui_params)) => {
                self.gui_params = gui_params;
                self.object_count = scene.objects.len();
                self.console_output = if warnings.is_empty() {
                    format!("Run OK — {} objects\n", self.object_count)
                } else {
                    format!(
                        "Run OK — {} objects\nWarnings:\n{}\n",
                        self.object_count,
                        warnings.join("\n")
                    )
                };

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
                        self.console_output = format!("Render error:\n{e}\n");
                        log::error!("{e}");
                    }
                }
            }
            Err(e) => {
                self.status = format!("Script error: {e}");
                self.console_output = format!("Script error:\n{e}\n");
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
            Ok((scene, _, _)) => {
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
                                self.console_output.push_str(&format!(
                                    "OBJ exported to {}\n",
                                    path.display()
                                ));
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
    /// Uses the custom template XML set via `SetTemplateFile` if one is
    /// configured, otherwise falls back to the built-in plain-text template.
    fn export_template(&mut self, path: &PathBuf) {
        use rustsynth_export_template::{Template, TemplateExporter};
        // Built-in minimal template — outputs a human-readable object list.
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
        // Load template XML: custom file if set, otherwise the built-in.
        let template_xml = match &self.template_path {
            Some(tmpl_path) => match std::fs::read_to_string(tmpl_path) {
                Ok(xml) => xml,
                Err(e) => {
                    self.status = format!("Failed to read template file: {e}");
                    return;
                }
            },
            None => BUILTIN_TEMPLATE_XML.to_string(),
        };
        match pipeline::run_pipeline(&self.source, &self.config) {
            Ok((scene, _, _)) => {
                match Template::from_xml(&template_xml) {
                    Ok(template) => {
                        let mut exporter = TemplateExporter::new(template);
                        match exporter.export(&scene) {
                            Ok(text) => match std::fs::write(path, text) {
                                Ok(()) => {
                                    let tmpl_name = self
                                        .template_path
                                        .as_ref()
                                        .and_then(|p| p.file_name())
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_else(|| "built-in".to_string());
                                    self.status = format!(
                                        "Template exported: {} (template: {})",
                                        path.file_name().unwrap_or_default().to_string_lossy(),
                                        tmpl_name
                                    );
                                    self.console_output.push_str(&format!(
                                        "Template export ok: {}\n",
                                        path.display()
                                    ));
                                }
                                Err(e) => self.status = format!("Template write error: {e}"),
                            },
                            Err(e) => self.status = format!("Template export error: {e}"),
                        }
                    }
                    Err(e) => self.status = format!("Template parse error: {e}"),
                }
            }
            Err(e) => self.status = format!("Script error during export: {e}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // T18 — Variable editor helpers
    // ─────────────────────────────────────────────────────────────────────────────

    /// Rewrite a `#define NAME value [gui-metadata]` line in `source` with a new value.
    ///
    /// The GUI type annotation `(float:lo-hi)` or `(int:lo-hi)` at the end of
    /// the line is preserved so the slider range survives re-runs.
    fn rewrite_define_value(source: &str, name: &str, new_value: &str) -> String {
        let prefix = format!("#define {} ", name);
        source
            .lines()
            .map(|line| {
                if let Some(rest) = line.strip_prefix(&prefix) {
                    // Preserve optional GUI annotation after the value
                    let meta = if let Some(idx) = rest.find(" (") {
                        &rest[idx..]
                    } else {
                        ""
                    };
                    format!("{}{}{}", prefix, new_value, meta)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Clear and rebuild the variable panel from the current `gui_params`.
    ///
    /// Shows the `expander` when there are params, hides it when there are none.
    fn rebuild_var_panel(
        expander: &gtk::Expander,
        panel: &gtk::Box,
        gui_params: &[GuiParam],
        sender: &relm4::Sender<AppMsg>,
    ) {
        // Remove all existing rows
        while let Some(child) = panel.first_child() {
            panel.remove(&child);
        }
        if gui_params.is_empty() {
            expander.set_visible(false);
            return;
        }
        expander.set_visible(true);
        expander.set_label(Some(&format!("Variables ({})", gui_params.len())));

        for param in gui_params {
            let row = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(6)
                .margin_start(2)
                .margin_end(2)
                .build();
            let lbl_text = match param {
                GuiParam::Float { name, .. } | GuiParam::Int { name, .. } => name.as_str(),
            };
            let label = gtk::Label::builder()
                .label(lbl_text)
                .width_chars(14)
                .xalign(0.0)
                .build();
            row.append(&label);

            match param {
                GuiParam::Float { name, default, min, max } => {
                    let step = (*max - *min) / 100.0;
                    let adj = gtk::Adjustment::new(*default, *min, *max, step.max(0.0001), step.max(0.001) * 10.0, 0.0);
                    let scale = gtk::Scale::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .adjustment(&adj)
                        .hexpand(true)
                        .draw_value(true)
                        .value_pos(gtk::PositionType::Right)
                        .build();
                    scale.set_digits(4);
                    let name_clone = name.clone();
                    let sender_clone = sender.clone();
                    scale.connect_value_changed(move |s| {
                        sender_clone.emit(AppMsg::GuiParamChanged {
                            name: name_clone.clone(),
                            value: format!("{:.6}", s.value()),
                        });
                    });
                    row.append(&scale);
                }
                GuiParam::Int { name, default, min, max } => {
                    let adj = gtk::Adjustment::new(
                        *default as f64, *min as f64, *max as f64, 1.0, 10.0, 0.0,
                    );
                    let spin = gtk::SpinButton::builder()
                        .adjustment(&adj)
                        .climb_rate(1.0)
                        .digits(0)
                        .hexpand(true)
                        .build();
                    let name_clone = name.clone();
                    let sender_clone = sender.clone();
                    spin.connect_value_changed(move |s| {
                        sender_clone.emit(AppMsg::GuiParamChanged {
                            name: name_clone.clone(),
                            value: format!("{}", s.value() as i64),
                        });
                    });
                    row.append(&spin);
                }
            }
            panel.append(&row);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // T19 — Camera I/O helpers
    // ─────────────────────────────────────────────────────────────────────────────

    /// Serialize the active camera to a JSON file.
    fn export_camera_state(&mut self, path: &PathBuf) {
        match camera_io::save_camera(self.backend.camera(), path) {
            Ok(()) => {
                self.status = format!(
                    "Camera exported: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            Err(e) => self.status = format!("Camera export error: {e}"),
        }
    }

    /// Deserialize a camera from a JSON file and apply it to the backend.
    fn import_camera_state(&mut self, path: &PathBuf) {
        match camera_io::load_camera(path) {
            Ok(cam) => {
                *self.backend.camera_mut() = cam;
                self.status = format!(
                    "Camera imported: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            Err(e) => self.status = format!("Camera import error: {e}"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::AppModel;

    #[test]
    fn rewrite_plain_define() {
        let src = "#define BRANCHANGLE 4\nset maxdepth 200";
        let result = AppModel::rewrite_define_value(src, "BRANCHANGLE", "7.5");
        assert_eq!(result, "#define BRANCHANGLE 7.5\nset maxdepth 200");
    }

    #[test]
    fn rewrite_gui_define_preserves_metadata() {
        let src = "#define BRANCHANGLE 4 (float:1.0-15.0)\nset maxdepth 200";
        let result = AppModel::rewrite_define_value(src, "BRANCHANGLE", "9");
        assert_eq!(result, "#define BRANCHANGLE 9 (float:1.0-15.0)\nset maxdepth 200");
    }

    #[test]
    fn rewrite_define_noop_when_name_absent() {
        let src = "#define OTHER 4\nset maxdepth 200";
        let result = AppModel::rewrite_define_value(src, "BRANCHANGLE", "9");
        assert_eq!(result, src);
    }

    #[test]
    fn rewrite_int_define_preserves_metadata() {
        let src = "#define MAXREC 9 (int:1-20)\nsome rule {}";
        let result = AppModel::rewrite_define_value(src, "MAXREC", "15");
        assert_eq!(result, "#define MAXREC 15 (int:1-20)\nsome rule {}");
    }
}
