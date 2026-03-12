# Legacy Structure Synth codebase overview

This is a high-level map of the original C++/Qt codebase in `StructureSynth/Structure Synth Source Code/`.

## What the legacy app is

Structure Synth is a desktop tool for generating 3D structures from a small rule-based language called EisenScript. The app supports:

- editing and loading `.es` scripts
- preprocessing and parameterized script variants
- recursive rule evaluation into geometry
- interactive viewport rendering
- template-based export to third-party renderers
- OBJ export
- an integrated raytracer
- JavaScript-driven automation workflows

## High-level startup path

### Entry point
- `Main.cpp`
  - creates the Qt application
  - shows splash screen
  - constructs `StructureSynth::GUI::MainWindow`

### Main window
- `StructureSynth/GUI/MainWindow.*`
  - central app controller
  - editor tabs, menus, toolbars, settings, recent files
  - render/export/raytrace actions
  - drag and drop and camera helpers

The main window currently orchestrates nearly everything, so it is a major source of coupling.

## Core execution pipeline

The main legacy pipeline is:

1. user text is read from the editor
2. `Preprocessor` expands `#define` values and random substitutions
3. `Tokenizer` produces symbols
4. `EisenParser` parses into a `RuleSet`
5. `RuleSet::resolveNames()` resolves rule references and dynamic primitive classes
6. `Builder` evaluates the rules recursively
7. a `Renderer` backend receives primitive draw calls

This pipeline appears in the normal render path, template export path, and OBJ export path.

## Important modules

## GUI layer

### `StructureSynth/GUI/MainWindow.*`
Responsibilities:
- text editing tabs
- syntax highlighting
- file loading/saving
- seed handling
- render/export commands
- example/template path discovery
- raytrace and screenshot UI
- integration with parser/evaluator/exporters

### `StructureSynth/GUI/VariableEditor.*`
Builds slider/spinner controls from preprocessor metadata like:
- `#define angle 20 (float:0-90)`
- `#define iterations 8 (int:1-12)`

### `StructureSynth/GUI/TemplateExportDialog.*`
Dialog for template-based export configuration, dimensions, post-processing, autosave, and external command execution.

## Parser layer

### `StructureSynth/Parser/Preprocessor.*`
Implements:
- `#define` substitutions
- float/int GUI parameter extraction
- `random[a,b]` replacement using seeded randomness

Important note: it is mostly line-oriented string rewriting, not a structured macro system.

### `StructureSynth/Parser/Tokenizer.*`
Splits EisenScript into tokens/symbols.

### `StructureSynth/Parser/EisenParser.*`
Recursive descent parser for EisenScript.
Handles:
- rules
- actions
- transformation lists
- rule modifiers like `weight` and `maxdepth`
- set commands
- recursion mode flagging

## Model/runtime layer

### `StructureSynth/Model/RuleSet.*`
Owns all rules.
Adds built-ins automatically:
- box
- sphere
- cylinder
- mesh
- line
- dot
- grid
- template

Also resolves:
- ambiguous rule overloads
- primitive classes like `box::metal`
- triangle inline syntax like `triangle[x,y,z;...]`

### `StructureSynth/Model/Builder.*`
Core evaluator/executor.
Handles:
- breadth-first and depth-first traversal
- max depth and max objects
- min/max size pruning
- deterministic seeds
- sync-random option
- per-renderer commands, especially raytracer settings

### Other notable model types
- `Action.*` — rule actions and set actions
- `Transformation.*` — translation, rotation, scale, reflect, matrix, color, alpha, blend
- `State.*` — transform matrix, HSV color, alpha, recursion depth state, previous state, seed
- `PrimitiveRule.*` — built-in primitives and triangle special rule
- `AmbiguousRule.*` — weighted overload resolution behavior
- `ColorPool.*` — color sources
- `RandomStreams.*` — RNG plumbing

## Rendering/export layer

### `StructureSynth/Model/Rendering/Renderer.*`
Abstract renderer interface for primitive callbacks and camera/settings commands.

### `OpenGLRenderer.*`
Sends primitives to the legacy OpenGL engine/widget.

### `TemplateRenderer.*`
Expands geometry into text templates for external renderers.
This is a key legacy differentiator and should survive the rewrite.

### `ObjRenderer.*`
Exports geometry as OBJ with grouping options.

## JavaScript automation layer

### `StructureSynth/JavaScriptSupport/*`
Uses QtScript to automate:
- loading scripts
- applying substitutions
- building
- rendering to file
- template export
- process execution

This feature is useful but should be decoupled in the rewrite. JavaScript compatibility may be expensive to preserve exactly.

## Legacy engine dependencies

### `SyntopiaCore/*`
Shared library-style code containing:
- OpenGL engine/widget
- raytracer
- math types
- logging
- persistence/version helpers

### `ThirdPartyCode/MersenneTwister/*`
RNG dependency.

## Build/tooling state

The legacy project uses very old tooling:
- Qt4 / QtScript
- Visual Studio project files
- qmake-based Linux build script
- custom Windows packaging scripts

This is one of the strongest arguments for a clean rewrite instead of an incremental port.

## Architectural problems in the legacy code

1. **MainWindow is too central.** UI orchestration, parsing, rendering, exporting, and scripting are tightly mixed.
2. **Core semantics are spread across UI and runtime code.** Behavior is correct but not isolated.
3. **Rendering and evaluation are coupled by imperative callbacks.**
4. **Legacy UI stack is obsolete.** Qt4 and QtScript are end-of-life choices.
5. **Parity will be behavior-based, not structure-based.** The rewrite should copy outcomes, not class shapes.

## Rewrite guidance derived from the legacy design

- Keep parsing/evaluation headless and independent.
- Define a canonical scene representation between evaluation and outputs.
- Treat deterministic random behavior as part of the public semantics.
- Use fixtures from `Examples/` as regression inputs.
- Reimplement exporters after the core evaluator exists.
- Keep Bevy behind a viewport adapter, not inside the parser/runtime.

## Most important legacy files to consult

- `Main.cpp`
- `StructureSynth/GUI/MainWindow.cpp`
- `StructureSynth/GUI/MainWindow.h`
- `StructureSynth/GUI/VariableEditor.h`
- `StructureSynth/GUI/TemplateExportDialog.h`
- `StructureSynth/Parser/Preprocessor.cpp`
- `StructureSynth/Parser/EisenParser.cpp`
- `StructureSynth/Model/RuleSet.cpp`
- `StructureSynth/Model/Builder.cpp`
- `StructureSynth/Model/PrimitiveRule.h`
- `StructureSynth/Model/Rendering/Renderer.h`
- `StructureSynth/Model/Rendering/OpenGLRenderer.cpp`
- `StructureSynth/Model/Rendering/TemplateRenderer.h`
- `StructureSynth/Model/Rendering/ObjRenderer.h`
- `StructureSynth/JavaScriptSupport/JavaScriptBuilder.cpp`
- `SyntopiaCore/GLEngine/Raytracer/*`

## Recommended first implementation target

The safest first technical target is:

- a Rust headless parser + evaluator that can load example scripts and emit a canonical scene

Once that exists, Bevy, exporters, and UI work become much easier and less risky.

## Current rewrite direction

The current preferred rewrite direction is:

- headless core first
- `gtk4-rs` + `Relm4` for the desktop application shell
- a renderer boundary between the core scene output and any viewport implementation

The viewport is intentionally not fixed yet. Current candidates are:

- Bevy
- custom OpenGL
- later `wgpu`
