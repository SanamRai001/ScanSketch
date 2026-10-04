# Project State

Last updated: **2026-10-04**. This is the single authoritative progress record. The actual code, tests and visual comparisons take precedence over phase labels.

## Goal

Create a deterministic, actual-stroke-based image-to-pencil-sketch engine built around top-to-bottom scanning; investigate Primitive-style scoring only when the drawing vocabulary becomes expressive enough. A halftone or uniformly striped rendition is not the target.

## Branches / review stack

- Repository: https://github.com/SanamRai001/ScanSketch
- `main`: initial bootstrap only; no features merged.
- `docs/foundation`: foundation documentation; draft PR #1.
- `feat/p1-rust-scanline-baseline`: P1 engine; draft PR #2 into the foundation branch, tip at P2-A branch-off: `64a935366edfeb5424637e0496f1ef25986d1c5f`.
- **Active:** `feat/p2-sketch-stroke-language`: P2-A, branched from P1. Keep P1 intact as a comparison baseline. P2-A PR targets P1.

## Verified P1 outcome (user's Windows machine)

- `cargo check --workspace` passed; `cargo test -p scansketch-core` passed **11/11** on 2026-10-04.
- Native CLI compiled; the supplied portrait generated an output using actual stroke records.
- Visual review: recognizable silhouette and blank background, but the face lost detail, hair turned into a gray mass and the regular horizontal bars resembled mechanical engraving rather than a compelling sketch.
- Dependency resolution generated a local `Cargo.lock`. It has **not been supplied/committed to GitHub yet**; retain it locally for reproducible builds and commit separately.
- P1 is a valuable functional baseline, not an accepted final visual style.

## Current phase — P2-A: Broken stroke language

Implementation on the active feature branch:
- Retains column-level splitting so a bright internal column cannot be crossed by a tonal stroke.
- Changes the default sampling segment width from 8 to 24 pixels to permit multiple marks within a region.
- Replaces continuous runs with bounded 2–10.5px fragments, seeded spacing and independent stroke pressure/width variation.
- Adds small per-fragment endpoint angle and vertical wobble, clamped to each scan band.
- Uses an independently staggered second fragment layer only for dark regions in sufficiently tall bands.
- Preserves renderer-independent `Stroke` records, the existing PNG/JSON CLI, input bounds and stroke budget.
- Adds tests for short fragments, visible gaps, angle variation and white-gap protection on top of the P1 tests.
- Adds GitHub Actions check/test workflow as a remote compilation gate.

**Verification:** P2-A compilation/tests and a real-photo comparison remain pending until the new branch workflow and local run are observed. No claim of aesthetic improvement without seeing the new output.

## Next gate

Follow `docs/P2A_VERIFY.md`, compare the same portrait and seed against the saved P1 output, inspect the white gap, face recognition, hair, tone density and visible strokes at 100% zoom. Tune fragmentation only based on the comparison. If P2-A produces less recognizable shading, preserve P1 and investigate the metrics before extending it.

## After P2-A

- **P2-B:** source-derived edge/contour reinforcement (glasses, eyes, nose/lips, hair outline), bounded and evaluated separately.
- **P2-C:** measurement protocol, fixtures, stroke count/path length, tone error, highlight protection and edge retention.
- **P3:** Primitive-inspired local proposal/optimization on a capable stroke vocabulary.
- **Deferred:** erasure and residual graphite, smudge, paper effects, AI, full browser UI and uncontrolled extra passes. See `docs/FUTURE_EXPERIMENTS.md`.

## Risks

Mechanical stripes can persist because scan bands are still horizontal; fragment gaps can weaken continuous tone. `max_strokes` can be reached sooner as fragments multiply. A near-white region's small dark features can be missed by column averaging; P2-B should be judged against real fixtures, not assumptions. Coordinate clamping prevents nominal spill across bright columns but rendered antialiasing must still be visually inspected.
