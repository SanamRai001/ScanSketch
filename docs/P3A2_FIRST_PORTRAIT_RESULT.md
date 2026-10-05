# P3-A.2 — First Portrait Hybrid Result

**Date:** 2026-10-05. **Status:** controlled same-source P2-B.1 vs P3-A.2 portrait A/B completed after the Windows PowerShell runner fix. The private source remains local.

## Controlled invariants observed

- Same total accepted strokes: **12,671 vs 12,671**
- Exact identical tonal prefix: **12,563 strokes**
- Structural budget: **108**
- Hybrid-selected: **108**
- Original contour fallback: **0**
- Hybrid candidates generated: **2,024**
- Therefore P3-A.2 replaced the *entire* P2-B.1 structural tail while leaving tone untouched.

## Exact user-supplied metrics

| Metric | P2-B.1 | P3-A.2 | Change |
| --- | ---: | ---: | ---: |
| strokes | 12671 | 12671 | identical |
| path length px | 80222.50718482705 | 80303.49043211777 | **+0.10%** |
| tone RMSE ↓ | 0.39743605947970556 | 0.3958093160622192 | **0.41% better** |
| midtone RMSE ↓ | 0.30444623687925343 | 0.3102673724631551 | **1.91% worse** |
| dark RMSE ↓ | 0.44725756214272244 | 0.4448562194962735 | **0.54% better** |
| white RMSE ↓ | 0.016080508609430236 | 0.016080508609430236 | **identical** |
| edge F1 ↑ | 0.28870588594667923 | 0.2884075233052197 | **0.10% worse** |
| highlight ink | truncated in terminal display | truncated | not inferred |

## Visual review

The uploaded P2-B.1 and P3-A.2 previews are **extremely similar**. This is a major improvement over P3-A.1 in one sense: P3-A.2 no longer destroys the tonal body or turns the entire portrait into directional line art.

However, the first hybrid does **not** produce a convincing visual improvement:
- broad tone is preserved;
- the portrait remains recognizable at the same level as P2-B.1;
- no obvious new white contamination is visible;
- local feature readability (eyes/glasses/nose/lips) is not clearly improved;
- the structural tail changed completely, yet the edge proxy is fractionally worse;
- midtone reconstruction is measurably worse.

## Interpretation

P3-A.2 validates the **hybrid architecture**, but not the current replacement policy.

The important diagnostic is:

```text
structural budget = 108
hybrid selected   = 108
contour fallback  = 0
candidate pool    = 2024
```

The selector currently answers: *"is a hybrid candidate valid and high-ranked?"* It never asks: *"is this hybrid candidate clearly better than the original P2-B.1 contour being displaced?"*

With 2,024 candidates for only 108 slots, a full replacement is too easy. The tiny visual/numeric delta suggests the next test should be **selective structural replacement**, not a larger hybrid budget.

## Decision

**P3-A.2 = promising architecture, inconclusive first policy. Do not promote over P2-B.1.**

Next: **P3-A.2.1 selective replacement**.

- Score original P2-B.1 contour strokes using the same tone-residual / source-structure evidence used for hybrid candidates.
- Admit a hybrid replacement only if its utility exceeds a baseline structural utility threshold by a meaningful margin.
- Preserve a conservative maximum replacement fraction so the experiment cannot silently become another 100% structural rewrite.
- Keep exact tonal prefix and exact total stroke count.
- Report baseline retained vs hybrid replacement counts explicitly.

This is refinement of the hybrid policy, not P3-B optimization.
