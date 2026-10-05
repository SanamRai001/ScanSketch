# P4-A — Human Stroke Hierarchy Architecture

**Status:** architecture phase opened after P3-G1. No renderer behavior is changed by this document. P2-B.1 and all P3 variants remain preserved as experimental evidence.

## Why P4 exists

P3-G1 exposed a mismatch between ScanSketch's stated visual goal and its actual primitive language.

The renderer is made from editable strokes, but almost all visible marks are still tiny local fragments:

- P2 tonal fragments: **2.0–10.5 px** at the working image scale.
- P3 structural candidates: **2.6–7.2 px**.
- P2-B.1 contour accents: short local edge marks of similar scale.

At a 512px working image, these marks are too short to behave like the confident gestures a human commonly uses to establish silhouette, form and major internal structure.

The chair, mug, plant and portrait experiments all show the same visual signature:

> recognizable source + horizontal dash field + repaired contours.

That is a useful computational sketch representation, but it is not yet a convincing ordinary human pencil sketch.

## Core hypothesis

A human-like sketch needs a **hierarchy of stroke roles and scales**.

P4 should stop treating all marks as variations of the same short line primitive.

The target hierarchy is:

### 1. Long structural / gestural paths

Purpose:
- major silhouette;
- long object boundaries;
- strong folds/seams;
- long internal structural edges;
- gesture / dominant directional flow.

Properties:
- continuous logical stroke;
- may curve;
- should span a meaningful fraction of the object;
- sparse;
- higher perceptual importance than hatching;
- allowed to cross old scan bands;
- deterministic and editable.

Initial working-scale target for a 512px image:
- typical path length roughly **24–96 px**;
- longer only when source support remains coherent;
- values are research bounds, not a final artistic rule.

### 2. Medium form-following strokes

Purpose:
- describe planes and volume;
- connect structural regions;
- folds, seams, rims, leaf veins, chair slats, handle/body flow;
- local form direction.

Initial target:
- roughly **8–32 px**;
- direction follows source structure rather than default horizontal scan;
- fewer marks than the current tonal field.

### 3. Short hatching / texture strokes

Purpose:
- local tone;
- texture;
- dark mass refinement;
- small residual errors.

Initial target:
- existing **2–10.5 px** vocabulary remains useful;
- but it becomes the *supporting* layer rather than the dominant visual language.

## Critical principle: long strokes must be first-class

Do **not** fake one human stroke by storing 20 unrelated straight `Stroke` records that merely touch.

The project goal includes editable deterministic marks. A logical curved gesture therefore needs its own path record.

Proposed representation:

```rust
enum Mark {
    Segment(SegmentStroke),
    Path(PathStroke),
}

struct PathStroke {
    points: Vec<Point>,
    width: f32,
    opacity: f32,
    role: StrokeRole,
}

enum StrokeRole {
    Gesture,
    Form,
    Hatch,
    Accent,
}
```

Exact API is not frozen yet. A compact polyline is the preferred first implementation because it is:
- deterministic;
- easy to render with tiny-skia;
- easy to serialize;
- easy to measure;
- easier to debug than immediately introducing cubic Bézier fitting.

Bezier compression may come later after path tracing is proven.

## Rendering

A `PathStroke` should render as one continuous pencil mark:
- one tiny-skia path;
- round cap/join behavior;
- one logical record in JSON;
- one path-length measurement;
- one role label.

For the first prototype, width/opacity may remain constant per path. Pressure variation can come later.

## Long-path proposal source

Long strokes should be traced from **coherent source structure**, not simply extended from the current horizontal tonal fragments.

Suggested first algorithm:

1. compute smoothed source edge magnitude;
2. compute / reuse multiscale tangent field;
3. find spatially separated high-confidence seeds;
4. integrate forward and backward along the tangent field;
5. stop when:
   - edge support falls below threshold;
   - tangent coherence collapses;
   - curvature exceeds the allowed turn;
   - the path reaches image bounds;
   - the path approaches an already-covered structural corridor;
   - maximum length is reached;
6. simplify the traced point chain conservatively;
7. validate the whole path against source edge support;
8. rank by length × support × coherence × missing-structure value.

A midpoint/RK2-style integration step is preferable to naive pixel hopping because it produces smoother direction changes without expensive global optimization.

