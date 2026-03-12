# T02 — Legacy Feature Matrix and Parity Checklist

Every feature is mapped to its legacy source module(s), its target Rust crate, the task that implements it, and a parity status.

## Status legend

| Symbol | Meaning |
|---|---|
| ✅ | Implemented and tested |
| 🏗 | Skeleton exists, implementation pending |
| ❌ | Not yet started |
| ⏭ | Intentionally deferred beyond v1 |

---

## 1. Preprocessor

**Legacy source:** `StructureSynth/Parser/Preprocessor.{h,cpp}`  
**Rust crate:** `rustsynth_eisenscript` (`src/preprocessor.rs`)  
**Implements:** T04

| Feature | Legacy detail | Status |
|---|---|---|
| `#define name value` substitution | Line-oriented text replace, max 100 recursive substitutions | ✅ |
| `#define name val (float:lo-hi)` | Extracts `FloatParameter`; substitutes default value in text | ✅ |
| `#define name val (int:lo-hi)` | Extracts `IntParameter`; substitutes default value in text | ✅ |
| Recursion guard on `#define` | Warns and skips if the replacement contains the token itself | ✅ |
| `random[lo,hi]` substitution | Seeded RNG replaces `random[a,b]` with a float in [a,b] | ✅ |
| `#include` handling | Legacy comment in parser hints at `#include` support; not in Preprocessor.cpp | ❌ |
| Preprocessor-driven GUI metadata | `FloatParameter` / `IntParameter` objects exposed to the UI layer | ✅ |

---

## 2. Tokenizer / Lexer

**Legacy source:** `StructureSynth/Parser/Tokenizer.{h,cpp}`  
**Rust crate:** `rustsynth_eisenscript` (`src/lexer.rs`)  
**Implements:** T05

| Feature | Legacy detail | Status |
|---|---|---|
| Symbol types | `Undefined`, `LeftBracket`, `RightBracket`, `MoreThan`, `End`, `Number`, `Multiply`, `UserString`, `Rule`, `Set`, `Operator` | ✅ |
| Operators (keywords) | `c`, `reflect`, `color`, `blend`, `a`, `alpha`, `matrix`, `h`, `hue`, `sat`, `b`, `brightness`, `v`, `x`, `y`, `z`, `rx`, `ry`, `rz`, `s`, `fx`, `fy`, `fz`, `maxdepth`, `weight`, `md`, `w` | ✅ |
| `//` line comments | Stripped during tokenization | ✅ |
| `/* */` block comments | Stripped during tokenization | ✅ |
| `#` preprocessor lines | Treated as inline comment (not parsed) in tokenizer | ✅ |
| `[...]` bracket token | Kept as single token for vector/matrix params | ✅ |
| Integer vs float number | Tracks `isInteger` flag on number tokens | ✅ |
| Case normalisation | `UserString` tokens are lowercased | ✅ |
| Arithmetic expression in params | `1/3` style division in `s` args (e.g. Menger.es) | ✅ |

---

## 3. Parser / AST

**Legacy source:** `StructureSynth/Parser/EisenParser.{h,cpp}`  
**Rust crate:** `rustsynth_eisenscript` (`src/parser.rs`, `src/ast.rs`)  
**Implements:** T06

| Feature | Legacy detail | Status |
|---|---|---|
| `rule <name> { ... }` definition | `CustomRule` | 🏗 |
| Rule weight modifier (`w` / `weight`) | `customRule->setWeight(param)` | 🏗 |
| Rule `maxdepth` modifier | `customRule->setMaxDepth(param)` | 🏗 |
| Retirement rule (`maxdepth N > rulename`) | `customRule->setRetirementRule(ruleName)` | 🏗 |
| `{ transform... } rulename` action | `Action(Transformation, ruleName)` | 🏗 |
| Bare `rulename` action | `Action(ruleName)` | 🏗 |
| Loop action (`N * { ... } rulename`) | `TransformationLoop(count, transform)` | 🏗 |
| Chained loops (`N * {...} M * {...} rulename`) | Multiple `TransformationLoop` on one action | 🏗 |
| `set key value` inside rule body | `setAction()` | 🏗 |
| Top-level `set key value` | Executed on the `TopLevelRule` | 🏗 |
| Top-level bare invocations | Rule calls at script scope | 🏗 |
| `set recursion depth` flag | Sets `recurseDepthFirst = true` on the `RuleSet` | 🏗 |
| Arithmetic division `/` in number literals | `s 1/3` syntax seen in Menger.es | ❌ |

---

## 4. Transformations

**Legacy source:** `StructureSynth/Model/Transformation.{h,cpp}`  
**Rust crate:** `rustsynth_eval` (`src/transform.rs`)  
**Implements:** T08 / T09

