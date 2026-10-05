# P3-G1 — First Nonportrait Generalization Result

**Date:** 2026-10-05. **Status:** first genuine nonportrait pack complete on chair, mug and plant photographs. **P3-A.2.3 does not generalize strongly enough to promote.** More importantly, visual review identifies the current short-mark vocabulary as the dominant perceptual bottleneck.

Original photographs and generated review assets remain local.

## Pack

Three distinct nonportrait source classes were tested with the frozen P3-A.2.3 harness:

- `chair.jpg` — manufactured/furniture structure;
- `mug.jpg` — simple manufactured curved object;
- `plant.jpg` — organic/natural structure.

Every comparison used frozen P2-B.1 vs P3-A.2.3, seed 42, max side 512 and exact invariant checks.

## Aggregate result

Metric win counts:

| Metric | P3-A.2.3 wins |
| --- | ---: |
| edge F1 | **1 / 3** |
| tone RMSE | **0 / 3** |
| midtone RMSE | **0 / 3** |
| dark RMSE | **1 / 3** |
| white non-worse | **2 / 3** |

Exploratory mean percentage changes, where positive means improvement:

| Metric | Mean change |
| --- | ---: |
| edge F1 | **-2.1368%** |
| tone | **-0.5959%** |
| midtone | **-1.8690%** |
| dark | **-0.3648%** |
| path length | **+0.2496%** |

The small pack is not a statistical benchmark, but it is sufficient to reject a universal promotion claim.

## Per-image result

### chair.jpg

- replacements: **28 / 108** structural slots;
- candidates: **238**;
- edge F1: **-0.7803%**;
- tone: **-0.6087%**;
- midtone: **-4.2762%**;
- dark: **-0.4223%**.

### mug.jpg

- replacements: **24 / 95**;
- candidates: **202**;
- edge F1: **-6.6462%**;
- tone: **-0.3991%**;
- midtone: **-0.6062%**;
- dark: **+0.1495%**.

### plant.jpg

- replacements: **29 / 115**;
- candidates: **214**;
- edge F1: **+1.0162%**;
- tone: **-0.7800%**;
- midtone: **-0.7247%**;
- dark: **-0.8217%**.

## Visual finding: the larger problem is the mark language

The generated chair, mug and plant are recognizable, but both P2-B.1 and P3-A.2.3 still look like **scanline reconstructions**, not ordinary human sketches.

The visible symptoms are consistent across all three source classes:

- large regions are filled with repeated short horizontal dashes;
- important object contours are represented by many tiny pieces rather than confident continuous strokes;
- curved forms such as the mug body/handle are outlined but not drawn with a flowing hand motion;
- the plant leaves and chair structure are recognizable, yet the drawing lacks gestural stroke hierarchy;
- P3-A.2.3 changes which small marks appear, but the fundamental visual vocabulary remains the same.

This is supported directly by the implementation:

- P2 tonal fragment requested length is clamped to **2.0–10.5 px**;
- P3 hybrid structural candidate length is clamped to **2.6–7.2 px**;
- P2-B.1 contour accents are likewise short local marks.

At a 512px working image these are intentionally tiny. **Long gestural strokes have not yet been implemented.**

## Decision

**P3-G1 rejects promotion of P3-A.2.3 as a general renderer.**

Do not continue tuning missing-edge candidate ranking, cap or replacement thresholds yet. Those mechanisms operate inside the wrong primitive vocabulary for the desired visual target.

The next major research phase must address **stroke language itself**.

### Next: P4-A human stroke hierarchy

The new renderer should support at least three mark scales:

1. **Long structural / gestural strokes**
   - continuous major silhouette and form boundaries;
   - substantially longer than current 3–10px fragments;
   - able to curve through coherent source structure.

2. **Medium form-following strokes**
   - describe planes, folds, seams and interior form;
   - align with local structure rather than default horizontal scanlines.

3. **Short hatching / texture strokes**
   - current fragment vocabulary remains useful here;
   - used selectively for tone and texture, not as the dominant representation of the entire object.

A human-like sketch should be dominated perceptually by the first two classes, with short hatching supporting them.

## Architectural implication

True long curved pencil marks should be **first-class editable records**, not merely dozens of disconnected straight `Stroke` records that happen to touch.

P4 should therefore evaluate a path/chain representation (polyline or Bézier-like stroke path) with:
- deterministic control points;
- one logical stroke identity;
- variable width/opacity along or per path;
- source-edge corridor validation;
- curvature stopping rules;
- top-to-bottom deterministic ordering retained where possible.

The current straight `Stroke {x0,y0,x1,y1,...}` schema remains useful for hatching and compatibility but is insufficient by itself for natural curved gesture strokes.

## Research lesson

P1–P3 answered useful lower-level questions:
- deterministic paper-safe marks work;
- contour evidence improves structural proxy;
- full directional replacement can destroy tone;
- selective replacement is safer;
- proposal origin matters;
- but better selection of tiny primitives does not automatically create human drawing behavior.

P3-G1 is therefore a valid stopping gate rather than a failure of the project: it tells us the next variable should be **primitive/stroke hierarchy**, not another edge score.
