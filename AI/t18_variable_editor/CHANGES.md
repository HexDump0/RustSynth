# T18 — Changes

## Files changed

| File | Reason |
|---|---|
| `crates/rustsynth_app_gtk/src/pipeline.rs` | Add `pub use GuiParam`, change return to 3-tuple including `Vec<GuiParam>` |
| `crates/rustsynth_app_gtk/src/app.rs` | Add `gui_params` field to model, `var_panel`/`var_expander` to widgets, `AppMsg::GuiParamChanged`, var panel rebuild, `rewrite_define_value` helper, 4 unit tests |
| `crates/rustsynth_app_gtk/src/main.rs` | No direct T18 change (but `camera_io` mod added for T19 in same pass) |

## Tests run

```
cargo test --workspace
# 86 total; 0 failed (4 new in app.rs::tests)
```

### New tests
- `rewrite_plain_define` — plain `#define` value is rewritten
- `rewrite_gui_define_preserves_metadata` — `(float:1.0-15.0)` annotation preserved
- `rewrite_define_noop_when_name_absent` — source unchanged when name not found
- `rewrite_int_define_preserves_metadata` — `(int:1-20)` annotation preserved
