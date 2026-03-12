# T05 — Changes

## Files changed
- `RustSynth/crates/rustsynth_eisenscript/src/lexer.rs`
  - Replaced the stub with a legacy-shaped token stream implementation.
  - Added token classification, comment stripping, operator normalization, fraction parsing, bracket capture, diagnostics, and tests.
- `AI/MASTER_TODO.md`
  - Marked T05 done and unblocked T06.
- `AI/t02_feature_matrix/FEATURE_MATRIX.md`
  - Marked the implemented tokenizer features as complete.

## Tests run
- `cargo test -p rustsynth_eisenscript`
- `cargo test --workspace`
