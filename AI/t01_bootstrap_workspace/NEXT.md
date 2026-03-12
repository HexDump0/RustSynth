# T01 — Next

## Recommended next tasks
T02 and T03 are both `READY` and independent of T01. Both can be done now.

## Known blockers
None. T04–T08 unblock once T01, T02, and T03 are all complete.

## Unanswered questions
- `rustsynth_app_gtk` currently has a `src/main.rs` entry point. Once T14 begins it should gain a proper binary + lib split if needed.
- The `Rng` in `rustsynth_core` is a placeholder xorshift64. Confirm the exact Mersenne Twister variant used in the legacy `SyntopiaCore/Math/Random.*` before T04/T09 to ensure seed parity.
- `glam` was chosen for math types. Verify it is sufficient for the full 4×4 matrix operations needed (affine decomposition, reflect transforms) in T09.