| Transform op | Description | Status |
|---|---|---|
| `x N` | Translate X | 🏗 |
| `y N` | Translate Y | 🏗 |
| `z N` | Translate Z | 🏗 |
| `rx N` | Rotate around X (degrees) | 🏗 |
| `ry N` | Rotate around Y (degrees) | 🏗 |
| `rz N` | Rotate around Z (degrees) | 🏗 |
| `s N` | Uniform scale | 🏗 |
| `s Nx Ny Nz` | Non-uniform scale | 🏗 |
| `fx` | Flip X (`scale -1 1 1`) | 🏗 |
| `fy` | Flip Y (`scale 1 -1 1`) | 🏗 |
| `fz` | Flip Z (`scale 1 1 -1`) | 🏗 |
| `reflect Nx Ny Nz` | Plane reflection by normal | 🏗 |
| `matrix a b c d e f g h i` | Free 3×3 affine matrix (padded to 4×4) | 🏗 |
| `hue N` / `h N` | Shift HSV hue by N degrees | 🏗 |
| `sat N` | Scale HSV saturation | 🏗 |
| `brightness N` / `b N` | Scale HSV value/brightness | 🏗 |
| `alpha N` / `a N` | Scale alpha | 🏗 |
| `color <name>` | Set absolute color | 🏗 |
| `color random` | Sample from active color pool | 🏗 |
| `blend <color> <strength>` | Blend current color toward target in HSV space | 🏗 |

---

## 5. Execution / Builder

**Legacy source:** `StructureSynth/Model/Builder.{h,cpp}`, `State.*`, `ExecutionStack.*`, `RandomStreams.*`  
**Rust crate:** `rustsynth_eval` (`src/builder.rs`, `src/state.rs`, `src/recursion.rs`)  
**Implements:** T08

| Feature | Legacy detail | Status |
|---|---|---|
| Breadth-first rule expansion (default) | `recurseBreadthFirst()` | 🏗 |
| Depth-first rule expansion | `recurseDepthFirst()` triggered by `set recursion depth` | 🏗 |
| Max depth (`set maxdepth N`) | `maxGenerations` limit | 🏗 |
| Max objects (`set maxobjects N`) | `maxObjects` limit | 🏗 |
| Max size pruning (`set maxsize F`) | Prune branches whose transform vector exceeds F | 🏗 |
| Min size pruning (`set minsize F`) | Prune branches whose transform vector is below F | 🏗 |
| Per-rule `maxdepth` | `State.maxDepths` map tracks depth per rule | 🏗 |
| Retirement rule fallback | When rule hits its maxdepth, exec switches to retirement rule | 🏗 |
| Weighted ambiguous rule selection | `AmbiguousRule` weighted random choice from overloads | 🏗 |
| Seed propagation (`set seed N`) | `RandomStreams::SetSeed(N)` | 🏗 |
| Seed `initial` | Freezes a seed for `set seed initial` re-use | 🏗 |
| `syncRandom` mode | If true, seed is re-seeded per object from `state.seed` | 🏗 |
| `set rng old/new` | Legacy toggle between old/new RNG implementation | ⏭ |
| Color pool (`set colorpool <type>`) | `RandomHue`, `RandomRGB`, `GreyScale`, `image:<file>`, `list:<csv>` | 🏗 |
| `PreviousState` / state stack | Previous state accessible for some transform semantics | 🏗 |

---

## 6. Built-in Primitives

**Legacy source:** `StructureSynth/Model/PrimitiveRule.*`, `RuleSet.cpp`  
**Rust crate:** `rustsynth_semantics` (`src/primitive.rs`), `rustsynth_scene` (`src/primitive.rs`)  
**Implements:** T07

| Primitive | Notes | Status |
|---|---|---|
| `box` | Default built-in | 🏗 |
| `sphere` | Default built-in | 🏗 |
| `cylinder` | Default built-in | 🏗 |
| `mesh` | Start/end cylinder-like primitive | 🏗 |
| `line` | Thin line | 🏗 |
| `dot` | Point primitive | 🏗 |
| `grid` | Grid plane | 🏗 |
| `template` | Template placeholder primitive | 🏗 |
| `triangle[...]` | Special inline triangle syntax | ❌ |
| `primitive::tag` syntax | E.g. `box::metal`, `sphere::shiny` — creates named `PrimitiveClass` | 🏗 |

---

## 7. Name Resolution

**Legacy source:** `StructureSynth/Model/RuleSet.*`, `AmbiguousRule.*`, `CustomRule.*`  
**Rust crate:** `rustsynth_semantics` (`src/resolution.rs`, `src/rule_graph.rs`)  
**Implements:** T07

| Feature | Legacy detail | Status |
|---|---|---|
| Rule lookup by name | `RuleSet` hosts all rules, resolved by name | 🏗 |
| Ambiguous rule merging | Multiple `CustomRule`s with the same name merge into `AmbiguousRule` | 🏗 |
| Primitive name collision check | Adding a rule with the same name as a primitive is an error | 🏗 |
| `PrimitiveClass` extraction from `rule::tag` | `existsPrimitiveClass()` / `getPrimitiveClass()` | 🏗 |

---

## 8. `set` Commands

**Legacy source:** `Builder::setCommand()`, `Renderer` interface  
**Rust crate:** `rustsynth_eval`, `rustsynth_scene`  
**Implements:** T08

