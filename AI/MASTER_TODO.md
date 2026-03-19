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
| T00A | DONE | Confirm app-shell and viewport direction | T00 | architecture: Rust core+CLI / Tauri+React+Three.js UI |
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
| T10A | DONE | Define renderer boundary and viewport API | T10 | geometry adapters merged into `rustsynth_scene`; `render_api` crate removed |
| T11 | DONE | Implement template exporter | T10 | template export backend and tests |
| T12 | DONE | Implement OBJ exporter | T10 | OBJ backend and tests |
| T13 | DONE | Decide scripting compatibility approach | T02 | decision doc: JS compatibility, Rhai, or deferred scope |
| T14 | DONE | Build GTK4 + Relm4 desktop app shell | T01, T10, T10A | app window, editor, file IO, settings shell |
| T15 | REMOVED | ~~Bevy viewport~~ | - | superseded by Three.js in web UI |
| T16 | REMOVED | ~~OpenGL viewport~~ | - | superseded by Three.js in web UI |
| T17 | REMOVED | ~~wgpu viewport~~ | - | superseded by Three.js in web UI |
| T18 | DONE | Implement variable editor and parameter controls | T04, T14 | UI for preprocessor-driven variables |
| T19 | DONE | Implement camera/settings import-export | T10, T14 | camera state persistence and script insertion support |
| T20 | DONE | Integrate template export UI flow | T11, T14 | export dialog and file output flow |
| T21 | DONE | Integrate OBJ export UI flow | T12, T14 | export dialog and file output flow |
| T22 | BLOCKED | Decide raytracer strategy for v1 | T02, T10, T10A | decision doc: reimplement, replace, or defer |
| T23 | READY | Port examples and gallery workflow | T14 | example browser/import path and curated sample set |
| T24 | READY | End-to-end parity regression suite | T08, T11, T12, T14 | golden tests and parity report |
| T25 | DONE | Performance profiling and optimization pass | T27 | Three.js InstancedMesh grouping and optimization |
| T26 | BLOCKED | Packaging, docs, and v1 release prep | T24 | user docs, build docs, release checklist |
| T27 | DONE | Migrate to Tauri + React + Three.js | T14, T17 | Tauri v2 app shell, React frontend, Three.js viewport |
| T28 | READY | Add syntax highlighting to Tauri app | T27 | syntax highlighting editor component in React |
| T29 | DONE | Simplify architecture: CLI + remove render_api | T27 | standalone `rustsynth_cli` binary, `render_api` merged into `rustsynth_scene` |

## Notes

- Architecture is two layers: Rust core+CLI (compute) / Web UI (Tauri+React+Three.js rendering).
- `T04` through `T12` keep the core fully headless.
- The Rust core has no UI dependencies — it can be used as a library or via the `rustsynth` CLI.
- The web layer (Three.js viewport, React editor) does no computation — it calls Rust over Tauri IPC.
- Viewport backends (Bevy, wgpu, OpenGL) are all superseded by Three.js in the web UI.
- If new tasks are discovered, add them here with dependencies and a short deliverable description.
