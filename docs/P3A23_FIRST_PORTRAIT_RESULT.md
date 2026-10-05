# P3-A.2.3 — First Portrait Deficit-Proposal Result

**Date:** 2026-10-05. **Status:** controlled same-source P2-B.1 vs P3-A.2.3 portrait A/B completed. Result is **promising but provisional**; do not promote to default until genuine nonportrait validation.

The private portrait source remains local and is not committed.

## Controlled comparison

P3-A.2.3 changed only candidate proposal origin relative to P3-A.2.2:

```text
P3-A.2.2 candidate signal = positive tonal residual
P3-A.2.3 candidate signal = missing-edge residual
```

Frozen:
- P2-B.1 baseline;
- exact tonal-prefix/total-count invariants;
- 40% structural replacement cap;
- 15% + 0.001 replacement margin;
- P3-A.2.2 missing-structure utility;
- tensor direction;
- source-support / protected-white rules;
- spacing + tile fairness;
- in-place structural substitution.

The user supplied the paired metric table but did **not** supply the P3-A.2.3 portrait `deficit_proposal_stats` line in this result message. Do not invent portrait candidate/replacement counts; CI fixture counts are not substitutes.

## Exact user-supplied metrics

| Metric | P2-B.1 | P3-A.2.3 | Change |
| --- | ---: | ---: | ---: |
| strokes | 12671 | 12671 | identical |
| path length px | 80222.50718482705 | 80270.72869521737 | **+0.0601%** |
| tone RMSE ↓ | 0.39743605947970556 | 0.3975872250315626 | **0.0380% worse** |
| midtone RMSE ↓ | 0.30444623687925343 | 0.3072948856077954 | **0.9357% worse** |
| dark RMSE ↓ | 0.44725756214272244 | 0.4472088993678238 | **0.0109% better** |
| white RMSE ↓ | 0.016080508609430236 | 0.016080508609430236 | **identical** |
| edge F1 ↑ | 0.28870588594667923 | 0.314450992642274 | **8.918% better** |
| highlight ink | same visible terminal value | same visible terminal value | unchanged in supplied table |

## Visual review

The two uploaded previews remain close at normal viewing size. P3-A.2.3 does not radically change the portrait and does not reproduce P3-A.1's destructive directional rewrite.

The structural improvement is plausible but subtle. The portrait remains dominated by the established tonal language; facial/interior structure is still not strongly articulated.

This is a useful result because:
- edge F1 rises materially at unchanged stroke count and white RMSE;
- path-length increase is tiny;
- the renderer remains visually stable;
- proposal origin was the only intentionally changed algorithmic variable from P3-A.2.2.

However, **P3-A.2.1 still has the larger global portrait edge-F1 result** (0.332647). P3-A.2.3 should therefore not be described as the numerically best portrait renderer. Its value is evidence that deficit-driven proposal coverage can recover structural signal without destabilizing the tonal body.

## Decision

**P3-A.2.3 = promising proposal-origin result; advance to generalization validation, not further tuning.**

Do not:
- raise the cap;
- relax the replacement margin;
- add more strokes;
- add portrait-specific feature rules;
- tune against this one portrait before testing other source classes.

### Next gate: P3-G1 nonportrait generalization

Freeze P3-A.2.3 exactly and test several permission-cleared nonportrait photographs.

Recommended first pack:
1. simple manufactured object (mug/bottle/tool);
2. footwear/bag or another object with mixed curves and seams;
3. architecture/furniture/room detail with straight edges;
4. optional natural object/plant with irregular structure;
5. optional second object with lighter/midtone interior edges.

For each image:
- same binary;
- seed 42;
- max-side 512;
- paired P2-B.1 vs P3-A.2.3;
- exact count/prefix checks;
- p2c-v1 metrics;
- structural stats;
- manual same-zoom visual review.

Do not publish original photographs or generated outputs without permission.

Only after this pack should we decide whether to:
- keep P3-A.2.3,
- return to P3-A.2.1 selective scoring,
- combine the two ideas in a new controlled phase,
- or move toward P3-B optimization.
