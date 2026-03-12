# T02 — Next

## Recommended next task
T03 (golden fixtures) is also now `READY` and can be completed immediately. T04 is the first `BLOCKED` task that becomes `READY` once T01, T02, and T03 are all done.

## Known gaps found during this audit

1. **Arithmetic in number tokens** — `Menger.es` uses `s 1/3` (division in scale arguments). The legacy tokenizer does not show arithmetic evaluation; it is likely that `1/3` is tokenized as a single `Number` symbol and evaluated by `getNumerical()`. This needs to be verified in `Tokenizer.cpp` before T05.

2. **`#include` support** — The EBNF comment inside `EisenParser.cpp` mentions `#include "../basicstuff.es"` but `Preprocessor.cpp` has no implementation for it. Either it is stripped as a comment or it was never implemented. Must be investigated before T04.

3. **`c` and `v` aliases** — The tokenizer lists `c` and `v` as operator keywords. `c` appears to be an alias for `color`; `v` for brightness/value. Both need to be confirmed against legacy behavior tests.

4. **`md` and `w` aliases** — `md` = `maxdepth`, `w` = `weight`. Confirmed in tokenizer operator list.

5. **`triangle[...]` syntax** — Referenced in `RuleSet.cpp` via `PrimitiveRule` but the inline syntax is not fully traced. Must be looked up in `PrimitiveRule.cpp` before T07.

6. **`set rng old/new`** — The legacy app has an `old` RNG mode using a different generator. This is marked ⏭ but the seed parity tests in T09 must confirm the `new` RNG is sufficient for all examples.

## Unanswered questions
- Does the legacy app evaluate `random[a,b]` at preprocessor time or at parse time? (Answer: preprocessor time — confirmed in `Preprocessor.cpp`)
- Is `color random` resolved at evaluation time using the color pool, or at parse time? (Likely evaluation time based on `Transformation::createColor("random")` checking `param.toLower() == "random"`)
