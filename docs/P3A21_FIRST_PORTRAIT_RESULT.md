# P3-A.2.1 — First Portrait Selective-Hybrid Result

**Date:** 2026-10-05. **Status:** controlled same-source portrait A/B complete. This is the first P3 experiment with a substantial structural-proxy gain while preserving the frozen P2 tonal body, but the visual improvement remains subtle and spatially biased. P2-B.1 remains the default until the next gate.

## Controlled invariants observed

- total strokes: **12,671 vs 12,671**
- exact tonal prefix: **12,563**
- structural budget: **108**
- maximum replacements: **43**
- replacements made: **43**
- original P2-B.1 structural strokes retained: **65**
- candidate pool: **2,024**
- changed structural slots: **43**

The selector therefore hit the 40% replacement cap on this portrait, but unlike P3-A.2 it retained most of the baseline structural layer.

## Exact user-supplied metrics

| Metric | P2-B.1 | P3-A.2.1 | Change |
| --- | ---: | ---: | ---: |
| strokes | 12671 | 12671 | identical |
| path length px | 80222.50718482705 | 80280.555595078 | **+0.072%** |
| tone RMSE ↓ | 0.39743605947970556 | 0.3964551699531677 | **0.247% better** |
| midtone RMSE ↓ | 0.30444623687925343 | 0.3066983545100135 | **0.740% worse** |
| dark RMSE ↓ | 0.44725756214272244 | 0.44591278512667565 | **0.301% better** |
| white RMSE ↓ | 0.016080508609430236 | 0.016080508609430236 | **identical** |
| edge F1 ↑ | 0.28870588594667923 | 0.33264684065239297 | **15.220% better** |
| highlight ink | identical terminal value | identical terminal value | unchanged |

## Visual review

The uploaded previews remain very similar at normal viewing size. This is expected from only 43 changed structural slots over 12,671 total strokes.

Direct comparison of the 368×512 paper regions shows roughly **888 pixels changed at all (~0.47% of the canvas)**. Most visible differences are concentrated in the upper hair/silhouette region; comparatively little change reaches the lower face and weak facial landmarks.

That spatial pattern matters:
- the selector is no longer globally destructive;
- it can improve the edge proxy substantially at almost unchanged path/tone budget;
- but it preferentially spends replacements where source structure is already strong and easy to detect;
- glasses/nose/lips and other underrepresented interior structures are still not clearly improved.

## Decision

**P3-A.2.1 = first promising P3 result, but not yet a visual promotion over P2-B.1.**

Do not simply raise the 40% cap. The next hypothesis should change **what counts as valuable structure**, not how much structure is allowed.

### Next: P3-A.2.2 missing-structure residual

Use the frozen P2-B.1 preview itself to identify **source structure that the baseline failed to reproduce**:

```text
missing_edge = max(source_edge_strength - baseline_preview_edge_strength, 0)
```

Then:
- generate source-supported tangent candidates as before;
- rank/select using missing-edge residual + positive tonal residual + tensor alignment;
- retain the direct baseline-vs-hybrid utility comparison;
- keep the same exact tonal prefix, exact total stroke count and conservative replacement cap;
- maintain tile/spatial fairness;
- do not add face-specific semantics.

This should downweight already-well-described hair/silhouette edges and prioritize genuinely missing structure, whether it is glasses, architecture, object boundaries or other details.

A genuine nonportrait photographic gate is still required before any generality claim.
