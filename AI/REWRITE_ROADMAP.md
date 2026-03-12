# RustSynth rewrite roadmap

## Rewrite goal

Build a complete Rust rewrite of Structure Synth with feature parity for v1, while improving maintainability and leaving room for better performance and a more modern UI.

Priority order:

1. working feature-complete rewrite
2. correctness and deterministic behavior
3. maintainable architecture
4. modern UI and performance wins where they are low-risk

## Recommended rewrite strategy

Do **not** start with the GUI.

Start by extracting a clean, testable Rust core that reproduces the legacy behavior:

1. language + preprocessing
2. semantic model + rule resolution
3. evaluator/builder
4. canonical scene/export representation
5. exporters
6. realtime viewport + desktop UI

This minimizes risk because the old app mixes Qt UI, parsing, evaluation, rendering, and export orchestration.

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
- box
- sphere
- cylinder
- mesh
- line
- dot
- grid
- template
- triangle special syntax

### Output backends
- realtime OpenGL viewport
- template exporter
- OBJ exporter
- integrated raytracer

### Application behavior
- multi-tab editor
- example loading
- recent files/settings
- screenshot/export flow
- camera settings import/export
- drag and drop
- JavaScript automation mode

## Recommended Rust architecture

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
    rustsynth_render_api/
    rustsynth_export_template/
    rustsynth_export_obj/
    rustsynth_app_gtk/
    rustsynth_viewport_bevy/
    rustsynth_viewport_gl/
    rustsynth_viewport_wgpu/   # optional later backend
    rustsynth_script/        # optional, if scripting is kept in v1
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
Renderer-agnostic scene/object stream. This is the boundary between language semantics and output backends.

### `rustsynth_render_api`
Renderer boundary traits, viewport-facing contracts, scene ingestion interfaces, camera/update hooks, and adapter types shared by multiple viewport backends.

### `rustsynth_export_template`
Template-based exporter compatible with the legacy template concept.

### `rustsynth_export_obj`
OBJ exporter with grouping behavior and sphere tessellation settings.

### `rustsynth_app_gtk`
GTK4 + Relm4 desktop application shell. Owns the editor, panes, dialogs, menus, settings, and orchestration of the headless core plus whichever viewport backend is selected.

### `rustsynth_viewport_bevy`
Optional Bevy adapter that renders the canonical scene interactively.

### `rustsynth_viewport_gl`
Optional custom OpenGL viewport backend, likely integrated through the GTK shell.

### `rustsynth_viewport_wgpu`
Optional later `wgpu` viewport backend if a custom renderer becomes preferable.

### `rustsynth_script`
Optional scripting compatibility layer. For v1, consider `rhai` first unless JavaScript compatibility becomes mandatory.

## App shell assessment

## Short answer

`gtk4-rs` + `Relm4` is the preferred choice for the **desktop application shell**.

## Why GTK4 + Relm4 fits

- this project is editor-heavy and tool-like, not game-like
- menus, panes, dialogs, file workflows, and settings are first-class requirements
- Relm4 gives a component/message architecture that fits long-lived desktop tooling
- GTK4 already covers the kind of application shell Structure Synth historically had

## What this means

- GTK4 + Relm4 should own the editor shell
- the headless core remains fully independent
- the viewport remains replaceable behind a renderer boundary

## Viewport assessment

## Short answer

Do **not** lock the rewrite to a single viewport implementation yet.

## Why Bevy still fits as a backend

- strong realtime 3D support
- ECS can manage scene objects cleanly
- modern rendering stack
- cross-platform potential
- good path to performance improvements on modern hardware

## Why GTK-only viewport rendering should not be assumed yet

- GTK4 is a strong widget toolkit, but not a complete 3D engine architecture
- `GLArea` is usable for custom rendering, but it pushes more renderer ownership into the app
- viewport complexity should be evaluated separately from shell complexity

## Recommendation

Use the stack like this:

- evaluator produces a renderer-agnostic `Scene`
- `rustsynth_render_api` defines how viewport backends consume that scene
- GTK4 + Relm4 owns the desktop shell
- viewport backend can be:
  - Bevy
  - custom OpenGL
  - later `wgpu`

Do not force the entire application into Bevy. Do not force the viewport into GTK-only rendering before the boundary is defined.

## Milestones

### Milestone 0 — planning and baseline capture
- audit legacy codebase
- define task system and AI handoff process
- decide initial workspace architecture
- collect examples and golden fixtures

### Milestone 1 — headless language core
- implement preprocessor
- implement lexer/tokenizer
- implement parser + AST
- implement semantic resolution
- build parser diagnostics and fixture tests

### Milestone 2 — evaluator parity
- implement transformations and state handling
- implement recursion modes
- implement weighted/ambiguous rule selection
- implement deterministic RNG behavior
- emit canonical scene objects

### Milestone 3 — export parity
- template export parity
- OBJ export parity
- camera/settings serialization parity
- baseline regression tests

### Milestone 4 — modern app shell
- bootstrap GTK4 + Relm4 application shell
- file/project handling
- editor and examples browser
- variable controls
- export dialogs

### Milestone 5 — viewport and rendering
- define renderer boundary
- evaluate and prototype viewport backends
- Bevy scene bridge
- custom OpenGL bridge if warranted
- camera controls
- selection/inspection if desired
- screenshot support

### Milestone 6 — advanced features
- scripting mode replacement
- integrated raytracer or equivalent decision
- performance profiling and optimization
- release packaging and docs

## v1 acceptance criteria

The rewrite can be called v1-ready when it has:

- working EisenScript parsing and evaluation
- support for the major legacy primitives
- deterministic seed behavior
- template export
- OBJ export
- a usable desktop editor and viewport
- enough examples passing to demonstrate practical parity

The following are desirable but not strictly required if they threaten delivery:

- legacy JavaScript compatibility byte-for-byte
- identical internal raytracer implementation
- every minor legacy UI behavior
- substantial visual redesign beyond a solid modern baseline

## Suggested finish order

1. headless core parity
2. exporters
3. renderer boundary
4. GTK4 + Relm4 app shell
5. viewport backend selection/prototype
6. scripting compatibility
7. raytracer/performance polish
