# T03 — Next

## Recommended next task
T04 (preprocessor), T05 (lexer), and T06 (parser) are all now unblocked. T04 is the lowest-ID blocked task and should go first.

## Fixture notes for T04 (preprocessor)

Tier 3 fixtures are the primary inputs:
- `Tutorials/Preprocessor.es` — verify plain `#define` substitution
- `Tutorials/PreprocessorGUI.es` — verify GUI parameter extraction + `float`/`int` interval parsing
- `Ball.es` — no `#define` → output = input (regression that the preprocessor does not corrupt clean scripts)

The `random[lo,hi]` feature is not demonstrated by any fixture in isolation. A micro-fixture for it should be added as an inline test in the preprocessor crate.

## Fixture notes for T05 (lexer)

Tier 1 fixtures are the primary token-stream targets:
- `Ball.es`, `SpiralTree2D.es`, `Tutorials/Blend.es` — small enough to hand-verify token sequences
- `Menger.es` — also needed to confirm `1/3` tokenization behavior

## Fixture notes for T08 (evaluator)

Before T24, the evaluator tests should target:
1. `Ball.es` with seed 0 — expected: N boxes (compute by tracing the recursion manually or from legacy app)
2. `Menger.es` at maxdepth 3 — expected: 20 boxes (the 3D Menger sponge removes 7 of 27 cubes per level)
3. `Tutorials/Blend.es` — expected: sphere stream with increasing blend applied

## Known gaps
- `Tutorials/CSG test.es` was not read; inspect it before T07 to check if it uses any unsupported syntax.
- `triangle[...]` fixture (`Tutorials/TriangleComposites.es`) cannot be tested until the triangle special rule is implemented (deferred).
