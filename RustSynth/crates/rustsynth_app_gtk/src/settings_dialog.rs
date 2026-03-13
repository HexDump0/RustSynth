//! Settings dialog — exposes [`BuildConfig`] fields in a modal GTK window.

use relm4::gtk;
use relm4::gtk::prelude::*;
use rustsynth_eval::{BuildConfig, RecursionMode};

/// Show a modal settings window.
///
/// When the user clicks **Apply**, the callback `on_apply` is called with the
/// updated [`BuildConfig`].
pub fn show<F>(parent: &gtk::ApplicationWindow, current: &BuildConfig, on_apply: F)
where
    F: Fn(BuildConfig) + 'static,
{
    let dialog = gtk::Window::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .default_width(380)
        .resizable(false)
        .build();

    // ── Layout ────────────────────────────────────────────────────────────────
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(16)
        .spacing(8)
        .build();
    dialog.set_child(Some(&vbox));

    // Helper to create a labelled row.
    let row = |label: &str| -> (gtk::Box, gtk::Label) {
        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();
        let lbl = gtk::Label::builder()
            .label(label)
            .xalign(0.0)
            .hexpand(true)
            .build();
        hbox.append(&lbl);
        (hbox, lbl)
    };

    // Max generations
    let (gen_row, _) = row("Max generations:");
    let gen_adj = gtk::Adjustment::new(
        current.max_generations as f64,
        1.0,
        100_000.0,
        100.0,
        1000.0,
        0.0,
    );
    let gen_spin = gtk::SpinButton::new(Some(&gen_adj), 100.0, 0);
    gen_row.append(&gen_spin);
    vbox.append(&gen_row);

    // Max objects
    let (obj_row, _) = row("Max objects:");
    let obj_adj = gtk::Adjustment::new(
        current.max_objects as f64,
        1.0,
        10_000_000.0,
        1000.0,
        10_000.0,
        0.0,
    );
    let obj_spin = gtk::SpinButton::new(Some(&obj_adj), 1000.0, 0);
    obj_row.append(&obj_spin);
    vbox.append(&obj_row);

    // Min / max object size
    let (min_row, _) = row("Min object size (0 = off):");
    let min_adj = gtk::Adjustment::new(current.min_dim as f64, 0.0, 1000.0, 0.001, 0.1, 0.0);
    let min_spin = gtk::SpinButton::new(Some(&min_adj), 0.001, 4);
    min_row.append(&min_spin);
    vbox.append(&min_row);

    let (max_row, _) = row("Max object size (0 = off):");
    let max_adj = gtk::Adjustment::new(current.max_dim as f64, 0.0, 1000.0, 0.001, 0.1, 0.0);
    let max_spin = gtk::SpinButton::new(Some(&max_adj), 0.001, 4);
    max_row.append(&max_spin);
    vbox.append(&max_row);

    // Recursion mode
    let (mode_row, _) = row("Recursion mode:");
    let modes = gtk::StringList::new(&["BFS (breadth-first)", "DFS (depth-first)"]);
    let mode_combo = gtk::DropDown::new(Some(modes), gtk::Expression::NONE);
    mode_combo.set_selected(match current.mode {
        RecursionMode::BreadthFirst => 0,
        RecursionMode::DepthFirst => 1,
    });
    mode_row.append(&mode_combo);
    vbox.append(&mode_row);

    // Sync-random
    let (sync_row, _) = row("Sync random (legacy):");
    let sync_switch = gtk::Switch::builder()
        .active(current.sync_random)
        .valign(gtk::Align::Center)
        .build();
    sync_row.append(&sync_switch);
    vbox.append(&sync_row);

    // ── Buttons ───────────────────────────────────────────────────────────────
    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .margin_top(8)
        .build();

    let cancel_btn = gtk::Button::builder().label("Cancel").build();
    let apply_btn = gtk::Button::builder()
        .label("Apply")
        .css_classes(["suggested-action"])
        .build();

    btn_box.append(&cancel_btn);
    btn_box.append(&apply_btn);
    vbox.append(&btn_box);

    // ── Signal connections ────────────────────────────────────────────────────
    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });

    let dialog_apply = dialog.clone();
    apply_btn.connect_clicked(move |_| {
        let mode = if mode_combo.selected() == 0 {
            RecursionMode::BreadthFirst
        } else {
            RecursionMode::DepthFirst
        };
        let new_config = BuildConfig {
            max_generations: gen_spin.value() as u32,
            max_objects: obj_spin.value() as usize,
            min_dim: min_spin.value() as f32,
            max_dim: max_spin.value() as f32,
            sync_random: sync_switch.is_active(),
            mode,
            // seed is managed separately in the toolbar
            seed: 0,
        };
        on_apply(new_config);
        dialog_apply.close();
    });

    dialog.present();
}
