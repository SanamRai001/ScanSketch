# P4-A.2 — Medium Form-Following Path Architecture

**Status:** architecture frozen after P4-A.1 succeeded on chair, mug and plant. P4-A.1 Gesture paths remain unchanged; P4-A.2 adds a second logical path scale rather than increasing gesture density.

## Why this phase exists

P4-A.1 answered the first stroke-language question positively:

- chair: 4 long Gesture paths, mean 54.76px, max 79.95px;
- mug: 4 paths, mean 52.47px, max 56.24px;
- plant: 5 paths, mean 45.39px, max 52.66px.

The paths-only images clearly contain meaningful continuous object structure.

They are also deliberately sparse.

What is missing now is neither:
- another long silhouette tracer; nor
- the old 2-10px hatch field.

The gap is **medium form structure**:
- chair slats, seat plane, braces and shorter leg/back segments;
- mug rim/body transitions and handle interior;
- leaf veins, secondary leaf boundaries, stem/pot planes;
- comparable 8-32px structural cues in arbitrary objects.

## Frozen P4-A.1 layer

P4-A.2 must generate the **exact same long Gesture paths** as P4-A.1 for the same source/options.

No P4-A.1 thresholds, path lengths, seed logic or accepted-path geometry are tuned in this phase.

This gives a clean causal comparison:

```text
P4-A.1 = Gesture paths
P4-A.2 = exact same Gesture paths + new Form paths
```

## Medium path role

New paths use:

```text
StrokeRole::Form
```

Initial working-scale target:

- minimum path length: about **8px**;
- maximum path length: about **32px**;
- typical expected length: roughly 12-26px;
- logically one editable polyline per mark.

These are research bounds.

## Proposal source

P4-A.2 reuses the same image-derived ingredients:

- source darkness map;
- continuous source edge-strength map;
- multiscale tangent field.

But medium seeds are denser and slightly more permissive than Gesture seeds:

- smaller seed cells;
- lower edge threshold;
- same deterministic spatial ordering;
- same tangent-coherence requirement.

## Interior-form preference

A major risk is simply filling the image with shorter pieces of the outer silhouette.

To counter that without semantics, medium seed ranking includes a generic **interior-support bonus**.

At an edge seed:
1. obtain tangent direction;
2. compute its normal;
3. sample source darkness on both sides of the edge;
4. reward seeds where both sides retain nontrivial source content.

Intuition:

- silhouette edge: object on one side, white/background on the other -> lower interior bonus;
- seam/rim/slat/vein/form transition: source content often exists on both sides -> higher interior bonus.

This is a ranking preference, not a hard semantic rule. A useful boundary can still become a medium path if evidence is strong.

## Gesture-corridor suppression

Before medium proposal:

1. generate the exact P4-A.1 Gesture layer;
2. rasterize a conservative coverage corridor around those Gesture path points;
3. suppress medium seeds inside that corridor.

During medium trace acceptance:
- allow small crossings/intersections;
- reject paths that spend too much of their length inside the existing Gesture corridor.

This prevents P4-A.2 from spending its budget on duplicates while allowing natural structural junctions.

## Medium tracing

Medium tracing follows the same conceptual rule as gestures:

- integrate forward/backward along source tangent;
- snap to nearby source-edge support;
- stop on low support, sharp turn, loop, bounds or length limit;
- validate the whole path's edge corridor;
- accept only 8-32px paths;
- spatially suppress nearby accepted medium paths.

Differences from Gesture tracing:
- shorter maximum integration distance;
- denser seed grid;
- slightly lower edge threshold;
- slightly looser turn/support bounds where needed for small curved details;
- smaller duplicate-suppression radius;
- Form role;
- thinner/lighter default ink than Gesture paths.

## Ordering

Path order remains deterministic:

1. all accepted Gesture paths in the exact P4-A.1 order;
2. all accepted Form paths sorted deterministically by topmost Y then X.

The old straight-segment layer remains untouched in overlay mode.

## Modes

P4-A.2 should expose:

### Hierarchy overlay
Frozen P2-B.1 segments + exact P4-A.1 Gesture paths + P4-A.2 Form paths.

### Hierarchy paths-only
Exact Gesture + Form paths, no legacy horizontal segments.

For visual research, paths-only remains the primary artifact until P4-A.3.

## Statistics

Record independently:

### Gesture
- count;
- total / mean / max length.

### Form
- seed candidates;
- accepted count;
- rejected due to Gesture coverage;
- total / mean / max length.

### Hierarchy
- Gesture count;
- Form count;
- combined logical paths;
- length share by role.

This will let P4-G later measure stroke hierarchy rather than only reconstruction error.

## Synthetic gates

P4-A.2 must prove:

1. P4-A.1 Gesture paths are bit/value-identical before and after P4-A.2;
2. a source with a long outer boundary plus a shorter internal contrast feature produces at least one Form path;
3. every Form path lies in the 8-32px band;
4. Form paths do not substantially overlap Gesture corridors;
5. paths-only contains zero legacy segments;
6. generation is deterministic;
7. blank input produces neither role.

## Real gate

Reuse chair / mug / plant.

Acceptance is visual, not merely metric:

- Gesture paths remain exactly as before;
- chair gains useful seat/slat/brace/internal cues;
- mug gains useful rim/body/handle-interior cues;
- plant gains useful veins/secondary leaves/pot structure;
- Form paths do not simply trace duplicate silhouette pieces;
- no explosion in path count.

If medium paths are useful but incomplete, proceed to P4-A.3.

If they mostly duplicate silhouettes or become noisy, refine medium proposal/suppression before touching hatching.

## What remains for P4-A.3

P4-A.2 still does **not** reduce the old scanline layer.

Only after Gesture + Form paths are useful should P4-A.3 reverse visual priority:

```text
Gesture -> Form -> residual tonal hatch
```

rather than:

```text
full horizontal tone -> contour repair
```