| Command | Type | Status |
|---|---|---|
| `set maxdepth N` | integer | 🏗 |
| `set maxobjects N` | integer | 🏗 |
| `set maxsize F` | float | 🏗 |
| `set minsize F` | float | 🏗 |
| `set seed N` | integer | 🏗 |
| `set seed initial` | keyword | 🏗 |
| `set syncrandom true/false` | bool | 🏗 |
| `set rng old/new` | keyword | ⏭ |
| `set background <color>` | color string | 🏗 |
| `set colorpool <type>` | pool spec | 🏗 |
| `set recursion depth` | keyword — enables depth-first | 🏗 |
| `set translation [x y z]` | vector | 🏗 |
| `set rotation [9 floats]` | matrix | 🏗 |
| `set pivot [x y z]` | vector | 🏗 |
| `set scale F` | float | 🏗 |
| `set perspective-angle F` | float | 🏗 |
| `set raytracer::*` | raytracer config passthrough | ⏭ |
| `set template *` | template renderer passthrough | 🏗 |

---

## 9. Canonical Scene Representation

**Legacy source:** `Renderer.h` draw calls, `State`, `PrimitiveClass`  
**Rust crate:** `rustsynth_scene`  
**Implements:** T10

| Feature | Status |
|---|---|
| `SceneObject` with kind, world transform, RGBA color, alpha, tag | 🏗 |
| `CameraState` (translation, rotation, pivot, scale) | 🏗 |
| Background color | 🏗 |
| Scene as flat `Vec<SceneObject>` | 🏗 |
| Renderer boundary trait (`ViewportBackend`) | 🏗 |

---

## 10. Template Exporter

**Legacy source:** `StructureSynth/Model/Rendering/TemplateRenderer.{h,cpp}`, `Misc/*.rendertemplate`  
**Rust crate:** `rustsynth_export_template`  
**Implements:** T11

| Feature | Status |
|---|---|
| Load `.rendertemplate` file | ❌ |
| Variable substitution in template | ❌ |
| Per-primitive template expansion | ❌ |
| Sunflow, POV-Ray, RenderMan, Blender templates | ❌ |
| Template export dialog (UI) | ❌ (T20) |

---

## 11. OBJ Exporter

**Legacy source:** `StructureSynth/Model/Rendering/ObjRenderer.{h,cpp}`  
**Rust crate:** `rustsynth_export_obj`  
**Implements:** T12

| Feature | Status |
|---|---|
| Box geometry | ❌ |
| Sphere tessellation (configurable segments) | ❌ |
| Cylinder geometry | ❌ |
| Grouping by tag/class | ❌ |
| Color / material (mtl) output | ❌ |

---

## 12. JavaScript Automation

**Legacy source:** `StructureSynth/JavaScriptSupport/JavaScriptBuilder.*`  
**Rust crate:** `rustsynth_script` (optional/future)  
**Implements:** T13

| Feature | Status |
|---|---|
| Load and run JS automation scripts | ⏭ |
| `build()` / `render()` / export from JS | ⏭ |
| Variable substitution from JS | ⏭ |

---

## 13. Desktop Application Shell

**Legacy source:** `StructureSynth/GUI/MainWindow.*`, `VariableEditor.*`, `TemplateExportDialog.*`  
**Rust crate:** `rustsynth_app_gtk`  
**Implements:** T14

| Feature | Status |
|---|---|
| Multi-tab code editor | ❌ |
| Syntax highlighting | ❌ |
| File open/save/recent | ❌ |
| Example browser | ❌ |
| Seed picker | ❌ |
| Render action | ❌ |
| Variable editor (sliders from preprocessor params) | ❌ (T18) |
| Template export dialog | ❌ (T20) |
| OBJ export dialog | ❌ (T21) |
| Camera settings import/export | ❌ (T19) |
| Screenshot / export flow | ❌ |
| Drag and drop | ❌ |

---

## 14. Viewport Rendering

**Legacy source:** `SyntopiaCore/GLEngine/`, `OpenGLRenderer.*`  
**Rust crates:** `rustsynth_viewport_bevy`, `rustsynth_viewport_gl`  
**Implements:** T15 (Bevy), T16 (OpenGL)

| Feature | Status |
|---|---|
| Render `box`, `sphere`, `cylinder`, `line`, `dot`, `grid`, `mesh` | ❌ |
| Camera control (orbit/pan/zoom) | ❌ |
| Real-time scene update | ❌ |
| Screenshot/export | ❌ |
| Integrated raytracer | ⏭ (T22) |

---

## Priority for parity

The order in which features should be implemented to unblock downstream work:

1. **Preprocessor** (T04) — all examples depend on this
2. **Lexer** (T05) — required by parser
3. **Parser + AST** (T06) — required by semantics
4. **Name resolution + rule graph** (T07) — required by evaluator
5. **Evaluator / builder** (T08) — emits canonical scenes
6. **Transformations + state parity** (T09) — correctness baseline
7. **Canonical scene + renderer boundary** (T10, T10A) — unblocks all exporters and viewports
8. **OBJ + template exporters** (T11, T12) — v1 export requirements
9. **GTK4 + Relm4 shell** (T14)
10. **Viewport backend** (T15 or T16)
