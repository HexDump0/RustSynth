# T20 — Next Steps

## Recommended next tasks (all READY)

| Task | Description |
|---|---|
| **T21** | Integrate OBJ export UI flow — the OBJ export already works end-to-end; add any UX polish (e.g. progress indicator for large scenes) |
| **T23** | Port examples and gallery workflow — add an example browser/import path |
| **T24** | End-to-end parity regression suite — golden tests across the full pipeline |

## Known deferred issues

1. **Template persistence across sessions**: The chosen `template_path` is
   not saved to disk.  A future enhancement would persist it in a config file
   (`~/.config/rustsynth/settings.json`) or as a GSettings key.

2. **Template preview**: There is no way to preview what a template produces
   before exporting.  A "Preview" pane showing the first few lines of output
   would help users verify their template is correct.

3. **Template gallery**: The legacy StructureSynth shipped with several
   renderer templates (Sunflow, POV-Ray, etc.).  These could be bundled into
   the binary as built-in choices alongside the current plain-text fallback.
   See `StructureSynth/Structure Synth Source Code/` for reference `.xml` files.

## Unanswered questions
- Should the default extension for the export dialog be inferred from the
  template's `defaultExtension` attribute?  Currently always `*.txt`.
