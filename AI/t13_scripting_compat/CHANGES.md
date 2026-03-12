# T13 Changes

## What was done

- Evaluated three scripting compatibility approaches: no additional scripting, Rhai, and JavaScript runtimes.
- Decided: **EisenScript only for v1; Rhai preferred for v2+ if a general scripting layer is ever needed; JS runtimes rejected.**
- No new code was written. T13 is a decision-only task.

## Files created

- `AI/t13_scripting_compat/SUMMARY.md` — full decision rationale
- `AI/t13_scripting_compat/NEXT.md` — recommended next task
- `AI/t13_scripting_compat/CHANGES.md` — this file

## MASTER_TODO.md updates

- T10A: `READY` → `DONE`
- T11: `READY` → `DONE`
- T12: `READY` → `DONE`
- T13: `READY` → `DONE`
- T14: `BLOCKED` → `READY` (T10A dependency satisfied)
- T17: `BLOCKED` → `READY` (T10A dependency satisfied)
