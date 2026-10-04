# Phased Research and Build Roadmap

Phases are gates, not calendar promises. Advance only after reviewing actual code, repo state, baseline results and open risks. Update the single docs/PROJECT_STATE.md after each phase.

## P0 — Foundation [committed on docs/foundation; PR #1 awaits review]

Deliver: README, copyright/license, project vision, algorithm specification, architecture boundaries, research citations, reproducible experiment protocol, future-idea parking lot and contribution guidance.

Exit: docs are internally consistent, README links work, no unimplemented feature is described as existing, and the foundation branch has been reviewed before merging.

## P1 — Deterministic original scanline baseline [11/11 tests, CLI compiled; visual stripe problem identified]

Deliver: bounded local image input, grayscale/brightness analysis, clean white paper, row-by-row mostly horizontal short-stroke generation, seed reproducibility, raster preview and ordered stroke records. Include zero-mark white-image fixture.

Exit: run the commands in P1_VERIFY.md and review representative source/output pairs. Same seed+input must produce matching ordered strokes; test outputs must visibly comprise strokes, not gray pixel painting; dimensions/time/memory must remain bounded. Native compilation and 11/11 core tests passed on Windows; the first portrait was mechanically striped, which motivates P2-A.

## P2-A — Broken-stroke language [14/14 tests passed in CI; portrait visually compared]

Hypothesis from the first real portrait: continuous horizontal bars create mechanical engraving even when P1 correctness tests pass.

Deliver: seeded 2–10.5 px broken tonal marks, controlled gaps, small endpoint angles, bounded vertical jitter and staggered second-layer marks in deep shadow. Keep row-by-row generation and source-white boundary protection. Retain P1 as a separate comparison branch.

Exit: new and old render of the **same** licensed portrait, same seed/max size, plus new regression tests and full-workspace compilation. Do not claim that more fragments automatically make a better picture.

## P2-B — Edge / contour reinforcement [CI passed: 20/20 tests; visual comparison pending]

Deliver: normalized linear-darkness Sobel gradient, directional non-maximum suppression, short tangent-aligned marks anchored on the dark side with support checks, sparse acceptance and bounded extra stroke budget. `--no-contours` runs the identical P2-A tone pass as a control. No semantic face recognition. Compare toggled output against P2-A before judging any visual improvement.

Exit: glasses/eyes/lips and hair contours are more recognizable without harsh synthetic outlines or dirty highlights.

## P2-C — Measurements and controlled alternatives [not started]

Deliver: fixture provenance, side-by-side original/preview, tone error, highlight-ink coverage, edge retention proxy, stroke count/path length and runtime/memory observations. Compare uniform and region-adaptive sampling with the same stroke budget.

Exit: saved metrics and representative visual comparisons justify moving to candidate optimization.

## P3 — Primitive-inspired local optimization

Deliver: candidate proposal, local raster mask, appropriate score and white-region penalty, analytic initial opacity where valid, bounded random restarts/hill climbing, deterministic seed, and exact incremental scoring for additive terms.

Exit: against P1 on the same fixtures and budgets, document wins/losses in fidelity, artistic preference, runtime, memory and number of strokes. Keep P1 as a baseline mode. No erasing.

## P4 — Structure and bounded additive refinement

Deliver: opt-in edge-aware proposal/score, optional short curved strokes, second pass concentrated on residual errors and complexity budgets.

Exit: demonstrate whether structural additions improve recognizability without creating mechanical contours or dirty highlights. Remove features that do not help. No eraser effects.

## P5 — Minimal product surface and export

Deliver: intuitive accessible upload/render/compare workflow, cancel/progress, reproducible presets or explicit seed, raster PNG export, SVG export of retained paths after parity verification, clear limits and failure handling.

Exit: local browser usage is practical and safe on modest hardware; output examples are accurately labelled; no backend/deployment needed unless new evidence changes the decision.

## Deferred, **not** part of P1–P5

Natural erasure and faint graphite residue; deliberately generated mistakes; paper textures/smudging; alternate full-angle hatching; generative enhancement/AI; account systems/cloud projects; bulk processing or SaaS packaging. See FUTURE_EXPERIMENTS.md. Revisiting requires both a satisfying core renderer and a distinct measured hypothesis.
