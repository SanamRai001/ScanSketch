# Visual Review — P1 through P2-B.1

Status: **qualitative observations from the user's first portrait, 2026-10-04**. Not a quantified benchmark or independent human study. Keep the image local; do not commit the original portrait or screenshots without explicit permission. Original was a portrait with prominent dark hair, spectacles and a light background. The generated working image was **368×512** at seed **42** for recorded P2-A/P2-B runs.

## Observed sequence

| Version | Baseline/branch | Actual evidence | Qualitative outcome |
| --- | --- | --- | --- |
| P1 | `feat/p1-rust-scanline-baseline` | Windows check succeeded; 11/11 core tests; CLI produced actual stroke preview | Silhouette and light background represented, but continuous horizontal bars resembled engraving/halftone. Eyes, glasses, nose and lips underrepresented. |
| P2-A | `feat/p2-sketch-stroke-language` | CI check and 14/14 tests; a preview using the same subject | Broken fragments reduce uninterrupted bars, but the subject still reads largely as tone texture, with overly uniform treatment of hair and face. |
| P2-B | `feat/p2b-contour-reinforcement` | CI check and 20/20 tests; same-photo CLI outputs: `--no-contours` 12,563 strokes, contour-on 13,024 strokes (**+461**) | Difference was subtle. Image-derived contours did not strongly clarify main facial anchors, despite valid added strokes. |
| P2-B.1 | `feat/p2b1-contour-coherence` | CI check and 23/23 tests; one new same-subject preview inspected | Current strongest **qualitative candidate** among the shown variations: somewhat more coherent, but improvement over P2-B remains incremental; key face structures still weak. Stroke count and runtime of this preview were **not supplied**. |

The P2-B source/control screenshot order was assumed from the user's earlier instructed output order; the exact output files were not independently inspected as binary artifacts. Subjective comparisons are provisional and should be repeated with saved, clearly labelled PNG/JSON files.

## Main diagnosis

1. Fragmenting tonal strokes yielded a more noticeable improvement than adding a few generic gradient accents.
2. A simple Sobel response is not a semantic priority mechanism. Strong hair texture can dominate while visually important glasses/eyes/lips remain underdrawn.
3. P2-B.1's smoothing, continuity ranking and regional quotas help coherence, but a further succession of threshold/opacity changes has diminishing expected returns.
4. Stroke count alone (even when additional contours are few) cannot establish perceptual or structural quality.
5. Black/white correctness, deterministic output and GitHub CI are necessary engineering gates; they are **not** proof of convincing artwork.

## Decision

- Preserve P1, P2-A and P2-B as historical baselines. Keep **P2-B.1 as the current strongest visual candidate**, not a finished default product or a statistically proven winner.
- **Stop extending the current Sobel heuristic family** until controlled P2-C comparisons establish a reason.
- Begin P2-C: fixed, permission-cleared fixtures, repeatable output records, objective metrics and blinded visual review.
- Preferred next research prototype: a **multi-scale, direction-aware stroke proposal field**. It is a larger hypothesis worth testing, not a promised improvement. Only after demonstrating improved recognizability should Primitive-inspired candidate scoring be layered in.
- Erasure/residual graphite remain deferred.

## Evidence needed to revise this decision

- Same-photo P2-B.1 JSON+PNG, a second licensed object/architecture image, plus synthetic white/shape/gradient.
- Numeric score and equal-budget comparison, along with 100%-scale crops at face anchors and blank background.
- At least two reviewers or blinded A/B evaluations if practical; record disagreement and failures, not only the favorite.
