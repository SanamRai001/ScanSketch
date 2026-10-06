# P4-A.2 — Medium Form Path Verification

**Status:** implementation branch `feat/p4a2-medium-form-paths`. This phase appends a medium-scale `Form` layer to the exact accepted P4-A.1 `Gesture` layer. P2-B.1 segments remain available only as a frozen overlay reference.

## Changed variable

Relative to P4-A.1:

```text
unchanged:
  P4-A.1 Gesture paths

added:
  8-32px Form paths
```

No long-path threshold or geometry is tuned in this phase.

## Medium proposal policy

Form paths use:
- denser 5px seed cells;
- source edge threshold below Gesture threshold;
- multiscale tangent direction;
- generic interior-support ranking bonus;
- explicit suppression around accepted Gesture corridors;
- forward/backward tangent integration;
- local edge snapping;
- path-wide edge support;
- 8-32px accepted length;
- smaller duplicate-suppression radius;
- thinner/lighter ink than Gesture paths.

No object semantics are used.

## Inspection modes

`--stroke-hierarchy`
- exact frozen P2-B.1 straight segments;
- exact P4-A.1 Gesture paths;
- new P4-A.2 Form paths.

`--stroke-hierarchy-only`
- exact same Gesture + Form paths;
- zero legacy straight segments.

## Paired proof script

`scripts/run-p4a2-ab.ps1` generates:

- `p2b1.png`;
- `p4a1-paths.png`;
- `p4a2-paths.png`;
- `p4a2-overlay.png`;
- stroke/path JSON and measurement reports;
- `summary.json`.

The script asserts:

- P2-B.1 segments are exact in P4-A.2 overlay;
- P4-A.1 path list equals the Gesture subset of P4-A.2 exactly;
- P4-A.2 overlay paths equal P4-A.2 paths-only exactly;
- paths-only contains no legacy segments;
- role counts match reported stats;
- every Form path measures between 8 and 32px.

## Core tests

Rust tests additionally require:

- blank source produces no Gesture or Form paths;
- a large object with a short internal contrast feature produces Form paths;
- all Form paths stay in the medium band;
- P4-A.1 Gestures are exact after adding P4-A.2;
- hierarchy generation is deterministic;
- paths-only contains no old segments.

## Synthetic CLI gate

The rights-clear `form-detail.png` fixture contains:
- a large mid-gray object boundary suitable for Gesture paths;
- a short darker internal feature deliberately shorter than the P4-A.1 minimum.

CI must show:
- exact P4-A.1 Gesture preservation;
- nonzero Form count;
- mean/max Form length in the intended band;
- exact baseline and path-only invariants.

## Real visual gate

Reuse chair, mug and plant.

Primary comparison:

```text
P4-A.1 paths only  vs  P4-A.2 paths only
```

We want new medium marks to add useful internal structure:

### Chair
- seat plane;
- slats;
- shorter braces / leg transitions;
- back/seat junctions.

### Mug
- rim interior;
- body transitions;
- handle interior;
- shorter base/body cues.

### Plant
- secondary leaf boundaries;
- leaf veins where supported;
- stem details;
- pot planes/rim details.

Reject if:
- Form paths mostly duplicate Gesture silhouettes;
- count explodes into a new dash field;
- medium paths wander off supported structure;
- P4-A.1 Gestures change.

## Phase boundary

P4-A.2 still leaves the old P2-B.1 hatch field untouched in overlay mode.

If Gesture + Form paths are useful, proceed to **P4-A.3 residual hatching rebalance**, where the old full-field horizontal tone is finally reduced and regenerated only where structural paths do not already carry the drawing.


## Multi-image real-pack harness

`run-p4a2-pack.ps1` now reuses 3-5 nonportrait sources and creates a focused three-column review: P4-A.1 Gestures, P4-A.2 Gestures+Forms, and P4-A.2 overlay. It aggregates role counts, Form length/share, and paths-only edge change. See [P4-A.2 runbook](P4A2_RUN.md).


## Pack-harness CI verification

The multi-image hierarchy validator passed GitHub Actions [run 37486009965](https://github.com/SanamRai001/ScanSketch/actions/runs/37486009965), including JSON/CSV/HTML output and zero-Form cases.

Synthetic pack behavior:

- `form-detail`: **4 Gestures + 1 Form**;
- `step`: **2 Gestures + 0 Forms**;
- `gradient`: **0 Gestures + 0 Forms**.

Pack summary: Forms on 1/3 sources, paths-only edge-F1 win on 1/3, overlay edge-F1 win on 2/3, white non-worse on 1/3. These synthetic values are not a quality benchmark. The important evidence is selective behavior: the medium layer is present on the dedicated internal-detail source and absent where the source does not justify it.


## First real-pack outcome

Chair/mug/plant all pass the medium hierarchy gate. Each source retains its exact P4-A.1 Gestures and adds six Forms. Gesture means are **54.76 / 52.47 / 45.39px**; Form means **12.24 / 16.06 / 13.23px**. Form path-length share stays **25.11–31.46%**. Paths-only edge F1 improves **31.596% / 30.538% / 23.917%** over P4-A.1. Visual review confirms the medium layer is subordinate and useful rather than a new dash field. See [P4-A.2 first real result](P4A2_FIRST_REAL_RESULT.md).

**Decision:** freeze Gesture+Form geometry and proceed to P4-A.3 residual hatching. The old full P2-B.1 overlay is not the final composition; white RMSE was non-worse on 0/3 overlay cases.
