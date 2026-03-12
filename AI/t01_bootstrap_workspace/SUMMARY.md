# T01 — Bootstrap Rust workspace

## Task goal
Create a compilable, well-structured Cargo workspace in `RustSynth/` with skeleton crates for every planned module, a shared dependency manifest, lint config, and a passing test suite baseline.

## Approach
- Created `RustSynth/Cargo.toml` as a workspace root with `resolver = "2"`.
- Pinned all shared dependency versions (`anyhow`, `thiserror`, `serde`, `log`, `glam`, `env_logger`, `pretty_assertions`) in `[workspace.dependencies]`.
- Created 11 crates, each with its own `Cargo.toml` pointing at workspace deps and a `src/lib.rs` (or `src/main.rs` for `rustsynth_app_gtk`).
- Filled in real skeleton modules with doc-comments explaining what will land in each task.
- Added `.cargo/config.toml` with build aliases (`ck`, `t`).
- Added `Clippy.toml` for workspace-wide lint policy.
- Added `tests/fixtures/README.md` as placeholder for T03.

## Result
- `cargo check --workspace` → clean (1 expected dead-code warning on placeholder struct field).
- `cargo test --workspace` → 4 tests pass (color hex parsing + RNG determinism/bounds), 0 failures.

## Status
**Complete.**
