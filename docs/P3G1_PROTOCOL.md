# P3-G1 — Nonportrait Generalization Protocol

**Goal:** determine whether frozen P3-A.2.3 improves underrepresented structure beyond the private portrait, without further renderer tuning.

## Frozen renderer

Do not change P3-A.2.3 while collecting this pack.

Frozen:
- P2-B.1 baseline;
- P3-A.2.3 deficit-driven candidate proposals;
- seed 42;
- max-side 512 for real photographs;
- exact tonal-prefix / total-count invariants;
- 40% replacement cap;
- 15% + 0.001 replacement margin;
- candidate geometry, tensor thresholds, white/source-support checks and tile fairness.

## Required source pack

Use **3–5 permission-cleared genuine nonportrait photographs** in one dedicated local folder.

Recommended coverage:
1. manufactured object with obvious silhouette + interior detail (mug, bottle, tool);
2. footwear/bag/object with curves, seams or layered structure;
3. architecture/furniture/room detail with straight edges;
4. optional natural object/plant;
5. optional lighter object where important structure is not very dark.

Avoid:
- portraits/faces for this phase;
- synthetic fixtures as quality evidence;
- several nearly identical views of the same object;
- images you cannot legally process.

The harness requires explicit operator confirmation for rights and nonportrait status.

## Comparison

For every image:
- same source;
- same binary/commit;
- same seed/working-size policy;
- P2-B.1 vs P3-A.2.3;
- exact stroke-count parity;
- exact tonal-prefix parity;
- exact changed-slot/replacement accounting;
- p2c-v1 metrics;
- P3-A.2.3 structural stats.

No output is accepted if frozen invariants fail.

## Automatic evidence

The batch harness writes:
- per-image P2-B.1 / P3-A.2.3 PNGs;
- per-image stroke JSON and measurement reports;
- per-image summary JSON;
- aggregate `summary.json`;
- aggregate `summary.csv`;
- local `review.html` with A/B images side-by-side.

Aggregate means are **exploratory only**. The small 3–5 image pack is not a statistical benchmark.

## Visual review

For each case inspect:
- Does P3-A.2.3 improve meaningful interior/object boundaries?
- Are differences only on the easiest silhouette?
- Does object recognizability improve?
- Does tonal mass remain stable?
- Does white paper remain clean?
- Are new lines coherent with source structure rather than decorative?

Record preference as:
- P2-B.1 better;
- tie / imperceptible;
- P3-A.2.3 better.

Do not choose based only on edge F1.

## Provisional decision rule

P3-A.2.3 earns continued promotion only if:
- frozen invariants hold on all sources;
- no source shows serious visual/white-space regression;
- structural/visual improvement appears on **multiple distinct nonportrait source classes**, not only one image;
- metric behavior is broadly compatible with the visual finding.

If results are mixed, preserve them and investigate which source class fails before changing the renderer.

If results are consistently neutral/negative, return to the prior evidence—especially P3-A.2.1—rather than tuning P3-A.2.3 against one case.

## Privacy

Source photographs, generated previews and local HTML remain local unless the user explicitly chooses to publish them. Repository documentation records metrics/decisions, not private images.
