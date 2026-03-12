# Next

## Recommended next task

Pick one of these ready tasks:

1. `T01` Bootstrap Rust workspace in `RustSynth/`
2. `T02` Build the legacy feature matrix and parity checklist
3. `T03` Collect golden fixtures from legacy examples

## Suggested order

If only one agent is working:

1. do `T01`
2. then `T02`
3. then `T03`

If multiple agents can work independently, `T01`, `T02`, and `T03` can proceed in parallel.

## Key design guidance for the next agent

- Do not start with Bevy-only app code.
- Keep the Rust workspace ready for headless crates first.
- Treat `StructureSynth/Structure Synth Source Code/Examples/` as a future test corpus.
- Preserve deterministic RNG behavior from the start.
- Do not bind parser/runtime crates to UI or rendering crates.
- The preferred desktop shell is now GTK4 + Relm4.
- The viewport must stay behind a renderer boundary.
- Candidate viewport backends are Bevy, custom OpenGL, and later `wgpu`.

## Open questions still worth documenting later

- Exact parity target for JavaScript automation in v1
- Whether the integrated raytracer is required for v1 or may be deferred
- Whether template renderer syntax needs byte-for-byte compatibility or practical compatibility
- Which viewport backend should be attempted first after the renderer boundary is defined
