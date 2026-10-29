# ScanSketch — Architecture, Algorithm and Product Critique

**Audit date:** 2026-10-29  
**Repository:** SanamRai001/ScanSketch  
**Inspected implementation:** feat/p4a3-residual-hatching at commit 58b385106e1b1c6c2e844b14d7bbddab83591d56 (draft PR #19).  
**Status:** REVIEW PROPOSAL, not an accepted change to the renderer.  
**Companion implementation roadmap:** [SCANSKETCH_IMPROVEMENT_PHASES_20261029.md](SCANSKETCH_IMPROVEMENT_PHASES_20261029.md)

## Review boundary and epistemic status

This critique is based on repository source inspection, existing phase documents, recorded experiments, and pull request metadata. **No new Rust binaries, local image runs, browser tests, or benchmarks were executed for this audit.** For each finding, VERIFIED means an observed source-level mechanism; HYPOTHESIS means its impact on actual images still requires testing; RECOMMENDATION is a proposed change, not a measured result.

The latest branch is a **research renderer**, not a released browser application. The default branch remains an initialization README. Do not describe a non-existent application UI as if it has been tested.

Relevant evidence:
- [PR #19 — P4-A.3](https://github.com/SanamRai001/ScanSketch/pull/19)
- [P4-A.3 verification](P4A3_VERIFY.md) and [real-image runbook](P4A3_RUN.md)
- [P4-A.2 real-image report](P4A2_FIRST_REAL_RESULT.md)
- [P3-G1 generalization report](P3G1_FIRST_RESULT.md)
- [Existing architecture document](ARCHITECTURE.md) and [project state](PROJECT_STATE.md)

## What the project actually implements

The current native-first Rust workspace has a UI-independent core and a local CLI. Images are decoded under bounded limits, resized, white-matted and converted into linear-light darkness. Long Gesture paths are traced from smoothed Sobel and multiscale structure-tensor evidence. Medium Form paths follow related evidence while avoiding established Gesture corridors. The structure is rendered on white paper, the remaining positive darkness is compressed, and short Hatch paths are generated in horizontal scan bands. A Sketch serializes straight legacy strokes and polyline paths with roles; tiny-skia renders a raster preview.

**Implemented:** deterministic seeded generators; physical, editable stroke/path records; PNG and JSON output; synthetic correctness tests; numeric tone/edge measurements; a P4 Gesture/Form/Hatch hierarchy in an opt-in mode.

**Not implemented:** global candidate-search optimization, semantic object understanding, variable-pressure drawing, native SVG export, WASM/browser UI, full-scale perceptual validation, or a released product.

### Verdict

The three-level stroke hierarchy is a worthwhile architectural correction to the earlier short-horizontal-fragment approach. Yet ScanSketch still largely assembles local image-processing heuristics. The code does **not** establish that each chosen stroke improves global perceptual reconstruction, that its allocation policy serves important parts of an object, or that the output consistently looks hand drawn to independent viewers.

This is a reason to improve experimental rigor and selection, **not** a reason to replace Rust with a large AI stack or rewrite everything.

## Findings, ordered by impact

### A01 — HIGH / VERIFIED: P4-only generation has a mandatory legacy baseline dependency

**Evidence:** [long_paths.rs lines 976–1010](../crates/scansketch-core/src/long_paths.rs#L976-L1010) generates the full P2 sketch even when include_baseline=false. [residual_hatch.rs lines 316–348](../crates/scansketch-core/src/residual_hatch.rs#L316-L348) subsequently generates a P2 control again, solely to calculate the Hatch budget.

**Cause:** Historical baseline computation also acts as validation, geometry metadata and a stroke-budget reference.

**Why this is harmful:** P4-only render pays for two P2 passes and can fail because P2 exceeds max_strokes even when the final P4 result would be within the requested logical-mark budget. The documented 64x64 form-detail fixture produced 161 P2 segments versus 4 Gesture + 1 Form + 42 Hatch paths, making this an actionable regression case, not just an efficiency preference.

**Fix:** Extract shared input/options validation; make the P4 geometry generator independent of P2; define explicit P4 budget semantics based on output marks; move optional P2 control/budget comparisons into the research harness. Freeze normal-seed P4 output before refactoring so structural geometry remains identical where intended.

**Gate:** A P4-only run with an output budget below the number of P2 legacy segments but above its own required paths completes; historical P1–P3 outputs and standard P4 paths remain stable where intended.

### A02 — HIGH / VERIFIED: Hatches are allocated first-come, top-to-bottom

**Evidence:** [residual_hatch.rs lines 228–298](../crates/scansketch-core/src/residual_hatch.rs#L228-L298) iterates scan bands and horizontal regions; generation stops when paths.len() reaches hatch_budget.

**Cause:** Scan traversal and candidate selection are fused.

**Risk (HYPOTHESIS):** Dense upper regions can consume the budget before lower, structurally meaningful areas receive tone. A stable seed only makes the bias reproducible.

**Fix:** Collect bounded candidates with source-residual support; rank by utility; add tile-level coverage constraints or quota reservations; then sort accepted marks for top-to-bottom playback. Test on synthetic top-heavy and bottom-heavy residual maps with the same total budget.

**Gate:** Coverage/utility on lower-priority-by-position but higher-value regions improves without unacceptable paper or tonal regressions; determinism is retained.

### A03 — HIGH / VERIFIED: No globally evaluated stroke utility in P4

**Evidence:** [long_paths.rs lines 104–485](../crates/scansketch-core/src/long_paths.rs#L104-L485) uses local seed ranking, tangent following, edge support and covered-radius exclusion. [residual_hatch.rs](../crates/scansketch-core/src/residual_hatch.rs) uses a compressed positive-darkness residual rather than a render-and-accept search.

**Risk:** Greedy local choices can miss occluded silhouettes, isolate useful interior contours, duplicate form or place strokes that do not improve the whole reconstruction.

**Fix:** First benchmark a small set of bounded, localized candidate-proposal and incremental render-error checks on top of existing P4 paths. Keep the baseline selectable; promote only if independent visual evaluation improves. Penalize redundancy and excess mark complexity, not only pixel error.

**Gate:** Paired tests show improvements on held-out image classes, not just the fixture used to tune the utility.

### A04 — HIGH / VERIFIED with visual impact UNPROVEN: The Hatch primitive remains mainly horizontal

**Evidence:** [residual_hatch.rs lines 151–226](../crates/scansketch-core/src/residual_hatch.rs#L151-L226) constructs small two-point paths with nearly horizontal endpoint variation and scan-band grouping.

**Why:** This was deliberately chosen to isolate density and avoid changing two variables at once. That is good experimental discipline.

**Risk:** A 70% mark-count reduction does not automatically make the remaining marks convincingly human or form-following.

**Fix:** Complete the real P4-A.3 chair/mug/plant review first. Only if orientation remains the dominant failure, compare existing horizontal Hatch to an orientation-field-aligned Hatch at matched marks, ink budget and seed. Do not tune density and angle simultaneously.

### A05 — HIGH / VERIFIED: Fixed pixel thresholds are intertwined with visual intent

**Evidence:** [long_paths.rs lines 16–40](../crates/scansketch-core/src/long_paths.rs#L16-L40) fixes thresholds, seed grid, trace lengths and suppression radii; [residual_hatch.rs lines 20–30](../crates/scansketch-core/src/residual_hatch.rs#L20-L30) fixes Hatch bands and corridor radii.

**Risk:** Behavior learned at 64px or 512px may change qualitatively at another working resolution; some constants encode visual assumptions without calibration.

**Fix:** Test 64/256/512/768 on the same permitted images, with equivalent normalization; adopt one canonical analysis scale or scale-aware geometry parameters only after measured differences. Keep explicit Gesture/Form/Hatch limits rather than hiding all heuristics in a universal 'quality' knob.

### A06 — HIGH / VERIFIED: Numeric and engineering gates do not prove hand-drawn quality

**Evidence:** [metrics.rs lines 130–237](../crates/scansketch-core/src/metrics.rs#L130-L237) evaluates source-vs-preview Sobel edge F1 with a fixed edge threshold and +/-2px tolerance. Related Sobel evidence also drives stroke generation. [P4-A.3 verification](P4A3_VERIFY.md) records 85/85 Rust tests and significant synthetic Hatch reduction, but the chair/mug/plant P4-A.3 aesthetic gate remains open.

**Risk:** Shared assumptions between proposal generation and evaluation can reward the current feature representation; low RMSE or high edge F1 is not a human preference score. Existing P4-A.2 improvements over a sparse Gesture-only baseline do not settle final sketch quality.

**Fix:** Add a blinded paired perceptual protocol, a reserved held-out image subset, multiple seeds and failure-case labeling. Keep metrics for debugging and repeatability; report them separately from human-likeness.

**Gate:** Do not claim 'convincing sketch quality' until reviewers prefer the renderer on multiple unseen subject classes.

### A07 — MEDIUM / VERIFIED: Duplicate analysis work and tight coupling

**Evidence:** [long_paths.rs lines 405–419](../crates/scansketch-core/src/long_paths.rs#L405-L419) and [lines 867–885](../crates/scansketch-core/src/long_paths.rs#L867-L885) construct darkness, edge and orientation data independently for Gesture and Form. P4 also performs repeated P2 generation as described in A01.

**Fix:** Add one internal analysis context carrying dimensions, darkness, edge strength and orientation fields; pass read-only references to structural generators. Avoid introducing many crates or a generic plugin framework.

**Gate:** Pixel/path parity in regression fixtures, measured reduction in repeated work, bounded memory consumption.

### A08 — MEDIUM / VERIFIED: The document model limits expressive drawing and playback

**Evidence:** [stroke.rs lines 17–106](../crates/scansketch-core/src/stroke.rs#L17-L106) has per-path constant width and opacity. [render.rs lines 38–90](../crates/scansketch-core/src/render.rs#L38-L90) renders legacy strokes followed by all paths; the model does not provide a unified interleaved draw order.

**Risk:** Tapered starts and ends, pressure variation and true top-to-bottom animation cannot be represented precisely. Separate arrays are backward-compatible but do not create one global schedule.

**Fix:** Propose a schema-versioned optional per-point pressure model and explicit drawing-order metadata. Preserve old JSON parsing and PNG replay first. Do not prematurely rewrite every mark into Béziers.

### A09 — MEDIUM / VERIFIED: CLI chooses history over current P4 and reports incomplete mark counts

**Evidence:** [main.rs lines 9–110](../crates/scansketch-cli/src/main.rs#L9-L110) exposes many mutually exclusive experiment flags; the default still falls back to P2-B.1. [main.rs lines 236–273](../crates/scansketch-cli/src/main.rs#L236-L273) prints sketch.strokes.len() as 'strokes', which is zero for P4 paths-only/final modes.

**Fix:** Keep stable experiments addressable but provide one named current-candidate mode, one explicit historical control mode and a clearer grouped CLI. Report segments, paths and logical marks accurately. Change defaults only after an accepted P4 quality gate.

### A10 — MEDIUM / VERIFIED: Output handling and reproducibility need tightening

**Evidence:** [main.rs lines 115–135 and 230–235](../crates/scansketch-cli/src/main.rs#L115-L135) checks the PNG input overwrite case but lacks general no-clobber handling for existing destination JSON; PNG and JSON writes are separate, not a paired transactional result. No Cargo.lock was found in the reviewed branch; [rust-toolchain.toml](../rust-toolchain.toml) selects 'stable'; CI performs check/test/smokes but no dedicated formatting, Clippy or performance-gate step.

**Fix:** Validate every destination, reject existing outputs by default, use temporary file + rename strategies and report partial-output failures clearly. Commit Cargo.lock for the application, capture exact build/version/config/source hash, and add cargo fmt / Clippy before prioritizing performance work. Keep private images local.

### A11 — MEDIUM / VERIFIED: Repository integration and current documentation are stale

**Evidence:** main is a bootstrap README. The latest P4-A.3 lives in [draft PR #19](https://github.com/SanamRai001/ScanSketch/pull/19), atop a chain of earlier work. The latest branch README and [ARCHITECTURE.md](ARCHITECTURE.md) still foreground older P2/P3 descriptions even though P4 geometry is implemented.

**Risk:** New contributors reproduce the wrong renderer; selective cherry-picks or unreviewed merges may include rejected experiments.

**Fix:** Establish an accepted integration candidate by a reviewable operation, reconcile README/ARCHITECTURE with actual code, keep a single authoritative [PROJECT_STATE.md](PROJECT_STATE.md) updated at accepted milestones, and preserve negative research results separately. Do not mass-merge the stacked history.

### A12 — PRODUCT GAP / VERIFIED: There is no browser UX to audit yet

**Evidence:** [ARCHITECTURE.md](ARCHITECTURE.md) explicitly describes a local CLI and defers WASM/React. No browser app is shipped in the reviewed implementation.

**Recommendation:** Evaluate the CLI research workflow now: discoverability, source/result comparisons, local permission handling, output no-clobber and unambiguous diagnostic states. After the visual quality gate, prototype a single-page local-first UI with upload, compare, render/cancel and export. Avoid authentication, storage, payments and GPU assumptions at this stage.

## Strengths worth preserving

- UI-independent native Rust core; local-image privacy by default.
- Deterministic ChaCha RNG with explicit seeds.
- Physical, replayable stroke/path records rather than only a filtered bitmap.
- Separate Gesture, Form, Hatch roles and specific synthetic invariants.
- Strong negative-result discipline: P3-A.0 and P3-A.1 were rejected; P3-G1 drove a real primitive-language change.
- Established paired A/B scripts and historical output preservation.
- No premature backend, database, cloud, accounts or neural network dependency.

## External implementations: adopt selectively, not wholesale

1. [Primitive by Michael Fogleman](https://github.com/fogleman/primitive) — study bounded candidate optimization, mutation, score deltas and partial redraw. It reconstructs using different geometric primitives; do not import its rendering aesthetics or blindly optimize only raster RMSE.
2. [VTracer by Visioncortex](https://github.com/visioncortex/vtracer) — benchmark its raster/vector boundaries and curve-fitting concepts against ScanSketch's existing tracing; consider a replaceable reference baseline or reusable component only if objective and visual metrics support it.
3. Continue using existing [tiny-skia](https://github.com/linebender/tiny-skia) and the Rust image crate rather than writing a second raster engine.

Check active upstream license versions, compatibility, build targets and maintenance before copying/vendoring; prefer APIs or comparative experiments first. Neither Primitive nor VTracer directly solves human pencil-stroke sequencing and pressure.

## Stop / start / postpone

**Stop:** treating higher edge F1 or lower Hatch count as final visual success; introducing another permanent heuristic before assessing P4-A.3 real outputs; treating P2 as mandatory in every P4 pass.

**Start:** freeze a versioned baseline, repair P4 budget/legacy coupling, evaluate with blinded comparisons and held-out images, benchmark a bounded global Hatch selection experiment.

**Postpone:** semantic AI, smudging/eraser simulation, paper texture, database/backend, full UX, GPU rendering and elaborate parametric stroke editors.

## Research decision

**Current gate:** The P4-A.3 synthetic engineering and harness gates are documented as passing, but the real chair/mug/plant quality gate is not yet established. Until then, no claim that P4-A.3 is aesthetically superior to P2-B.1 is justified.

**Recommended next actions:** Follow the companion [phased plan](SCANSKETCH_IMPROVEMENT_PHASES_20261029.md). Every phase must have an explicit reject/rollback path; a failed experiment should improve the understanding of the problem, not force another permanent algorithm branch.
