# P3-A.2.2 — First Portrait Missing-Structure Result

**Date:** 2026-10-05. **Status:** controlled same-source P2-B.1 vs P3-A.2.2 portrait A/B completed. P3-A.2.2 is **rejected as a scoring-only refinement**. The source photograph remains private/local.

## Controlled comparison

P3-A.2.2 intentionally changed only the structural utility relative to P3-A.2.1:
- same frozen P2-B.1 baseline;
- same exact tonal prefix;
- same total stroke count;
- same P3-A.2/P3-A.2.1 candidate generator;
- same 40% replacement cap;
- same 15% + 0.001 replacement margin;
- same source-support and white-space rules;
- only the utility added source-minus-baseline edge deficit.

## Exact user-supplied metrics

| Metric | P2-B.1 | P3-A.2.2 | Change |
| --- | ---: | ---: | ---: |
| strokes | 12671 | 12671 | identical |
| path length px | 80222.50718482705 | 80229.46664874525 | **+0.0087%** |
| tone RMSE ↓ | 0.39743605947970556 | 0.3972707480373136 | **0.0416% better** |
| midtone RMSE ↓ | 0.30444623687925343 | 0.305038234816305 | **0.1945% worse** |
| dark RMSE ↓ | 0.44725756214272244 | 0.4470141323876371 | **0.0545% better** |
| white RMSE ↓ | 0.016080508609430236 | 0.016080508609430236 | **identical** |
| edge F1 ↑ | 0.28870588594667923 | 0.28584278634391014 | **0.9915% worse** |
| highlight ink | same visible terminal value | same visible terminal value | unchanged in supplied table |

## Visual review

The uploaded pair remains extremely close. P3-A.2.2 does not visibly recover the weak interior structures that motivated the experiment. The face/interior remains dominated by the same tonal language, while visible structural differences remain subtle and biased toward already-obvious exterior/hair regions.

This is important because P3-A.2.2 changed only scoring. A scoring-only intervention should have moved replacements toward missing details if suitable candidates already existed.

## Interpretation

**The current candidate pool is now the leading bottleneck.**

The P3-A.2/P3-A.2.1 candidate generator is still driven primarily by:
- source darkness;
- positive tonal residual;
- the existing dark-region anchor search;
- tensor direction.

P3-A.2.2 can rank those candidates by missing-edge deficit, but it cannot choose a structural mark that was never proposed. Weak-but-important source edges in comparatively light/midtone regions may therefore be absent from the pool entirely.

This explains why changing utility alone did not reproduce P3-A.2.1's edge-F1 gain and did not move useful structure into the face/interior.

## Decision

**Reject P3-A.2.2 as a scoring-only refinement. Preserve the result.**

Do not:
- raise the 40% replacement cap;
- lower the 15%+0.001 replacement margin;
- add portrait-specific feature rules;
- add more total strokes.

### Next hypothesis: P3-A.2.3 deficit-driven candidate proposals

Keep the proven selective-replacement controls, but change **where candidate anchors come from**:

1. freeze P2-B.1, exact tonal prefix, total count, cap and replacement margin;
2. compute the same missing-edge residual;
3. generate candidate anchors from local maxima / spatially separated peaks of missing-edge residual instead of positive tonal residual;
4. derive tangent direction from the source tensor as before;
5. retain unsmoothed source support and white-gap safety;
6. score candidates with the P3-A.2.2 missing-structure utility;
7. compare them against baseline contour utilities exactly as in P3-A.2.1/P3-A.2.2;
8. substitute only earned replacements into original structural slots;
9. keep tile/spatial fairness so high-frequency hair cannot consume the pool.

This remains generic image structure recovery, not face semantics.

A genuine permission-cleared nonportrait image is still required before any generality claim.
