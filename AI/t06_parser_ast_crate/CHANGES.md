# T06 — Changes

## Files changed

### `RustSynth/crates/rustsynth_eisenscript/src/ast.rs`
Replaced stub. Full AST node types:
- `Script { statements: Vec<Statement> }`
- `Statement::Rule(RuleDef) | Action(Action) | Set(SetCmd)`
- `RuleDef { name, weight, max_depth, retirement, body: Vec<BodyItem> }`
- `BodyItem::Action(Action) | SetCmd(SetCmd)`
- `Action { loops: Vec<TransformLoop>, target: String }`
- `TransformLoop { count: u32, transforms: Vec<TransformOp> }`
- `SetCmd { key: String, value: String }`
- `TransformOp` enum — all legacy operators:
  `X/Y/Z` (translate), `Rx/Ry/Rz` (rotate), `Sx/Sy/Sz/S` (scale), `Fx/Fy/Fz` (flip/reflect),
  `ReflectNx/Ny/Nz` (plane reflection), `Matrix([f64;9])`, `Hue/Sat/Brightness/Alpha`,
  `Color(String)`, `Blend { color: String, strength: f64 }`

### `RustSynth/crates/rustsynth_eisenscript/src/parser.rs`
Replaced stub. Recursive-descent parser:
- `parse(source: &str) -> ParseResult` — runs lexer then parser, returns script + diagnostics.
- Internal `Parser { tokens, pos, diagnostics }`.
- Key methods: `parse_script`, `parse_rule`, `parse_set_cmd`, `parse_action`,
  `parse_transform_list`, `parse_transform_op`, `collect_target`.
- Handles: multiply syntax (`N * { ... } target`), `w N` weight, `maxdepth N > retirement`,
  bare rule-name actions, `triangle[payload]` merge, fraction numbers.

### `AI/MASTER_TODO.md`
Marked T06 `DONE`, T07 changed from `BLOCKED` to `DONE` (resolved in same session).

## Tests run
- `cargo test -p rustsynth_eisenscript` — 22 passed
- `cargo test --workspace` — 51 passed, 0 failed
