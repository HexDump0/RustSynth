# Next

## Recommended next tasks

The highest-value ready tasks are still:

1. `T01` Bootstrap Rust workspace in `RustSynth/`
2. `T02` Build the legacy feature matrix and parity checklist
3. `T03` Collect golden fixtures from legacy examples

## Guidance for the next agent

When bootstrapping the workspace, prefer crate names and boundaries that support:

- a headless core
- a GTK4 + Relm4 application shell
- a separate renderer boundary
- multiple viewport backends

A good follow-up after the core scene representation exists will be `T10A`, because it formalizes the boundary that keeps viewport choices flexible.
