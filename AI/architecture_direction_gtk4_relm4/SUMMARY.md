# Summary

## Task

Record and propagate the current architecture decision for the Rust rewrite.

## Decision

The current architecture direction is:

- headless core first
- `gtk4-rs` + `Relm4` for the desktop application shell
- no commitment yet to a pure GTK-rendered viewport
- a renderer boundary must keep viewport implementations swappable

Current viewport candidates:

- Bevy
- custom OpenGL
- later `wgpu`

## Why this direction was chosen

- Structure Synth is primarily a desktop tool/editor, not a game-engine-first app
- GTK4 + Relm4 is a better fit for menus, panes, dialogs, settings, and file workflows
- a headless core reduces risk and improves testability
- the viewport is important, but it should not determine the whole architecture too early

## Outcome

Planning documents and AI instructions were updated to reflect this as the active project direction.
