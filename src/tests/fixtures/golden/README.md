# Expected golden outputs

This directory is intentionally empty until T24 (end-to-end parity regression suite).

Once the evaluator (T08), template exporter (T11), and OBJ exporter (T12) are complete:

1. Run the Rust evaluator on each Tier 1 / Tier 2 fixture with a fixed seed.
2. Save the resulting object-count snapshots here as `.json` files.
3. Save OBJ and template export outputs for round-trip regression.

Format proposal for object-count snapshots:
```json
{
  "fixture": "eisenscript/Ball.es",
  "seed": 0,
  "object_count": 42,
  "primitive_counts": { "box": 42 }
}
```
