# T04 — Changes

## Files changed
- `RustSynth/crates/rustsynth_eisenscript/src/preprocessor.rs`
  - Replaced the stub with a working preprocessor.
  - Added GUI parameter extraction, recursive substitution guard, random replacement, and unit tests.
- `RustSynth/crates/rustsynth_eisenscript/src/diagnostics.rs`
  - Added `PartialEq`/`Eq` derives to support diagnostic assertions in tests.
- `AI/MASTER_TODO.md`
  - Marked T04 done and unblocked T06.
- `AI/t02_feature_matrix/FEATURE_MATRIX.md`
  - Marked the implemented preprocessor features as complete.

## Tests run
- `cargo test -p rustsynth_eisenscript`
- `cargo test --workspace`
