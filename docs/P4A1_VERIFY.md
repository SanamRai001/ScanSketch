# P4-A.1 — Long Structural Path Tracing Verification

**Status:** implementation passed GitHub Actions: **74/74 Rust tests**, every historical P2/P3/P3-G1 check, and the dedicated long-path smoke. Real chair/mug/plant path-layer review remains the quality gate. [CI run 37406856611](https://github.com/SanamRai001/ScanSketch/actions/runs/37406856611).

## Hypothesis

The current renderer looks mechanical because its dominant primitives are 2–10px fragments.

P4-A.1 tests a more basic capability:

> Can ScanSketch trace sparse, continuous, source-supported structural paths in the **24–96px** scale range and store each as one editable logical gesture?

If not, there is no reason to proceed to medium form strokes or tonal rebalance.

## Algorithm

P4-A.1 does not lengthen existing horizontal scan fragments.

It:

1. computes the source darkness map;
2. computes the existing continuous source edge-strength map;
3. reuses the multiscale source structure tensor for tangent direction;
4. chooses one high-confidence edge seed per deterministic spatial cell;
5. traces forward and backward along the tangent field;
6. snaps each integration step back to nearby edge support;
7. stops on:
   - lost edge support,
   - incoherent tangent,
   - excessive local turn,
   - bounds,
   - loops,
   - maximum half-length;
8. requires whole-path edge-corridor support;
9. rejects paths below **24px**;
10. spatially suppresses seeds near accepted paths;
11. stores accepted paths as `StrokeRole::Gesture` logical polylines.

Initial path target:
- minimum: **24px**;
- typical/allowed: up to roughly **96–100px**;
- sparse canvas-dependent maximum path count.

These are research bounds, not an artistic final.

## Why edge-corridor support

Long silhouette strokes cannot require darkness underneath both sides of the pencil width. A valid boundary often separates dark/object pixels from white background.

P4-A.1 therefore follows source **edge evidence** rather than using the P2/P3 full-dark-footprint hatch check.

This is the first practical separation between:
- tonal support for hatching; and
- structural support for contour gestures.

## Two inspection modes

### `--long-structural`

Returns exact frozen P2-B.1 straight segments plus the long logical gesture layer.

This is an overlay for controlled comparison. It is expected to remain visually busy because P4-A.3 has not yet reduced old hatching.

### `--long-structural-only`

Returns the **exact same gesture paths** with zero legacy straight segments.

This is the most important P4-A.1 visual artifact. Inspect it first to answer:

> Are we tracing object structure with meaningful continuous strokes?

## Required invariants

The A/B runner verifies:

- overlay straight segments are byte/value-identical to frozen P2-B.1;
- paths-only mode contains zero old segments;
- overlay and paths-only logical paths are exactly identical;
- reported accepted path count equals serialized path count;
- measurements recognize each path as one logical mark.

## Synthetic engineering gate

On the 64px vertical step fixture:

- at least one long gesture must be accepted;
- mean long-path length must be >=24px;
- maximum path length must be >=40px;
- overlay must preserve exact baseline segments;
- paths-only must contain no hatch/contour segments.

Core tests additionally cover:
- blank source -> no long paths;
- vertical boundary -> long gesture;
- disk boundary -> genuinely curved path;
- deterministic generation;
- overlay/path-only parity.

## Real-image gate

After green CI, run the same source classes that exposed the short-stroke problem:

- chair;
- mug;
- plant;
- portrait optionally after nonportrait inspection.

For P4-A.1, **do not judge the overlay alone**. First inspect `p4a1-paths.png`.

Desired:
- chair legs/back/slats traced by long coherent lines;
- mug rim/body/handle represented by continuous curved gestures;
- plant pot/leaf boundaries represented by longer curves;
- sparse path layer with obvious structural hierarchy.

Failure:
- paths fragment back into many tiny pieces;
- long paths wander off source edges;
- only outer silhouette is traced while meaningful internal structure disappears;
- paths bridge unsupported white space.

## What P4-A.1 does not solve

- medium form strokes;
- reducing horizontal hatch density;
- residual tone;
- pressure variation;
- Bézier fitting;
- semantic feature recognition.

Those belong to later P4 slices.

## Exit decision

If paths-only previews look structurally meaningful and CI invariants hold, proceed to **P4-A.2 medium form paths**.

If path tracing itself is poor, improve tracing/support before touching tonal hatching.


## First CI result

On the deterministic 64x64 vertical step fixture:

- seed candidates: **22**
- accepted logical gesture paths: **2**
- frozen baseline segments: **193**
- total gesture-path length: **97.00px**
- mean gesture-path length: **48.50px**
- max gesture-path length: **49.00px**
- paths-only legacy segments: **0**
- overlay and paths-only logical path lists: exact match

This clears the engineering gate. The generated marks are materially longer than P2's 2-10.5px hatch vocabulary. It does not yet prove that real-object paths are useful; chair/mug/plant path-layer inspection is next.


## Multi-image real-pack harness

`run-p4a1-pack.ps1` now runs 3–5 permission-cleared nonportrait sources through the exact same three-view comparison and produces aggregate JSON/CSV plus a local three-column `review.html`.

The review intentionally emphasizes **paths-only first**. Overlay metrics are secondary because old horizontal hatching remains frozen until P4-A.3. See [P4-A.1 real-pack runbook](P4A1_RUN.md).
