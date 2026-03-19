# RustSynth rewrite roadmap

## Rewrite goal

Build a complete Rust rewrite of Structure Synth with feature parity for v1, while improving maintainability and leaving room for better performance and a more modern UI.

Priority order:

1. working feature-complete rewrite
2. correctness and deterministic behavior
3. maintainable architecture
4. modern UI and performance wins where they are low-risk

## Architecture

Two layers:

1. **Rust core + CLI** — all compute: parsing, evaluation, scene generation, export. Runs standalone via `rustsynth` CLI.
2. **Web UI (Tauri + React + Three.js)** — thin wrapper calling Rust over IPC, rendering with Three.js.

The Rust core has zero UI dependencies.

## Workspace layout

```text
RustSynth/
  Cargo.toml
  crates/
    rustsynth_core/
    rustsynth_eisenscript/
    rustsynth_semantics/
    rustsynth_eval/
    rustsynth_scene/
    rustsynth_export_template/
    rustsynth_export_obj/
    rustsynth_cli/
    rustsynth_app_tauri/
  tests/
    fixtures/
    golden/
```

## Crate responsibilities

### `rustsynth_core`
Common errors, IDs, configuration types, color utilities, deterministic RNG wrapper, and shared math adapters.

### `rustsynth_eisenscript`
Preprocessor, lexer, parser, AST, diagnostics, and parser tests.

### `rustsynth_semantics`
Rule graph, name resolution, primitive lookup, validation, and normalization.

### `rustsynth_eval`
Execution engine that expands rules into a canonical scene representation.

### `rustsynth_scene`
Renderer-agnostic scene/object stream and geometry adapter helpers (transform decomposition, matrix string formatting, sphere/cylinder extraction).

### `rustsynth_export_template`
Template-based exporter compatible with the legacy template concept.

### `rustsynth_export_obj`
OBJ exporter with grouping behavior and sphere tessellation settings.

### `rustsynth_cli`
Standalone CLI binary. Subcommands: `build` (JSON scene output), `export-obj`, `export-template`.

### `rustsynth_app_tauri`
Tauri v2 desktop app. React frontend with Three.js viewport. Calls the Rust pipeline over Tauri IPC. Does no computation — pure UI and rendering.

## Legacy feature groups to preserve

### Language and execution
- EisenScript tokenization/parsing
- `#define` preprocessing and random substitution
- GUI-driven variable definitions from preprocessor metadata
- rule weights, max depth, retirement rules
- breadth-first and depth-first recursion
- deterministic seed behavior and sync-random mode
- per-rule transformations, colors, alpha, reflections, matrices

### Built-in primitives
- box, sphere, cylinder, mesh, line, dot, grid, template
- triangle special syntax

### Output backends
- JSON scene output (for web viewport or external tools)
- template exporter
- OBJ exporter
- integrated raytracer (deferred)

### Application behavior
- editor with EisenScript support
- example loading
- export workflows
- camera controls (Three.js OrbitControls)
- variable panel for `#define` parameters

## Milestones

### Milestone 0 — planning and baseline capture ✅
- audit legacy codebase
- define task system and AI handoff process
- collect examples and golden fixtures

### Milestone 1 — headless language core ✅
- preprocessor, lexer, parser, AST
- semantic resolution
- parser diagnostics and fixture tests

### Milestone 2 — evaluator parity ✅
- transformations, state handling, recursion modes
- weighted/ambiguous rule selection
- deterministic RNG, canonical scene objects

### Milestone 3 — export parity ✅
- template export, OBJ export
- camera/settings serialization
- baseline regression tests

### Milestone 4 — app shell ✅
- Tauri + React app shell
- Three.js viewport with instanced rendering
- file handling, editor, variable panel, export dialogs

### Milestone 5 — CLI and polish
- standalone CLI binary
- syntax highlighting
- example browser
- end-to-end parity regression suite

### Milestone 6 — advanced features
- scripting mode replacement
- raytracer decision
- performance optimization
- release packaging and docs

## v1 acceptance criteria

The rewrite can be called v1-ready when it has:

- working EisenScript parsing and evaluation
- support for the major legacy primitives
- deterministic seed behavior
- template export and OBJ export
- a usable editor and 3D viewport
- standalone CLI for headless use
- enough examples passing to demonstrate practical parity
