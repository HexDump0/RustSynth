# T20 — Changes

## Files changed

| File | Reason |
|---|---|
| `crates/rustsynth_app_gtk/src/app.rs` | Add `template_path` to model; `SetTemplateFile`, `TemplateFileSet`, `ClearTemplateFile` variants + handlers; `export_template` uses dynamic template XML; "Set Template File…"/"Clear Template File" menu items |

## Tests run

```
cargo test --workspace
# 86 total; 0 failed (no new tests for this task — UI plumbing only)
```

The template export core logic is already covered by T11's 8 tests in
`rustsynth_export_template`.
