# Golden Fixture Manifest

All `.es` source files have been copied to `RustSynth/tests/fixtures/eisenscript/`.

This manifest classifies each example, lists the language features it exercises, and notes its intended use as a golden test.

---

## Tier 1 — Minimal parser smoke tests

These are the simplest possible inputs. Use them first when building T05/T06 parser tests.

| File | Features covered | Expected output notes |
|---|---|---|
| `Ball.es` | `set maxdepth`, two-weight `rule w N`, `x/rz/ry/s` transforms, `box`, bare rule call | Deterministic box stream; seed 0; small count |
| `SpiralTree2D.es` | `rule w N` (weighted ambiguous), `box`, `y/rx/s/b` transforms | Two competing rule variants; weight 100:1 |
| `Tutorials/Blend.es` | `blend color strength`, `sphere`, `x/rz/ry/s` | No preprocessor, clean parse |
| `Menger.es` | `maxdepth N > retirement`, `s 1/3` arithmetic in scale | Fractal; deterministic box count at depth 3 |

---

## Tier 2 — Core language coverage

These exercise the main language constructs. Use them for T07/T08 evaluator tests.

| File | Features covered | Expected output notes |
|---|---|---|
| `Default.es` | `set background`, camera `set`, raytracer `set`, multi-rule, `3 * { ... }` loop, `sphere::shiny` tag, `set raytracer::dof`, `set raytracer::phong` | Representative of a "full" script |
| `BinaryKite.es` | Multiple rules, `hue/sat/b/a` color transforms | Color-heavy; deterministic if seed fixed |
| `MeshTest.es` | `mesh` primitive, retirement rule, `maxobjects`, two rule variants with different weights | Tests `mesh` draw and retirement |
| `RoundTree.es` | Complex transform chains | Tree-like recursion |
| `NablaSystem.es` | Multiple interdependent rules | Tests name resolution |
| `Nabla.es` | Nested rules | Simple recursive |
| `Torus2.es` | Torus-like structure, `ry/rz/s` | Deterministic with seed |
| `Torus3.es` | Like Torus2 | |
| `Grinder.es` | Complex structures | |
| `Frame In Frame.es` | Nested frame-like structure | |
| `Konstrukt.es` | Multiple primitives | |
| `Mondrian.es` | Color focus | |
| `Pure Structure.es` | Clean structural recursion | |

---

## Tier 3 — Preprocessor tests

Use these specifically for T04 preprocessor tests.

| File | Features covered | Expected output notes |
|---|---|---|
| `Tutorials/Preprocessor.es` | `#define name value` plain substitution, multiple defines | After preprocessing: no `#define` lines remain; substituted values appear in body |
| `Tutorials/PreprocessorGUI.es` | `#define name val (float:lo-hi)`, `#define name val (int:lo-hi)`, GUI param extraction, `set seed initial` | Extracts `sizeStep` (float 0-1), `angle1` (float 0-90), `angle2` (float 0-90), `iterations` (int 1-90) |
| `Tutorials/RandomColor.es` | `set colorpool list:...`, `color random` | No preprocessor directives; tests color pool |

---

## Tier 4 — Feature-specific isolation tests

| File | Features covered | Notes |
|---|---|---|
| `Tutorials/Primitives.es` | All 6 basic primitives: `sphere`, `box`, `dot`, `line`, `grid`, `mesh` | Ideal for T07 primitive resolution tests |
| `Reflection.es` | `blend`, `a` (alpha), `set raytracer::*`, multiple simultaneous rules | Tests alpha and raytracer set commands |
| `Tutorials/TriangleComposites.es` | `triangle[x,y,z;...]` inline syntax | Tests triangle special rule; deferred until `triangle` is supported |
| `Arc Sphere.es` | Arc-based geometry, `sphere` | |
| `Synctor.es` | Detailed structure | |
| `Thingy.es` | Small script | |
| `NineWorthies.es` | Nine-fold repetition pattern | |
| `Nouveau.es`, `Nouveau2.es`, `Nouveau3.es` | Complex art-style structures | |
| `City of Glass.es` | City-block style | |
| `Moduli Creatures.es` | Complex animal-like structure | |
| `Octopod II.es` | Octopus-like tentacles | |
| `Trees 3d.es` | 3D tree | |

---

## Tier 5 — JavaScript / automation (deferred)

| File | Features covered | Notes |
|---|---|---|
| `Tutorials/JavaScript - Movie.es` | JavaScript automation header | Skip until T13 |
| `Tutorials/NouveauMovie.es` | JS animation script | Skip until T13 |
| `Tutorials/CSG test.es` | CSG geometry test | May use special commands; inspect before T07 |
| `Tutorials/Raytracer - Light.es` | Raytracer lighting config | Raytracer deferred (T22) |

---

## Expected output approach

Since the legacy app requires a running Qt environment to produce actual output, we cannot generate pre-computed golden outputs right now. Instead, the fixture test strategy for T24 is:

1. **Preprocessor output** (T04): assert that preprocessing `Tutorials/Preprocessor.es` and `Tutorials/PreprocessorGUI.es` produces the correct substituted text and extracts the right GUI parameters.

2. **Token stream** (T05): assert that tokenizing each Tier 1 example produces the expected token type sequence.

3. **AST shape** (T06): assert that parsing each Tier 1/2 example produces an AST with the expected number of rules, actions, and modifiers.

4. **Object count** (T08): assert that evaluating each Tier 1 example with a fixed seed produces the expected number of scene objects (to be determined by running the legacy app or by manual analysis).

5. **OBJ / template output** (T11, T12): assert that exporting a known Tier 1 scene snapshot produces byte-stable output.

---

## Fixture directory layout

```
RustSynth/tests/fixtures/
  README.md                     — overview (this file via MANIFEST.md)
  eisenscript/                  — all .es source files (copied from legacy)
    Ball.es
    Default.es
    Menger.es
    ...
    Tutorials/
      Blend.es
      Preprocessor.es
      PreprocessorGUI.es
      Primitives.es
      RandomColor.es
      ...
  golden/                       — expected outputs (populated in T24)
    (empty — to be filled when evaluator is complete)
```