## Boundary support must change

The existing short-stroke support checks often require darkness under the whole mark. That is correct for hatching but wrong for a silhouette line: a genuine boundary often has dark/object pixels on one side and white/background on the other.

Long structural paths therefore need **edge-corridor support**, not full-dark-footprint support.

A structural path may be accepted when:
- its center follows strong source-edge evidence;
- local edge normal separates meaningfully different source values;
- tangent direction is coherent;
- the line does not wander into unsupported blank paper.

This is an important architectural distinction between:
- **structure support**, and
- **tone support**.

## Ordering and ScanSketch identity

Top-to-bottom scanning remains part of ScanSketch's identity, but it should not force every physical stroke to be horizontal or tiny.

P4 can preserve deterministic ordering by recording:
1. long gesture paths sorted by their topmost / seed Y;
2. medium form paths in deterministic spatial order;
3. short residual hatching in the existing top-to-bottom scan order.

Thus the output remains replayable and deterministically ordered without sacrificing natural stroke geometry.

## Tone strategy

Do not simply add long paths on top of all ~12k current tonal fragments. That would make the drawing busier, not more human.

P4 should progressively make the short tonal layer **sparser** where long/medium structure already carries visual information.

Initial strategy:
- generate long structural layer first;
- generate medium form layer second;
- render those layers;
- compute residual tone;
- generate short hatching only for residual tonal needs.

This reverses the current visual priority:
- old: tone field first, tiny structure repair second;
- P4 target: structure first, form second, residual tone last.

## Phased implementation

### P4-A.0 — Path primitive foundation

Engineering only:
- introduce logical path stroke representation;
- JSON serialization;
- tiny-skia rendering;
- path-length metrics;
- deterministic equality/roundtrip tests;
- no automatic long-path generation yet.

Acceptance:
- existing straight-stroke modes remain byte/replay compatible;
- one synthetic curved path renders continuously;
- metrics count a path as one logical mark and measure its geometry correctly.

### P4-A.1 — Long structural path tracing

Add opt-in source-driven long paths:
- edge/tangent seed selection;
- forward/backward streamline tracing;
- curvature and support stopping;
- spatial suppression;
- long-path stats.

Acceptance:
- long paths are genuinely long relative to P2/P3 marks;
- curved synthetic boundaries are followed continuously;
- no unsupported bridge across blank space;
- chair/mug/plant contours visibly use continuous gestures.

Do **not** solve tone yet.

### P4-A.2 — Medium form paths

Add medium-scale interior strokes:
- same structural field;
- shorter trace length;
- lower priority than gesture paths;
- spatially distributed through useful internal structure.

Acceptance:
- object interiors gain directional/form strokes;
- not merely additional silhouette ink.

### P4-A.3 — Residual hatching rebalance

Reduce reliance on full-field horizontal fragments:
- render gesture + form first;
- measure remaining tonal residual;
- place short hatch marks only where needed;
- maintain white-space safety.

Acceptance:
- total short-hatch count drops materially;
- object remains tonally readable;
- sketch no longer visually reads as a horizontal raster.

### P4-G — Human-likeness validation

Use portrait + chair + mug + plant + additional objects.

Metrics must include conventional reconstruction measures **and new stroke-language measures**:
- logical mark count by role;
- path-length distribution (P10/P50/P90);
- fraction of path length from gesture/form/hatch;
- orientation entropy / horizontal dominance;
- average curvature for path strokes;
- visual preference review.

The primary quality question becomes:

> Does this look like a person constructed the drawing with purposeful strokes?

Not merely:

> Did edge F1 increase?

## What P4 deliberately does not do yet

- semantic face detection;
- neural style transfer;
- diffusion/image generation;
- paper texture;
- graphite smudge simulation;
- eraser simulation;
- pressure-varying brushes;
- expensive Primitive-style render-search;
- attempt to imitate a particular artist.

Those can be considered only after the stroke hierarchy itself works.

## Research expectation

P4 is a larger architectural change than P3-A.2.x.

That is intentional.

P3 showed that increasingly intelligent selection over tiny primitives eventually hits a visual ceiling. The next credible experiment is to change the **representation and hierarchy of marks**, because that is what the chair/mug/plant evidence says is missing.
