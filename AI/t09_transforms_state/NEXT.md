# T09 — Next

## Recommended next task
T10 is `DONE`. The next ready tasks are **T10A**, **T11**, **T12**, **T13**.

## Notes for the next agent

### Pivot convention (critical for parity)
All rotations and non-uniform scales in EisenScript pivot about the **unit-cube centre
`(0.5, 0.5, 0.5)`**, not the origin. The helper `rotate_about_pivot` computes:

```
T(+pivot) · Rotation(axis, deg) · T(-pivot)
```

and `scale_about_center` does the same for scale. This matches `Transformation.cpp` exactly.
Any future viewport or export code that interprets `SceneObject::transform` must be aware
of this convention — objects are not centred at the origin.

### Blend formula
The legacy HSV blend is:
```
blended = (current + strength * blend_hsv) / (1 + strength)
```
This is a weighted arithmetic mean in HSV space (not RGB), matching `Transformation.cpp`.

### splitmix64 seed mixing
The xorshift64 PRNG was a fixed-point at seed 0.  The splitmix64 mixer is now applied once
in `Rng::new`, mapping 0 → a large well-distributed value.  This changes the RNG sequence
for all seeds compared to before T09.  If any future task needs to regenerate golden fixtures,
re-run from this baseline.

### What is not yet tested
- `Matrix([f64;9])` interaction with subsequent rotation/scale (compositional test).
- Blend with named colours other than the 7 SVG primaries in `named_color()`.
- `Fz` and `ReflectNx/Ny/Nz` operators (only `Fx` is tested).
