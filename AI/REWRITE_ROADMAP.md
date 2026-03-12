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
- realtime wgpu viewport (via `GtkGLArea` EGL surface)
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
    rustsynth_viewport_wgpu/   # chosen viewport backend
    rustsynth_viewport_bevy/   # deferred
    rustsynth_viewport_gl/     # deferred
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

### `rustsynth_viewport_wgpu`
**Chosen viewport backend.** Renders the canonical scene using wgpu targeting a `GtkGLArea` EGL context. Implements `ViewportBackend` with WGSL shaders, geometry upload, and an arcball camera uniform. GTK4 owns the surface and drives the render loop via signals.

### `rustsynth_viewport_bevy`
Deferred. Bevy adapter skeleton kept for reference. Not the chosen implementation path.

### `rustsynth_viewport_gl`
Deferred. Custom OpenGL backend skeleton kept for reference. Not the chosen implementation path.

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

## Decision

**`wgpu` via `GtkGLArea` EGL surface is the chosen viewport backend.**

## Why wgpu fits best

- GTK4's `GtkGLArea` owns the EGL context — wgpu targets it directly via `wgpu::Backends::GL`
- no separate thread, no inter-process channels, no window handle hacks
- GTK signals (`realize`, `unrealize`, `resize`, `render`) map directly onto the `ViewportBackend` trait lifecycle
- camera input (orbit, zoom, pan) is handled by GTK event controllers in the normal GTK event loop
- the scene is a flat list of `SceneObject` — no ECS needed, each object is one draw call
- WGSL shaders give full control over primitive rendering without engine overhead
- smaller binary, faster compile times, and less dependency surface than Bevy

## Why Bevy was not chosen

- Bevy requires disabling its own window and event loop, fighting WinitPlugin ownership
- requires a dedicated thread and `mpsc` channels for every viewport interaction
- ECS is architectural overhead for a static procedural scene model
- significant compile-time and binary-size cost for features that are not needed here

## Architecture

- evaluator produces a renderer-agnostic `Scene` (`rustsynth_scene`)
- `rustsynth_render_api` defines the `ViewportBackend` trait all backends implement
- GTK4 + Relm4 owns the desktop shell
- `rustsynth_viewport_wgpu` implements `ViewportBackend` and is wired to a `GtkGLArea` widget in the app shell

Bevy (T15) and custom OpenGL (T16) are deferred indefinitely.

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
- define renderer boundary (T10A)
- implement wgpu viewport backend in `rustsynth_viewport_wgpu` (T17)
- wire `GtkGLArea` + wgpu EGL surface in the app shell (T14)
- WGSL shaders for box, sphere, cylinder primitives
- arcball camera controls
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
