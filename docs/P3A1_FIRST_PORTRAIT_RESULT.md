# P3-A.1 — First Portrait Result: Placement-Dominant Variant Rejected

**Date:** 2026-10-05. **Status:** controlled same-source portrait A/B completed visually and numerically. The source photograph remains private/local and is not committed. P2-B.1 remains the frozen baseline.

## Controlled pair

The user generated the P2-B.1 baseline and P3-A.1 placement-aware output from the same local `sample.jpg`, seed 42, max-side 512, using `scripts/run-p3a1-ab.ps1`. Both outputs contain **12,671 strokes**, so stroke count is matched. P3-A.1 has a longer total path length (+3.06%), so deposited ink/path usage is not perfectly matched.

The two uploaded previews show a dramatic change in drawing language: one image has strongly form-following/directional structure throughout hair and face, while the baseline remains dominated by short horizontal tonal fragments. Based on the measured geometry counts, the highly directional image is P3-A.1.

## Exact user-supplied metrics

| Metric | P2-B.1 | P3-A.1 | Change |
| --- | ---: | ---: | ---: |
| Total strokes | 12671 | 12671 | identical |
| Strongly nonhorizontal strokes | 97 | 2367 | **24.4×** (about +2340%) |
| Total path length px | 80222.50718482705 | 82681.05842941202 | **+3.06%** |
| Overall tone RMSE ↓ | 0.39743605947970556 | 0.5291196784337965 | **+33.13% worse** |
| Midtone RMSE ↓ | 0.30444623687925343 | 0.3242132342493189 | **+6.49% worse** |
| Dark RMSE ↓ | 0.44725756214272244 | 0.6012755912381827 | **+34.44% worse** |
| White RMSE ↓ | 0.016080508609430236 | 0.015784026790158678 | **~1.84% better** |
| Edge F1 | terminal table truncated | terminal table truncated | not recorded numerically |
| Highlight ink | terminal table omitted/truncated | omitted/truncated | not inferred |

Do **not** invent the truncated edge/highlight values.

## Visual review

P3-A.1 clearly succeeds at one narrow goal: it removes much of the rigid horizontal-only look. Hair and some silhouette/feature boundaries follow local source geometry more strongly.

However, it fails as the main tonal renderer:
- broad dark masses are much less faithfully reconstructed;
- the face becomes contour-heavy / structurally overdrawn;
- large flat or dark regions lose stable tonal coverage;
- local directional detail overwhelms the tonal body;
- the resulting picture is more "line-art directional" but less faithful to the source's values;
- the very large dark-region RMSE increase matches the visible loss of dark mass.

This is stronger evidence than P3-A.0 because P3-A.1 changes **2,367 strongly nonhorizontal strokes** while keeping total count fixed. The failure is therefore not "too few directional marks." It is that directional placement is being asked to carry too much of the tonal reconstruction.

## Decision

**Reject P3-A.1 as a replacement tonal renderer. Preserve it as evidence.**

The next hypothesis is **P3-A.2 Hybrid Structural Reinforcement**:

1. freeze the P2 tonal body as the primary carrier of values;
2. keep a very small structural budget;
3. spend that budget on structure-aware source-driven accents where P2 under-describes form;
4. avoid increasing total accepted stroke count by reusing/replacing the existing P2-B.1 contour budget where practical;
5. score accents partly by positive tonal residual and structural confidence so marks must "earn" the structural budget;
6. retain strict source-white support and deterministic top-to-bottom ordering;
7. reject any hybrid that improves edge proxies while visibly damaging tone.

This result supports **hybrid augmentation**, not another whole-field directional rewrite.

The genuine nonportrait photographic gate remains open.
