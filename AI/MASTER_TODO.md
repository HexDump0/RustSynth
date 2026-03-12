# Master todo

Status values:

- `DONE` — completed and documented
- `READY` — available to start now
- `BLOCKED` — waiting on dependencies or a decision
- `LATER` — intentionally deferred

Selection rule: pick the first `READY` item with the lowest ID unless a task folder explicitly recommends a different unblocked next step.

| ID | Status | Task | Depends on | Deliverable |
| --- | --- | --- | --- | --- |
| T00 | DONE | Legacy audit, roadmap, and AI workflow setup | - | planning docs and initial context |
| T00A | DONE | Confirm app-shell and viewport direction | T00 | architecture direction: headless core, GTK4 + Relm4 shell, pluggable viewport |
| T01 | DONE | Bootstrap Rust workspace in `RustSynth/` | T00 | Cargo workspace, crate skeletons, base lint/test config |
| T02 | DONE | Build legacy feature matrix and parity checklist | T00 | feature inventory mapped to legacy modules/examples |
| T03 | DONE | Collect golden fixtures from legacy examples | T00 | curated example set, fixture manifest, expected outputs notes |
| T04 | DONE | Implement preprocessor crate | T01, T02, T03 | Rust preprocessor with tests for `#define`, random substitution, GUI parameter metadata |
| T05 | DONE | Implement lexer/tokenizer crate | T01, T02, T03 | token stream and diagnostics tests |
| T06 | DONE | Implement parser AST crate | T04, T05 | AST, parse diagnostics, parser fixtures |
| T07 | DONE | Implement semantic resolution layer | T06 | resolved rule graph, primitive resolution, validation |
| T08 | DONE | Implement evaluator/builder core | T07 | deterministic expansion engine and scene emission |
| T09 | DONE | Implement transformations and state parity tests | T08 | transform/state tests covering matrix, HSV, alpha, seed behavior |
| T10 | DONE | Define canonical scene representation | T08 | renderer-agnostic scene/object model |
| T10A | DONE | Define renderer boundary and viewport API | T10 | traits/interfaces allowing Bevy, OpenGL, or `wgpu` viewport backends |
| T11 | DONE | Implement template exporter | T10 | template export backend and tests |
| T12 | DONE | Implement OBJ exporter | T10 | OBJ backend and tests |
| T13 | DONE | Decide scripting compatibility approach | T02 | decision doc: JS compatibility, Rhai, or deferred scope |
| T14 | READY | Build GTK4 + Relm4 desktop app shell | T01, T10, T10A | app window, editor, file IO, settings shell |
| T15 | LATER | Prototype Bevy viewport backend | T01, T10, T10A | scene-to-Bevy viewport prototype (deferred — wgpu is the chosen path) |
| T16 | LATER | Prototype custom OpenGL viewport backend | T01, T10, T10A | scene-to-OpenGL viewport prototype (deferred — wgpu is the chosen path) |
| T17 | READY | Implement `wgpu` viewport backend | T01, T10, T10A | wgpu renderer via `GtkGLArea` EGL surface: geometry upload, WGSL shaders, arcball camera |
| T18 | BLOCKED | Implement variable editor and parameter controls | T04, T14 | UI for preprocessor-driven variables |
| T19 | BLOCKED | Implement camera/settings import-export | T10, T14 | camera state persistence and script insertion support |
| T20 | BLOCKED | Integrate template export UI flow | T11, T14 | export dialog and file output flow |
| T21 | BLOCKED | Integrate OBJ export UI flow | T12, T14 | export dialog and file output flow |
| T22 | BLOCKED | Decide raytracer strategy for v1 | T02, T10, T10A | decision doc: reimplement, replace, or defer |
| T23 | BLOCKED | Port examples and gallery workflow | T14 | example browser/import path and curated sample set |
| T24 | BLOCKED | End-to-end parity regression suite | T08, T11, T12, T14 | golden tests and parity report |
| T25 | BLOCKED | Performance profiling and optimization pass | T17, T24 | profiling notes and targeted fixes |
| T26 | BLOCKED | Packaging, docs, and v1 release prep | T24 | user docs, build docs, release checklist |

## Notes

- `T01`, `T02`, and `T03` can be done independently and in any order.
- `T04` through `T12` should keep the core fully headless.
- `T10A` is the key boundary task before committing to any viewport implementation.
- `T14` is the preferred shell path because GTK4 + Relm4 is now the chosen desktop framework direction.
- `T17` is the chosen viewport path: `wgpu` via `GtkGLArea` EGL surface. Bevy (T15) and custom OpenGL (T16) are deferred indefinitely.
- Do not let frontend work block the language/runtime work.
- If new tasks are discovered, add them here with dependencies and a short deliverable description.
