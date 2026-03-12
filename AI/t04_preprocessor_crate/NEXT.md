# T04 — Next

## Recommended next task
- T06 — implement the parser AST crate.

## Known blockers
- None for parser start-up.

## Notes for the next agent
- The current preprocessor removes `#define` directive lines from emitted output while keeping comment text intact.
- GUI-style `#define ... (float:...)` and `#define ... (int:...)` directives both emit metadata and substitute their default values into the rewritten text.
- Random substitution currently uses the shared Rust RNG wrapper; exact legacy RNG parity can be revisited later if evaluator-level parity requires it.
