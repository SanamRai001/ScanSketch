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


## Multi-image harness CI result

The three-view pack harness passed [GitHub Actions run 37408490425](https://github.com/SanamRai001/ScanSketch/actions/runs/37408490425) after explicitly hardening zero-path handling under PowerShell strict mode.

Synthetic pack behavior:

- `step`: **2** gesture paths, mean **48.5px**, max **49px**;
- `thin-lines`: **4** gesture paths, mean/max **47px**;
- `gradient`: **0** paths, correctly preserved as a valid no-op rather than treated as an error;
- coverage: paths on **2/3** fixtures;
- >=40px gesture on **2/3** fixtures;
- overlay edge-F1 wins on **2/3** synthetic fixtures.

The synthetic pack is only a harness proof. The quality gate remains the real chair/mug/plant paths-only review.


## Three-view pack harness CI result

The real-pack validation harness passed [GitHub Actions run 37473050878](https://github.com/SanamRai001/ScanSketch/actions/runs/37473050878) after making zero-path numeric/stat parsing robust.

Synthetic harness coverage:

- step: **2** long paths, mean **48.50px**, max **49.00px**;
- thin-lines: **4** long paths, mean/max **47.00px**;
- gradient: **0** seeds / **0** long paths, accepted as a valid no-op;
- sources with paths: **2/3**;
- sources with a >=40px path: **2/3**;
- paths-only outputs contain **0** legacy segments;
- overlay/path-only path records remain identical per case.

The synthetic pack exists only to prove validation behavior. It is not evidence of human-sketch quality. The real chair/mug/plant pack remains the P4-A.1 exit gate.


## Three-view pack harness CI result

The multi-image validation harness passed GitHub Actions [run 37477931352](https://github.com/SanamRai001/ScanSketch/actions/runs/37477931352) after fixing zero-path numeric handling. The smoke deliberately included a no-path gradient case and treated it as valid evidence.

Synthetic pack outcome:

- step: **2 paths**, mean **48.5px**, max **49px**;
- thin-lines: **4 paths**, mean **47px**, max **47px**;
- gradient: **0 paths**, mean/max **0px**;
- sources with paths: **2/3**;
- sources with >=40px path: **2/3**;
- overlay edge-F1 wins: **2/3**.

The important harness property is that a source with no valid long structural corridor now yields a clean **zero-path result** rather than a parser failure or forced gesture.


## First real-image outcome

Chair/mug/plant all generated meaningful human-scale Gesture paths. Mean lengths were **54.76px**, **52.47px**, and **45.39px** respectively; maximums **79.95px**, **56.24px**, **52.66px**. Visual paths-only review confirms the marks follow real structure and no longer read as stitched scanline fragments. The layer is intentionally sparse and misses medium interior form. See [P4-A.1 first real result](P4A1_FIRST_REAL_RESULT.md).

**Decision:** accept P4-A.1 and move to P4-A.2 medium form paths rather than increasing long-path density.
