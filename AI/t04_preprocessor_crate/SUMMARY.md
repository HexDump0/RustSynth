# T04 — Implement preprocessor crate

## Task goal
Implement the EisenScript preprocessor in Rust with parity-oriented support for `#define`, GUI parameter metadata, recursion guards, and `random[a,b]` substitution.

## Approach
- Read the legacy `Preprocessor.cpp` behavior and the fixture manifest.
- Implemented a line-oriented preprocessor in `rustsynth_eisenscript`.
- Preserved legacy-style recursive substitution behavior with a 100-round guard.
- Added GUI parameter extraction for `float` and `int` metadata and default-value substitution into the output.
- Added deterministic `random[a,b]` expansion using the shared RNG wrapper.
- Added fixture-backed unit tests for plain defines, GUI defines, recursion warnings, and seeded random replacement.

## Result
- `preprocess()` now returns rewritten source, GUI metadata, and diagnostics.
- `Tutorials/Preprocessor.es` and `Tutorials/PreprocessorGUI.es` are covered by tests.
- The crate remains headless and ready for parser integration.

## Status
Complete.
