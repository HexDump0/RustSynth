# Instructions for AI agents

## Primary objective

Rewrite Structure Synth in Rust while keeping the work:

- planned
- modular
- testable
- resumable across agent handoffs

## Architecture overview

RustSynth has a simple two-layer architecture:

1. **Rust core + CLI** — all compute-heavy work: EisenScript parsing, semantic analysis, evaluation/scene generation, and export (OBJ, template). Can be used standalone as a CLI tool (`rustsynth`).
2. **Web UI (Tauri + React + Three.js)** — a thin wrapper that calls the Rust pipeline over Tauri IPC and renders the scene with Three.js. The web layer does no computation — it only handles UI and 3D visualization.

The Rust core must never depend on any UI framework (Tauri, React, Three.js, GTK, etc.).

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
7. **Keep the core independent of UI.** Parser, semantics, evaluator, and exporters must not depend on Tauri, React, Three.js, or any frontend.
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

## Workspace layout

```text
RustSynth/
  Cargo.toml                    # workspace root
  crates/
    rustsynth_core/             # shared types, errors, math, RNG
    rustsynth_eisenscript/      # preprocessor, lexer, parser, AST
    rustsynth_semantics/        # name resolution, rule graph, validation
    rustsynth_eval/             # evaluator/builder, scene emission
    rustsynth_scene/            # scene representation + geometry adapters
    rustsynth_export_template/  # template exporter
    rustsynth_export_obj/       # OBJ exporter
    rustsynth_cli/              # standalone CLI binary
    rustsynth_app_tauri/        # Tauri app (React + Three.js frontend)
  tests/
    fixtures/
    golden/
```

### Crate responsibilities

- **`rustsynth_core`** — common errors, IDs, config types, color utilities, deterministic RNG, math adapters
- **`rustsynth_eisenscript`** — preprocessor, lexer, parser, AST, diagnostics
- **`rustsynth_semantics`** — rule graph, name resolution, primitive lookup, validation
- **`rustsynth_eval`** — execution engine that expands rules into scene objects
- **`rustsynth_scene`** — renderer-agnostic scene/object model + geometry adapter helpers (decompose transforms, matrix strings, etc.)
- **`rustsynth_export_template`** — template-based exporter
- **`rustsynth_export_obj`** — OBJ exporter
- **`rustsynth_cli`** — standalone CLI: `rustsynth build`, `rustsynth export-obj`, `rustsynth export-template`
- **`rustsynth_app_tauri`** — Tauri v2 desktop app with React frontend and Three.js viewport

## Recommended engineering principles

### 1. Preserve legacy semantics first
Legacy execution pipeline:

`source text -> preprocessor -> tokenizer -> parser -> ruleset/name resolution -> builder/evaluator -> scene`

Recreate this pipeline in Rust before adding UX improvements.

### 2. Keep determinism explicit
The legacy app uses seeds, random streams, recursion mode, and object limits. Treat reproducibility as a feature, not an implementation detail.

### 3. Prefer golden tests
Use legacy examples as fixtures. Capture parser outputs, scene snapshots, exported text, and rendered metadata where practical.

### 4. Document incomplete parity
If a feature is deferred, record:
- current gap
- affected examples/files
- suggested implementation path

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
