# T18 — Next Steps

## Recommended next task
T19 (Camera/settings import-export) and T20 (Template export UI flow) are both
complete in the same session.  See their respective task folders.

## Known deferred issues

1. **Auto-render on param change**: Currently the user must press Run after
   adjusting a slider.  A "Live preview" checkbox that auto-runs after each
   slider release would improve UX, but requires either debouncing or moving
   the pipeline to a background thread to avoid UI freezes.

2. **Slider value precision**: Float sliders use 4-digit precision.  Scripts
   with very small ranges (e.g. `(float:0.001-0.01)`) may need more digits.
   The `scale.set_digits()` call already supports this; the step size
   computation `(*max - *min) / 100.0` may need a floor when the range is tiny.

3. **Multi-line define support**: The rewrite only touches the first matching
   `#define NAME` line.  If a script accidentally has duplicate names the
   second definition is not updated (harmless — the preprocessor would use
   the last one).

4. **Slider desync after manual edit**: If the user manually edits a `#define`
   line in the text editor while the slider panel is visible, the slider value
   will not update until the next Run.  This is acceptable for now.

## Unanswered questions
- Should the slider update the text buffer character-by-character (current) or
  only on drag-end (`connect_change_value` instead of `connect_value_changed`)?
  The current approach gives live feedback but may emit many messages.
