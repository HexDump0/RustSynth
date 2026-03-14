# RustSynth AI coordination

This directory is the shared memory layer for AI agents working on the Rust rewrite of Structure Synth.

## Goals

- Keep the rewrite planned, incremental, and recoverable across context resets.
- Preserve the important behavior of the legacy app before making improvements.
- Make every task leave behind enough context for the next agent to continue quickly.

## Required workflow for every AI task

1. Read these files first:
   - `AI/AGENT_INSTRUCTIONS.md`
   - `AI/MASTER_TODO.md`
   - `AI/REWRITE_ROADMAP.md`
   - `AI/LEGACY_CODEBASE_OVERVIEW.md`
   - the most recent relevant folder under `AI/<task_name>/`
2. Pick the next **ready** task from `AI/MASTER_TODO.md`.
3. Create a new folder: `AI/<task_slug>/`
4. During the task, keep short notes in that folder.
5. Before finishing, update `AI/MASTER_TODO.md`.
6. Leave a short handoff for the next agent.

## Folder convention

Each task folder should use a short slug, for example:

- `AI/bootstrap_rust_workspace/`
- `AI/legacy_feature_matrix/`
- `AI/eisenscript_parser_v1/`

Minimum files expected inside each task folder:

- `SUMMARY.md` — what was done, why, and outcome
- `CHANGES.md` — files created/edited and short reasoning
- `NEXT.md` — recommended next actions, blockers, open questions

Optional files:

- `TESTS.md`
- `DECISIONS.md`
- `RISKS.md`
- `NOTES.md`

## Project strategy

The rewrite should be **parity-first, modular, and test-heavy**:

- first reproduce the legacy behavior in a headless Rust core
- then add a Tauri v2 + React desktop shell around that core
- keep viewport rendering behind a renderer boundary
- only then spend effort on visual and performance improvements

The legacy source remains the reference implementation and should be consulted often.

## Current architecture direction

The current preferred direction is:

- headless core first
- `Tauri v2` for the cross-platform desktop application shell
- `React` + `TypeScript` for the frontend UI
- `Three.js` (via `@react-three/fiber`) for the viewport rendering
- viewport renders the canonical `Scene` JSON produced by the Rust core
- GTK4+Relm4 and wgpu are deprecated (remain in workspace but superseded by the Tauri app)

Viewport decision:

- **chosen: `Three.js`** — renders Scene JSON in a React component via @react-three/fiber; supports all primitive types
- deprecated: `wgpu` via `GtkGLArea` — complex EGL plumbing, limited cross-platform reach
- deferred: Bevy — too much engine overhead, fights GTK for window ownership
- deferred: custom OpenGL — wgpu covers this use case with a better API
