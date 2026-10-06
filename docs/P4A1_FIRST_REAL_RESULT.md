# P4-A.1 — First Real Long-Gesture Pack Result

**Date:** 2026-10-06. **Status:** chair/mug/plant paths-only review complete. **P4-A.1 succeeds as a structural-stroke capability phase, but the gesture layer is intentionally sparse and not yet a complete sketch.**

Original source photographs and local generated review assets remain private/local.

## Pack

The exact nonportrait classes that exposed P3's short-stroke bottleneck were reused:

- `chair.jpg`;
- `mug.jpg`;
- `plant.jpg`.

Each source generated:
- frozen P2-B.1;
- P4-A.1 overlay;
- the exact same P4-A.1 logical gesture paths with all legacy segments removed.

## Automatic coverage

All three source classes produced long gesture paths:

| Source | Paths | Seeds | Mean length | Max length | Total gesture length |
| --- | ---: | ---: | ---: | ---: | ---: |
| chair | 4 | 176 | **54.76px** | **79.95px** | 219.05px |
| mug | 4 | 127 | **52.47px** | **56.24px** | 209.87px |
| plant | 5 | 151 | **45.39px** | **52.66px** | 226.95px |

Coverage summary:
- sources with >=1 path: **3/3**;
- sources with a >=40px path: **3/3**;
- overlay edge-F1 wins: **2/3**;
- overlay white-RMSE non-worse: **0/3**.

The white-RMSE result is expected to be treated cautiously in this phase because valid silhouette strokes deliberately ride an edge corridor and may anti-alias onto source-white pixels. P4-A.1 is not a white-background optimization phase.

## Visual review

### Chair

The paths-only layer contains:
- one long chair-back/right-side structural path;
- long lower/leg-like paths;
- a shorter seat/interior boundary cue.

These are clearly different from the old scanline vocabulary and are plausible as early construction/contour strokes. Coverage remains incomplete: the seat plane, left leg/back, slats and other medium structures are mostly absent.

### Mug

The paths-only layer is especially informative:
- paired long top/rim strokes;
- lower body/handle-side structural curves.

The marks are continuous and human-scale rather than stitched horizontal fragments. They capture important object geometry, but the mug body plane and handle interior are still under-described.

### Plant

The paths-only layer includes:
- a long curved leaf boundary;
- a long pot-side path;
- pot-rim/stem-like structural cues.

Again, the layer is sparse but recognizably follows real object structure. Additional leaves, veins, pot planes and interior form are missing.

## Main conclusion

P4-A.1 clears the question that motivated P4:

> **Can ScanSketch automatically generate long, continuous, editable, human-scale structural marks from real photographs?**

For these three distinct source classes, the answer is **yes**.

The successful paths are roughly **45–80px**, compared with the historical 2–10px hatch language.

This is the first phase where the renderer's mark vocabulary visibly changes from:
- raster-like short fragments

toward:
- sparse construction / contour gestures.

## Important limitation

P4-A.1 is **too sparse to be a finished human sketch**.

That is not a failure of the phase. Its purpose was the top level of the hierarchy.

The missing layer is now obvious:
- seat planes/slats on the chair;
- mug body/handle/rim interior;
- leaf veins/secondary leaf boundaries;
- pot planes and other interior structure.

These are too small/local for long 24–100px gestures, but too important to delegate to 2–10px hatching.

## Decision

**P4-A.1 accepted. Proceed to P4-A.2 medium form-following paths.**

Do not tune P4-A.1 to emit many more long paths simply to fill the image. That would collapse the intended hierarchy.

P4-A.2 should introduce a second path scale:

- target path length roughly **8–32px**;
- role = `form`;
- lower structural threshold than long gestures;
- use source edge/tensor coherence;
- spatially avoid duplicating long gesture corridors;
- prefer useful internal structure;
- remain first-class logical paths;
- keep long gesture layer unchanged.

Examples:
- chair slats, seat boundary, shorter leg/brace structure;
- mug rim segments, body transitions, handle interior;
- leaf veins, secondary leaf/pot boundaries.

Only after long + medium path layers are visually useful should P4-A.3 reduce the old horizontal hatch field and regenerate tone as residual support.

## Tooling note

The first multi-image CI smoke exposed a validation-only bug: a no-path source printed `total-path=-0.00`, while the PowerShell parser accepted only unsigned floats. The parser is being corrected to accept signed numeric output. This does not affect rendering or path generation.
