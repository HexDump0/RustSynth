# Summary

## Task

Investigate the legacy Structure Synth codebase and create the initial planning/context system for the Rust rewrite.

## What was inspected

The following legacy areas were reviewed:

- app entry point and startup flow
- main window and render/export orchestration
- preprocessor and parser
- rule set and evaluator/builder
- renderer abstraction and export backends
- JavaScript automation support
- build notes, changelog, and roadmap notes

## Key findings

1. The legacy app has a clear semantic pipeline, but the implementation is spread across Qt UI code.
2. The most important reusable mental model is:
   - preprocess
   - tokenize
   - parse
   - resolve
   - evaluate
   - render/export
3. `MainWindow` is the orchestration bottleneck and should not be mirrored in Rust.
4. Template export and deterministic generation are core product features, not side features.
5. The chosen direction is GTK4 + Relm4 for the desktop shell, while keeping the language/runtime independent from any frontend.
6. The viewport should remain pluggable behind a renderer boundary, with Bevy, custom OpenGL, and later `wgpu` all remaining possible.
7. A parity-first headless core is the lowest-risk start.

## Outputs created

- `AI/README.md`
- `AI/AGENT_INSTRUCTIONS.md`
- `AI/MASTER_TODO.md`
- `AI/REWRITE_ROADMAP.md`
- `AI/LEGACY_CODEBASE_OVERVIEW.md`
- this task folder

## Outcome

Planning and AI workflow setup is complete.

The next best tasks are:

- `T01` Bootstrap Rust workspace
- `T02` Build legacy feature matrix
- `T03` Collect golden fixtures
