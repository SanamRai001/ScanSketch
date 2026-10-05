# P3-A.2.2 — Missing-Structure Residual Architecture

**Status:** architecture frozen before implementation on `feat/p3a22-missing-structure-residual`; implementation now passes the engineering CI gate. P3-A.2.1 is preserved as the first promising selective-hybrid result. P2-B.1 remains the default baseline.

## Motivation

P3-A.2.1 improved edge F1 by **15.22%** at exact tonal-prefix and total-stroke parity, but direct image comparison showed most changed pixels were concentrated in upper hair/silhouette. The selector learned to prefer strong/easy structure, not necessarily structure the baseline actually failed to draw.

This experiment changes only one concept:

> **Score structural value by what P2-B.1 is missing, not by source strength alone.**

Everything else remains frozen:
- same P2-B.1 baseline;
- same exact tonal prefix;
- same total stroke count;
- same P3-A.2 candidate generator;
- same 40% replacement cap;
- same 15% + 0.001 replacement margin;
- same white/support checks;
- same deterministic replacement slots.

## Missing-edge residual

Render frozen P2-B.1 and compare its continuous edge-strength map to the source using the same 3×3 binomial smoothing + Sobel family already used by the P2-C edge proxy.

For each pixel:

```text
missing_edge = max(source_edge_strength - baseline_edge_strength, 0)
```

This is a continuous map, not a binary face/object detector.

Interpretation:
- strong source edge already represented by baseline → low deficit;
- source edge weak/missing in baseline → high deficit;
- source-white region with no source edge → no structural reward.

## Controlled candidate pool

Do **not** change candidate generation in P3-A.2.2.

Reuse the exact P3-A.2/P3-A.2.1 source-supported tangent candidate pool. This isolates the experiment to **utility/scoring**.

If this fails because useful missing structures never enter the existing candidate pool, that becomes explicit evidence for a later proposal-generation change.

## Shared missing-structure utility

Score both baseline contour strokes and hybrid candidates using the same sampled fields:

- mean missing-edge residual;
- mean positive tonal residual after the frozen P2 tonal prefix;
- mean source darkness;
- local source-tangent confidence/alignment.

A candidate utility should be dominated by missing-edge evidence, with tone residual as a secondary stabilizer.

Conceptually:

```text
utility =
    (0.75 * missing_edge + 0.25 * positive_tone_residual)
    * structural_alignment_factor
    * source_support_factor
```

The exact bounded constants live in code and tests.

## Replacement policy

Keep P3-A.2.1 unchanged:

1. score baseline contours weakest-first;
2. score hybrid candidates strongest-first;
3. replace only if:
   ```text
   hybrid_utility >= baseline_utility * 1.15 + 0.001
   ```
4. stop when the strongest remaining hybrid cannot beat the weakest remaining baseline;
5. cap replacements at **40% of structural budget**;
6. substitute in the exact original structural slots.

## Required statistics

Expose:
- tone count;
- structural budget;
- max replacements;
- replacements;
- baseline retained;
- candidate count;
- weakest baseline utility;
- strongest hybrid utility;
- optionally mean/maximum missing-edge evidence at replaced slots.

## Engineering gates

- exact tonal prefix equality;
- exact total stroke count equality;
- changed structural slots == replacements;
- replacement cap respected;
- deterministic output;
- blank/no-contour behavior preserved;
- protected white channel clear;
- P3-A.2 and P3-A.2.1 remain reproducible and unchanged.

## Quality gate

First compare on the same private portrait:
- P2-B.1 vs P3-A.2.2;
- inspect whether changed structure moves toward weak interior features rather than upper hair;
- compare tone/midtone/dark/white RMSE, edge F1 and path length;
- use image-difference localization, not only aggregate F1.

Then run a **genuine permission-cleared nonportrait** before claiming generality.

A higher edge F1 is still insufficient if all gains come from already-obvious silhouette regions.

This remains P3-A proposal/selection research, not P3-B rendered candidate optimization.
