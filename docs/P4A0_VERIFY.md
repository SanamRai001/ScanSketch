# P4-A.0 — First-Class Path Primitive Verification

**Status:** implementation passed GitHub Actions: **69/69 Rust tests**, all historical P2/P3/P3-G1 checks and the dedicated logical-path smoke. This phase is engineering-only: automatic source tracing begins in P4-A.1. [CI run 37405989296](https://github.com/SanamRai001/ScanSketch/actions/runs/37405989296).

## Purpose

P1–P3 represented every visible mark as an independent straight `Stroke { x0, y0, x1, y1, width, opacity }`.

P4 needs one human gesture to remain one editable record even when it bends. P4-A.0 therefore introduces:

- `PathPoint`;
- `PathStroke`;
- generic `StrokeRole` values: gesture, form, hatch, accent;
- a backward-compatible `Sketch.paths` collection;
- continuous polyline rendering;
- path-specific and combined logical-mark metrics.

## Backward compatibility contract

Historical P1–P3 generators still create straight segments only.

For a stroke-only sketch:
- `paths` is empty;
- serde omits the empty `paths` field;
- old JSON without `paths` deserializes with an empty path list;
- legacy `strokes.count`, widths, opacities and path-length metrics keep their old meaning;
- all existing P2/P3 scripts can continue reading `report.strokes`.

This is intentional. P4 does not force a repository-wide JSON migration before we have proven path tracing.

## P4 metrics extension

Measurement reports add:

```text
paths:
  count
  total_path_length_px
  mean_path_length_px
  mean_points_per_path
  width/opacity statistics
  gesture/form/hatch/accent counts

marks:
  logical_count
  segment_count
  path_count
  total_path_length_px
```

A polyline with seven points and six geometric subsegments is still **one logical mark**.

## Bounds

A logical path:
- must have at least 2 points;
- is capped at 4096 points;
- must remain inside the working canvas;
- requires finite positive width;
- requires finite opacity in 0..=1.

The complete sketch remains bounded by the existing research logical-mark limit.

## Rendering

tiny-skia renders one `PathStroke` with:
- one `PathBuilder`;
- `move_to` then `line_to` through ordered points;
- round cap;
- round join;
- one width and opacity in P4-A.0.

Pressure variation and Bézier compression are deferred.

## Required tests

Rust unit tests must prove:
- polyline arc length;
- old JSON compatibility;
- empty paths omitted from old stroke-only JSON;
- invalid one-point/out-of-bounds paths rejected;
- one logical curved path renders;
- measurement counts it as one logical mark;
- invalid path JSON is rejected rather than silently scored.

## CI fixture

`scansketch-path-fixture` writes a deterministic 64×64 seven-point gesture path.

CI then verifies:
- stroke list is empty;
- exactly one path record exists;
- role is `gesture`;
- path has seven points;
- legacy straight-stroke count is zero;
- path count is one;
- combined logical mark count is one;
- measured path length is >55px.

This proves capability only. It is **not** evidence that real images now look human sketched.

## Exit gate

P4-A.0 can close when:
1. all historical tests remain green;
2. path serialization/rendering/measurement CI is green;
3. no P1–P3 experiment output behavior changes.

Then begin **P4-A.1 long structural path tracing** from source edge/tangent fields.


## First CI result

The deterministic path fixture produced:

- logical marks: **1**;
- legacy straight strokes: **0**;
- logical paths: **1**;
- path role: **gesture**;
- control/sample points: **7**;
- measured path length: **82.868557 px**;
- mean width: **1.5 px**;
- mean opacity: **0.82**.

Old P2/P3/G1 workflows remained green. This satisfies the P4-A.0 exit gate: a long curved mark can now exist, serialize, render and measure as one editable logical record without changing legacy straight-stroke semantics.
