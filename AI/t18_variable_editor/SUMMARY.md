# T18 — Variable editor and parameter controls

## Task goal
Implement a live variable-editor panel that surfaces the preprocessor-driven
`#define NAME value (float:lo-hi)` and `#define NAME value (int:lo-hi)`
parameters as interactive GTK sliders and spinbuttons inside the app shell.

## Approach

### Pipeline change (`pipeline.rs`)
- Added `pub use rustsynth_eisenscript::preprocessor::GuiParam;` re-export so
  callers don't need to reach into the eisenscript internals.
- Changed `run_pipeline` return type from `Result<(Scene, Vec<String>)>` to
  `Result<(Scene, Vec<String>, Vec<GuiParam>)>`.
- `gui_params` are captured from `pre.gui_params.clone()` right after
  preprocessing and returned as the third tuple element.

### AppModel changes (`app.rs`)
New field: `gui_params: Vec<GuiParam>` — populated on every `Run`.

### AppWidgets changes (`app.rs`)
Two new widget references:
- `var_expander: gtk::Expander` — collapsible section, hidden when no params.
- `var_panel: gtk::Box` — vertical box of rows, one per `GuiParam`.

### UI layout
The left pane is now a `gtk::Box` (Vertical) containing:
1. `ScrolledWindow → TextView` (code editor, vexpand=true)
2. `gtk::Expander` (Variables panel, visible=false when empty)

The expander is positioned below the editor and expanded by default.
Its label updates to `"Variables (N)"` when params are present.

### Message: `AppMsg::GuiParamChanged { name, value }`
Fired by sliders and spinbuttons when the user drags/edits a value.  The
handler calls `Self::rewrite_define_value()` to update the `#define` line
in the source buffer **without** triggering an automatic re-run.  The status
bar shows: *"Variables updated — press ▶ Run (F5) to re-render."*

### Helper: `AppModel::rewrite_define_value(source, name, new_value) -> String`
Scans source lines, finds the `#define NAME ...` line, and replaces the value
while preserving any GUI annotation (`(float:1.0-15.0)` etc.) so the slider
range survives the next Run.

### Helper: `AppModel::rebuild_var_panel(expander, panel, gui_params, sender)`
Static method called from `update_with_view` after every `AppMsg::Run`.
Clears all existing children of `var_panel` and builds fresh rows:
- **float params** → `gtk::Scale` (horizontal, range lo–hi, digits=4)
- **int params** → `gtk::SpinButton` (integer)
Each widget is connected to emit `GuiParamChanged` on change.

## Result
Complete. The variables panel:
- Appears automatically when the script uses GUI `#define` directives.
- Shows float/int sliders for each param.
- Editing a slider rewrites the source buffer (syncs the text view).
- Pressing Run re-extracts params from the updated source and refreshes the panel.

## Status
Complete. 4 new tests in `app.rs::tests`.
