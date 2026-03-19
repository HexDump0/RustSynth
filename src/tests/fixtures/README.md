# RustSynth golden fixture test inputs

This directory contains curated test inputs and expected output snapshots.

## `eisenscript/`

All `.es` example files copied from `StructureSynth/Structure Synth Source Code/Examples/`.
Classified into tiers in `AI/t03_golden_fixtures/FIXTURE_MANIFEST.md`.

## `golden/`

Expected output snapshots — to be populated in T24 once the evaluator (T08) is complete.

## How to add a new fixture

1. Add the `.es` file to `eisenscript/`.
2. Classify it in `AI/t03_golden_fixtures/FIXTURE_MANIFEST.md`.
3. Once the evaluator is ready, generate the expected output and add it to `golden/`.
