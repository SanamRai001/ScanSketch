# P4-A.3 — Residual Hatching Rebalance Architecture

**Status:** architecture frozen before implementation on `feat/p4a3-residual-hatching`.

P4-A.1 long Gestures and P4-A.2 medium Forms are now frozen. P4-A.3 changes **tone composition only**.

## Why this phase exists

The P4-A.2 paths-only hierarchy is finally sparse and human-scale, but the old overlay still contains the full P2-B.1 horizontal scan field underneath it.

That produces the wrong visual priority:

```text
old composition:
P2-B.1 full horizontal tone
+ Gesture
+ Form
```

Even when Gesture/Form geometry is useful, the eye still sees scan bands first.

P4-A.3 reverses that priority.

## New composition

```text
1. frozen Gesture paths
2. frozen Form paths
3. render the structural hierarchy
4. measure remaining tonal need
5. add sparse short Hatch marks only where tone is still useful
```

P2-B.1 remains a historical/control renderer only.

The P4-A.3 output must contain:

- **0 legacy straight `Stroke` records**;
- exact frozen P4-A.2 Gesture paths;
- exact frozen P4-A.2 Form paths;
- new short `PathStroke { role: Hatch }` records.

Using two-point PathStroke records for hatching gives P4 explicit role-aware metrics without changing old P1-P3 straight-stroke data.

## Single changed variable

Relative to P4-A.2 paths-only:

Frozen:
- Gesture path list and order;
- Form path list and order;
- widths/opacities of Gesture/Form;
- source preprocessing;
- deterministic seed;
- path renderer;
- working image dimensions.

Added:
- residual Hatch layer.

No Gesture/Form re-tuning is allowed in P4-A.3.

## Tonal residual

Render Gesture + Form onto white paper.

Let:

```text
source_darkness    = target source darkness
structure_darkness = darkness already contributed by Gesture + Form
raw_residual       = max(source_darkness - structure_darkness, 0)
```

Do **not** attempt to reconstruct raw residual exactly.

A human sketch should leave more paper than a raster reconstruction.

P4-A.3 therefore uses a deliberately compressed tonal target:

```text
hatch_need = max(raw_residual - residual_floor, 0) * residual_gain
```

Initial frozen research constants:

- residual floor: approximately **0.06**;
- residual gain: approximately **0.58**;
- hatch white threshold: approximately **0.10–0.12**;
- hatch scan band: wider than P2-B.1 (target **5px** instead of 3px);
- one hatch layer only;
- short mark length stays roughly **2–10px**.

These values are intentionally biased toward paper preservation and sparse support.

## Structure corridor suppression

Hatching should not bury the lines that establish form.

Build a small protected corridor around every Gesture and Form path.

Inside that corridor:
- residual hatch need is set to zero or strongly suppressed.

Initial target radius:
- Gesture: about **2–3px**;
- Form: about **1–2px**.

This is not semantic masking. It is generic mark hierarchy.

## Hatch geometry

P4-A.3 intentionally does **not** solve expressive hatch direction yet.

Initial Hatch marks:
- short;
- slightly jittered;
- mostly horizontal like the proven P2 short-mark vocabulary;
- role-tagged as `Hatch`;
- substantially fewer than P2-B.1 segments.

This isolates the composition question:

> Does making hatching sparse and subordinate already fix the scanline-dominance problem?

If horizontal orientation is still too visually dominant after count rebalance, a later P4-A.3.x experiment may make Hatch direction form-aware.

Do not combine both variables in the first A.3 test.

## Determinism

Use an independent deterministic hatch RNG derived from the user seed.

Adding/removing residual hatching must not change Gesture or Form geometry.

## Required statistics

Record:

- Gesture count / length;
- Form count / length;
- Hatch count;
- Hatch total path length;
- mean Hatch length;
- structural path length share;
- Hatch path length share;
- P2-B.1 control segment count;
- Hatch-count reduction versus P2-B.1;
- logical mark count;
- paper / white-region metrics;
- reconstruction metrics as secondary evidence.

## Primary acceptance criteria

The first successful P4-A.3 result should satisfy all of these:

1. **Gesture list exactly equals P4-A.2.**
2. **Form list exactly equals P4-A.2.**
3. **No legacy P2-B.1 segments in the P4-A.3 output.**
4. Hatch count is materially below old P2-B.1 segment count.
5. Structural lines remain visually dominant.
6. White paper visibly increases relative to the old overlay.
7. Object still reads clearly.
8. Tone remains sufficient to describe dark masses.
9. The image no longer reads first as horizontal raster bands.

## Quantitative direction, not hard benchmark

Initial desired range:
- Hatch count **at least 40% lower** than P2-B.1 segment count on typical chair/mug/plant cases;
- preferably 50–80% lower if object readability remains good;
- structural path-length share materially larger than before;
- no requirement to beat P2-B.1 tone RMSE.

A modest RMSE regression is acceptable if the human-sketch visual hierarchy is substantially better.

## Validation views

For each real image generate:

1. **P2-B.1 control**
2. **P4-A.2 hierarchy only** — Gesture + Form
3. **P4-A.3 final hierarchy** — Gesture + Form + residual Hatch

The third image is the main candidate.

Optional diagnostic:
- Hatch-only view.

## Failure modes

Reject or refine P4-A.3 if:

- Hatch count remains near P2-B.1 count;
- output still looks like horizontal scan bands first;
- hatches cover Gesture/Form lines;
- large dark regions become unreadably empty;
- bright background gets unnecessary ink;
- hatching floods boundaries instead of tone interiors;
- hierarchy path geometry changes.

## Deliberately deferred

P4-A.3 does not yet add:

- directional/form-following hatch orientation;
- cross-hatching;
- graphite pressure simulation;
- smudge;
- erasing;
- paper texture;
- learned semantics;
- Primitive-style optimization.

Those remain downstream only after sparse residual composition works.

## Exit

If chair/mug/plant show:

- clear Gesture/Form hierarchy;
- substantially reduced hatch count;
- recognizability retained;
- scanline dominance visibly reduced,

then P4-A.3 passes and P4 can enter **P4-G human-likeness validation** or a very small hatch-direction refinement if still needed.
