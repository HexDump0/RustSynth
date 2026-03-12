# Instructions for AI agents

## Primary objective

Rewrite Structure Synth in Rust while keeping the work:

- planned
- modular
- testable
- resumable across agent handoffs

## Hard rules

1. **Read before acting.** Start with:
   - `AI/MASTER_TODO.md`
   - `AI/REWRITE_ROADMAP.md`
   - `AI/LEGACY_CODEBASE_OVERVIEW.md`
   - the latest relevant `AI/<task_slug>/` folder
2. **Pick one task at a time.** Do not mix unrelated roadmap items unless the master todo explicitly says they are coupled.
3. **Create a task folder first.** Every task must have a folder `AI/<task_slug>/`.
4. **Update the master todo.** Mark task state changes, added subtasks, and blockers.
5. **Leave handoff notes.** Assume the next agent has no memory except what is in this repository.
6. **Use the legacy code as the behavior reference.** Do not invent semantics when the old code answers the question.
7. **Keep the core independent of UI and renderer choices.** Parser, semantics, and evaluator must not depend on GTK, Relm4, wgpu, or any frontend.
8. **Add tests with behavior changes.** Parser/evaluator/export work should land with automated tests whenever practical.
9. **Prefer parity before redesign.** Improvements are welcome after a legacy-equivalent baseline exists.
10. **Record decisions.** If you choose a design path, write it down in the task folder.

## Task selection algorithm

Choose the next task using this order:

1. Highest-priority task marked `READY`
2. Lowest task ID among ready tasks
3. If multiple are ready, prefer:
   - tasks that unblock many later tasks
   - tasks that improve test coverage
   - tasks that reduce ambiguity in legacy behavior

Do not start a task marked `BLOCKED` until its dependencies are resolved.

## Required outputs per task

Inside `AI/<task_slug>/`, create at least:

### `SUMMARY.md`
Include:
- task goal
- short approach
- result
- whether the task is complete, partial, or blocked

### `CHANGES.md`
Include:
- file list
- short reason for each change
- tests run

### `NEXT.md`
Include:
- recommended next task
- known blockers
- unanswered questions

## Recommended engineering principles

### 1. Preserve legacy semantics first
Legacy execution pipeline:

`source text -> preprocessor -> tokenizer -> parser -> ruleset/name resolution -> builder/evaluator -> renderer/export backend`

Recreate this pipeline in Rust before adding UX improvements.

### 2. Separate crates by responsibility
Recommended split:

- `rustsynth_core` — shared types, errors, math helpers, IDs
- `rustsynth_eisenscript` — lexer, preprocessor, parser, AST, diagnostics
- `rustsynth_semantics` — name resolution, rule graph, validation
- `rustsynth_eval` — execution/builder logic and deterministic expansion
- `rustsynth_scene` — canonical render/export scene representation
- `rustsynth_render_api` — renderer boundary traits and viewport-facing scene contracts
- `rustsynth_export_template` — template exporter
- `rustsynth_export_obj` — OBJ exporter
- `rustsynth_viewport_wgpu` — **chosen** viewport backend: wgpu via `GtkGLArea` EGL surface
- `rustsynth_viewport_bevy` — deferred; Bevy viewport option (not the chosen path)
- `rustsynth_viewport_gl` — deferred; custom OpenGL option (not the chosen path)
- `rustsynth_app_gtk` — GTK4 + Relm4 desktop UI shell
- optional: `rustsynth_script` — scripting compatibility layer

### 3. Keep determinism explicit
The legacy app uses seeds, random streams, recursion mode, and object limits. Treat reproducibility as a feature, not an implementation detail.

### 4. Prefer golden tests
Use legacy examples as fixtures. Capture parser outputs, scene snapshots, exported text, and rendered metadata where practical.

### 5. Document incomplete parity
If a feature is deferred, record:
- current gap
- affected examples/files
- suggested implementation path

## GTK4 + Relm4 guidance

The current preferred app-shell stack is:

- `gtk4-rs` for native desktop widgets and windowing
- `Relm4` for component/message architecture

This stack is preferred for:

- editor-heavy desktop UX
- menus, shortcuts, dialogs, panes, and settings
- export workflows and file handling
- maintainable component structure

## Viewport guidance

The viewport backend decision is made: **`wgpu` via `GtkGLArea` EGL surface**.

Current preferred rule:

- the app shell is GTK4 + Relm4
- the core is headless
- the viewport sits behind a renderer boundary (`rustsynth_render_api`)
- the viewport implementation is `rustsynth_viewport_wgpu`

Chosen approach:

- `GtkGLArea` owns the surface and EGL context — GTK drives the loop
- `wgpu` targets that context via `wgpu::Backends::GL`
- GTK signals (`realize`, `unrealize`, `resize`, `render`) drive the wgpu lifecycle
- camera input is handled by GTK event controllers, no inter-thread channels needed
- WGSL shaders handle primitive rendering (box, sphere, cylinder, etc.)

Bevy and custom OpenGL are deferred indefinitely. Do not implement T15 or T16.

## When context is running low

Before stopping:

1. update `AI/MASTER_TODO.md`
2. write `AI/<task_slug>/NEXT.md`
3. summarize any important legacy behavior you discovered
4. note exact files in the legacy code that still need inspection

## Definition of done for a task

A task is done when:

- its deliverables exist
- code compiles or tests pass when applicable
- follow-up work is documented
- the master todo is updated
- the task folder contains enough context for a fresh agent to continue
