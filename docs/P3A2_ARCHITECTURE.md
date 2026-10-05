# P3-A.2 — Hybrid Structural Reinforcement Architecture

**Status:** architecture frozen first, then implemented on `feat/p3a2-hybrid-structural-reinforcement`; CI engineering gate passed. Real-image quality gate remains open. P2-B.1 remains the default/frozen baseline. P3-A.0 rotation-only and P3-A.1 placement-dominant experiments are preserved as rejected evidence.

## Why a hybrid?

P3-A.0 changed direction for 581/12,671 portrait strokes but slightly regressed tone/edge metrics and did not improve the preview.

P3-A.1 changed the *whole tonal placement field*: strongly nonhorizontal strokes rose **97→2367** at the same 12,671 total stroke count. It made the drawing much more directional but damaged the source's value structure: tone RMSE **0.397436→0.529120** and dark RMSE **0.447258→0.601276**.

These failures isolate a useful conclusion:

> **P2's tonal field is still our best value carrier. Directional structure should reinforce it, not replace it.**

## Controlled-budget design

For one input/source/options:

1. Generate **P2-A tone only** (contours disabled). This exact ordered stroke list is the immutable tonal prefix.
2. Generate frozen **P2-B.1** with contours enabled.
3. Compute:
   - `tone_count`
   - `baseline_total_count`
   - `structural_budget = baseline_total_count - tone_count`
   - original P2-B.1 contour records = baseline tail after `tone_count`.
4. Render the tone-only prefix over white.
5. Compare source linear darkness against rendered tonal darkness:
   ```
   positive_residual = max(source_darkness - tonal_preview_darkness, 0)
   ```
6. Build multiscale structural candidates only where:
   - source support is dark enough;
   - structure-tensor coherence/energy is reliable;
   - positive residual exceeds a small threshold;
   - the **whole line + round caps + lateral footprint** is supported by the original unsmoothed source.
7. Rank candidates by a bounded combination of:
   - positive tonal residual,
   - structure confidence,
   - target darkness,
   - mild coarse/fine preference.
8. Select at most `structural_budget`, with local spacing and tile quotas.
9. If fewer hybrid candidates qualify, fill remaining slots with the **original P2-B.1 contour records** in deterministic order.
10. Append the structural layer after the exact tonal prefix.

Therefore:

```
P3-A.2 total stroke count == P2-B.1 total stroke count
P3-A.2 tonal prefix       == P2-A/P2-B.1 tonal prefix exactly
```

Path length, raster darkness and individual structural marks may differ and must be measured.

## Why positive residual?

P3-A.1 showed that structure can destroy tonal mass when allowed to own the whole image. P3-A.2 only spends structural budget where the frozen tonal base is **under-representing source darkness**. It does not add a directional mark simply because a gradient is strong.

This is not full Primitive-style optimization: each candidate is not temporarily rendered and globally scored. It is a cheap, deterministic source/residual filter appropriate for the current CPU-first research stage.

## Candidate geometry

- Straight `Stroke` records only; no curves/new schema.
- Fine/coarse orientation comes from the existing multiscale structure tensor.
- Candidate midpoint is sampled/snapped to supported source-dark pixels near a small deterministic grid.
- Tangent direction follows reliable source form.
- Length remains short and pencil-like (target roughly 3–7 px).
- Width/opacity remain in the structural-accent range, deliberately lighter than using P3-A.1 as a full tonal carrier.
- White/background gaps are never authorized by blurred fields; unsmoothed source support remains final authority.

## Spatial fairness

Hair/fabric/high-frequency regions must not consume the structural budget.

Use:
- candidate spacing around accepted anchors;
- fixed 32×32 tile quotas;
- deterministic score/order tie-breaking.

A future version may make region quotas adaptive, but P3-A.2 should first test the simple bounded hypothesis.

## Explicit statistics

Expose:
- `tone_count`
- `structural_budget`
- `generated_hybrid_candidates`
- `hybrid_selected`
- `baseline_contour_fallback`

A high fallback count means the hybrid candidate generator is conservative; it is not hidden.

## CLI and branch behavior

Add one opt-in flag such as `--hybrid-structural`.

- no experiment flag: exact frozen P2-B.1
- `--directional`: historical rejected P3-A.0
- `--placement-aware`: historical rejected P3-A.1
- `--hybrid-structural`: P3-A.2

Experiment flags must be mutually exclusive.

## Engineering acceptance

Before any photograph:

- exact tonal prefix equality with P2-B.1;
- exact total stroke-count equality with P2-B.1;
- deterministic same-source/seed output;
- blank input stays blank;
- protected white channel remains clear;
- residual-aware hybrid selection actually occurs on at least one structured fixture;
- baseline contour fallback exactly fills any shortfall;
- existing stroke-limit errors remain authoritative.

## Quality gate

P3-A.2 is **not accepted** because CI passes.

For the original portrait, compare:
- p2b1.png vs p3a2.png at identical zoom;
- tone/midtone/dark/white RMSE;
- edge F1;
- highlight contamination;
- total path length;
- structural mix (hybrid vs fallback).

Desired behavior:
- preserve P2-B.1 tonal mass;
- improve specific structure/feature readability;
- avoid P3-A.1's contour-heavy whole-image look;
- avoid new white contamination.

Then repeat on a **genuine permission-cleared nonportrait**. P2-C's nonportrait gate remains open.

## Rejection criteria

Reject or revise P3-A.2 if:
- tonal prefix changes unexpectedly;
- total stroke count exceeds P2-B.1;
- dark/midtone reconstruction degrades materially without clear visual benefit;
- edge proxy rises but the drawing becomes harsher or less recognizable;
- hybrid candidates crowd one textured region;
- protected white regions regress.

P3-B optimization remains deferred until a hybrid or another P3-A proposal demonstrates a real visual win.
