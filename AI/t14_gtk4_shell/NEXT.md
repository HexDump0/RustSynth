# T14 — Next Steps

## Newly Unblocked Tasks

With T14 complete the following tasks are now **READY**:

| Task | Description | Was Blocked On |
|---|---|---|
| **T18** | Variable editor and parameter controls | T04 ✓, T14 ✓ |
| **T19** | Camera / settings import-export | T10 ✓, T14 ✓ |
| **T20** | Template export UI flow (full round-trip) | T11 ✓, T14 ✓ |
| **T21** | OBJ export UI flow (full round-trip) | T12 ✓, T14 ✓ |
| **T23** | Port StructureSynth examples and gallery workflow | T14 ✓ |

## Recommended Next Priority

**T20 (Template export UI)** and **T21 (OBJ export UI)** are the lowest-effort
items since export backends already exist (T11 and T12).  The app shell already
has skeleton handlers for both; they just need UI polish and real integration
testing end-to-end with actual examples.

**T18 (Variable editor)** adds real interactive value and is a good candidate
after T20/T21.

**T23 (Port examples)** should be done in parallel with UI tasks — each ported
example validates the pipeline is correct.

## Known Deferred Issues in T14 to Address

1. **Viewport resize signal**: `gtk::Picture` has no resize event.  Consider:
   - Replace `Picture` with a `DrawingArea` that calls `set_draw_func` and blits
     the texture, which has a native `connect_resize`.
   - Or add a `SizeGroup` / `Allocation` monitor via `connect_notify("allocation")`.

2. **Threaded rendering**: The current render path blocks the GTK main thread.
   For scenes with many objects this causes UI freezes.  Solution: move
   `run_pipeline` + `render_to_pixels` to a `tokio::task::spawn_blocking` or
   `rayon` thread, then send result back via `sender.input()`.

3. **Error reporting in UI**: Compiler errors are shown in the status bar as a
   single line.  A proper error panel or popover would improve UX.

4. **Save-on-close**: No `delete-event` handler; unsaved changes are silently
   discarded.  Add a confirmation dialog.
