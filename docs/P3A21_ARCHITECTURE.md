# P3-A.2.1 — Selective Hybrid Replacement Architecture

**Status:** architecture frozen before implementation on `feat/p3a21-selective-hybrid-replacement`. P3-A.2 remains preserved as the first full-replacement hybrid experiment; P2-B.1 remains the default baseline.

## Why this refinement exists

P3-A.2 solved the largest P3 failure: it preserved the exact 12,563-stroke tonal body and avoided P3-A.1's tonal collapse. But on the first portrait it replaced **all 108/108 structural slots** from a pool of 2,024 hybrid candidates and produced only tiny metric changes with almost no visible improvement.

The missing comparison was obvious:

> P3-A.2 ranked hybrid candidates, but never scored the P2-B.1 contour it was displacing.

P3-A.2.1 asks a narrower question:

> **Can a small number of hybrid structural strokes beat the weakest existing P2-B.1 contours while most of the proven baseline structural layer remains untouched?**

## Exact invariants

For the same source/options:

1. Generate P2-A tonal-only and frozen P2-B.1.
2. Preserve the **exact P2 tonal prefix**.
3. Preserve the **exact P2-B.1 total stroke count**.
4. Use P2-B.1's contour tail as the initial structural layer.
5. Generate the same residual-aware hybrid candidate pool as P3-A.2.
6. Score **both** hybrid candidates and baseline contour strokes using one comparable utility function.
7. Replace only selected baseline contour slots; every non-replaced baseline contour remains byte-identical and in its original structural order.

## Comparable structural utility

For any structural stroke, sample its centerline against:
- positive darkness residual after the frozen tonal prefix;
- source darkness;
- fine/coarse tensor confidence near the stroke midpoint;
- alignment of the stroke tangent with the best local source tangent.

A shared utility has the form:

```text
utility = mean_positive_residual
        * structural_factor(confidence, alignment)
        * source_darkness_factor
```

The exact constants are implementation details recorded in code/tests. The key constraint is that **baseline contours and hybrid candidates are scored by the same function** before replacement decisions.

## Replacement decision

1. Apply the existing P3-A.2 spacing/tile filter to hybrid candidates.
2. Score all P2-B.1 baseline contours under the shared utility.
3. Sort:
   - hybrid candidates strongest first;
   - baseline contour slots weakest first.
4. A hybrid may replace the currently weakest available baseline contour only when:

```text
hybrid_utility >= baseline_utility * 1.15 + 0.001
```

5. Stop when that condition fails. Since hybrids are descending and baseline slots ascending, later pairs cannot improve the comparison.
6. Hard cap replacements at **40% of the structural budget**.

The margin and cap are intentional research controls, not tuned to guarantee a win.

## Why a 40% cap?

The first hybrid replaced 100% of structural strokes. This experiment needs to answer whether **selective augmentation** works, not simply rerun full replacement with another threshold.

At structural budget 108, the maximum is 43 replacements; at 20, the maximum is 8. If fewer candidates earn replacement, use fewer. Zero replacements is a valid experimental outcome.

## Output ordering

Do **not** append hybrids followed by retained contours.

Instead:
- clone the original P2-B.1 contour tail;
- replace only the chosen contour indices with hybrid strokes;
- append this modified structural tail after the exact tonal prefix.

This preserves:
- total count;
- structural list length;
- original order for every retained contour;
- exact tonal prefix;
- deterministic substitution positions.

## Required statistics

Expose:
- tonal prefix count;
- structural budget;
- candidate count;
- maximum replacements;
- replacements actually made;
- baseline contours retained;
- strongest/weakest relevant utility values if useful for diagnostics.

No hidden full replacement.

## Engineering gates

- exact tonal prefix parity;
- exact total count parity;
- replacements <= floor(40% structural budget), with a minimum cap of one when budget > 0;
- replacements + retained == structural budget;
- deterministic output;
- protected channel remains clear;
- no-contours mode remains exact P2-A;
- existing P3-A.2 behavior remains available behind `--hybrid-structural`;
- new mode gets a separate CLI flag such as `--hybrid-selective`.

## Quality gate

Compare frozen P2-B.1 vs P3-A.2.1 at the same source/seed/size.

Desired portrait behavior:
- preserve P2-B.1 tone and midtone;
- avoid wholesale structural replacement;
- improve one or more meaningful feature boundaries visibly;
- edge/tone proxies should not materially regress;
- white contamination unchanged.

Then test a genuine permission-cleared nonportrait.

This is still P3-A research, **not** P3-B rendered candidate optimization.
