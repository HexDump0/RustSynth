# T13 — Scripting Compatibility Decision

## Decision

**EisenScript is the only scripting surface for v1. No additional scripting engine will be added.**

If a general-purpose embedded scripting layer is ever needed, **Rhai** is the preferred choice for v2+. JavaScript runtimes (Deno, QuickJS) are deferred indefinitely.

---

## Framing

The legacy StructureSynth application has exactly one scripting language: EisenScript. It has no plugin API, no JavaScript layer, and no general-purpose extension mechanism. The feature matrix (T02) confirms that EisenScript coverage is complete in `rustsynth_eisenscript` as of T09.

The question T13 asks is: *should RustSynth v1 add a broader scripting/extension mechanism beyond EisenScript?*

---

## Options Evaluated

### Option A — No additional scripting (Chosen ✅)

**Rationale:**
- EisenScript is already fully implemented and tested (T04–T09, 22 tests).
- The legacy application had no scripting beyond EisenScript; v1 parity does not require more.
- Adding an embedded runtime before the app shell (T14) and viewport (T17) exist would be premature.
- Reduces v1 scope risk and keeps the core headless crates dependency-light.

**Accepted limitation:** Power users cannot extend behaviour with custom scripts in v1.

---

### Option B — Rhai (Deferred to v2+)

[Rhai](https://rhai.rs) is a lightweight, sandboxable scripting language implemented entirely in Rust with no unsafe code and no external C dependencies.

**Why Rhai when the time comes:**
- Pure Rust, no `unsafe`, no FFI overhead.
- Can register Rust types and functions directly with zero-cost ergonomics.
- Tiny binary footprint (~400 kB release, no runtime heap allocation for scripts).
- Already used in several Rust GUI/game projects for user scripting.
- `cargo add rhai` is the only dependency change required.

**Suggested integration point (v2):**
- A `rustsynth_scripting` crate wrapping Rhai.
- Exposes the `Scene` type and a builder API to scripts.
- Allows procedural generation of scenes that EisenScript's rule-based model cannot express (e.g. parametric surface generators, lattice builders).

---

### Option C — JavaScript via QuickJS or Deno (Rejected ❌)

**Reasons for rejection:**
- QuickJS bindings (`rquickjs`) require a C compiler and add ~1 MB to binary size.
- Deno embedding is production-quality but heavy (V8, ~15 MB minimum).
- Neither has meaningful compatibility story with EisenScript.
- No user demand identified in legacy codebase or roadmap documents.
- Incompatible with the zero-C-dependency goal in `rustsynth_core`.

---

## Outcome

| Aspect | Decision |
|---|---|
| v1 scripting surface | EisenScript only (already done) |
| Extension API | None in v1 |
| Future scripting engine | Rhai, when/if needed |
| JS runtimes | Deferred indefinitely |
| New code required by T13 | None |

T13 is a decision-only task. No new crates, no new dependencies.

---

## Impact on Downstream Tasks

| Task | Impact |
|---|---|
| T14 (GTK4 app shell) | No impact — shell loads `.es` files only |
| T18 (variable editor) | No impact — variables come from preprocessor metadata, not a scripting engine |
| T24 (parity regression) | No impact — golden fixtures are all EisenScript |
| Hypothetical `rustsynth_scripting` | Blocked until after v1; Rhai is the chosen engine when unblocked |
