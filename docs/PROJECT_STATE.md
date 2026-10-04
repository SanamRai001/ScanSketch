# Project State

Last updated: **2026-10-04**

This is the **single authoritative progress record**. Update it at the end of each implementation or documentation phase; the actual Git tree and verification results take precedence over any prose.

## Goal

Develop ScanSketch, an image-to-sketch reconstruction engine that uses top-to-bottom guided scanline sampling, intelligent pencil-stroke placement, and optional Primitive-inspired optimization, while retaining a visibly hand-drawn line character.

## Repository and branch

- Repository: https://github.com/SanamRai001/ScanSketch
- Default branch: main
- Documentation branch: docs/foundation
- Source at inspection: empty public repository, no commits/branches, no application.
- Bootstrap action: initial minimal README on main, commit 0e2c9100e36a6f5021640867a70a52434ea00cb0, solely to establish a Git history from which to create the working branch.
- No merge, deployment, or deletion authorized/performed.

## Current phase: P0 — Documentation foundation

### Intended changes on docs/foundation

- Founding README and attribution.
- MIT LICENSE with Copyright (c) 2026 Sanam Rai.
- EditorConfig and conservative Git ignore patterns.
- Contributing guidance.
- Vision, proposed algorithm, preliminary architecture, phased roadmap, research references, benchmark protocol, and separately parked future experiments.

### Implementation status

- Scanline renderer: not started.
- Edge detector: not started.
- Stroke optimization: not started.
- Erasure/residual graphite: explicitly deferred.
- UI, export, CI and deployment: not started.

## Verification

- Repository metadata, emptiness and permissions inspected using the connected GitHub repository.
- Initial Git bootstrap commit returned by GitHub; foundation branch created.
- Documentation contents require PR review; **no source code or algorithm tests can pass yet because no engine exists**.
- Do not treat illustrative pseudocode, benchmarks or acceptance thresholds as empirical results.

## Key decisions

1. Pencil strokes, not filled primitive shapes or grayscale pixel repainting, are the intended output.
2. Top-to-bottom scanline traversal is the core identity; refinements may revisit regions afterward.
3. Begin with a deterministic baseline to prove improvements attributable to optimization.
4. Use inspired principles from Primitive, not a blind port; preserve upstream notices if implementation code is ever reused.
5. Start on white paper, minimize unwanted marks in highlights, and control complexity.
6. MIT is selected for reuse and contribution while preserving Sanam Rai's copyright notice.
7. The erasure/residual-graphite concept is **recorded only, not scheduled**.

## Open risks

- A dense scanline approach could resemble engraving/halftone rather than a drawn sketch.
- Pure tonal scoring could obscure important contours or overfill light areas.
- Greedy adding cannot undo an over-dark accepted stroke; guard acceptance and cap pressure until an actual edit/correction model is investigated.
- Partial scoring is exact only for appropriately local/additive objectives; nonlocal features need padded regions or full re-evaluation.
- Stroke count and candidate search cost could exceed browser memory/performance budgets.
- SVG path count and compositing behavior may differ from raster previews.
- Licensing of third-party reference images/test fixtures must be verified.
- All runtime, image quality and performance claims remain unverified.

## Next phase: P1 — Deterministic baseline

Implement only a small, local image-to-strokes proof: controlled image input, luminance/darkness analysis, white-paper canvas, top-to-bottom band scanning, reproducible short mostly-horizontal strokes, raster preview, and output/stroke capture for tests. Establish a baseline fixture set and metrics before optimizing.

**P1 exit condition:** a user can reproduce a recognizable line-based rendering from the same fixture and seed without candidate hill climbing, AI, erasing, hosted services or a complex frontend. See docs/ROADMAP.md and docs/EXPERIMENTS.md.
