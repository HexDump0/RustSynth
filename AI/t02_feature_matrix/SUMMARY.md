# T02 — Summary

## Task goal
Build a comprehensive feature inventory of the legacy Structure Synth codebase and map each feature to its legacy source module, its target Rust crate, and the task responsible for implementing it.

## Approach
- Read all key legacy source files: `Preprocessor.{h,cpp}`, `Tokenizer.{h,cpp}`, `EisenParser.{h,cpp}`, `Transformation.h`, `Builder.{h,cpp}`, `State.h`, `RuleSet.cpp`, `Renderer.h`, plus several example `.es` scripts.
- Organized features into 14 sections matching the major legacy subsystems.
- Flagged every feature with one of: ✅ done, 🏗 skeleton exists, ❌ not started, ⏭ deferred.
- Noted two exceptions that require special attention: `1/3`-style arithmetic in number tokens (seen in Menger.es) and the `#include` preprocessor directive (referenced in the EBNF comment but not in `Preprocessor.cpp`).

## Result
`FEATURE_MATRIX.md` — 14 sections, ~80 rows, every feature traceable to legacy code and a target task.

## Status
**Complete.**
