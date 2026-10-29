# ScanSketch — Phased Architecture and Quality Improvement Plan

**Proposed:** 2026-10-29  
**Status:** PROPOSED / read-only planning artifact. No algorithm or UI changes are authorized by this document alone.  
**Starting point:** P4-A.3 draft PR #19, branch feat/p4a3-residual-hatching, inspected commit 58b385106e1b1c6c2e844b14d7bbddab83591d56.  
**Evidence and criticism:** [SCANSKETCH_ARCHITECTURE_AUDIT_20261029.md](SCANSKETCH_ARCHITECTURE_AUDIT_20261029.md)

## Intended outcome (definition of success)

ScanSketch should turn a rights-cleared photograph or illustration into a **recognizable, purposeful, human-sketch-like drawing composed of actual editable strokes**. It should preserve meaningful structure, use tone as support, keep the process local and deterministic, and make its drawing records replayable.

Success is not merely more edges, fewer marks, lower pixel RMSE, more features or higher test counts. We need independent evidence that people prefer the drawing to the existing scanline and simple contour baselines across different image subjects.

**Scope boundary:** Research-quality local CPU renderer first. UI, SVG and WASM only after the core quality gates. No backend, accounts, payments, image uploads or AI service dependency are currently justified.

## Non-negotiable invariants

1. Preserve existing P1–P3 outputs, experiment flags, historical fixtures and recorded negative results unless a deliberately approved compatibility change is necessary.
2. Fixed source bytes, processing configuration, engine version and seed must reproduce the same ordered records and raster preview within defined platform/build constraints.
3. The accepted P4 composition has Gesture, Form and Hatch as first-class logical paths, with **zero legacy straight-stroke records** in P4-only output.
4. White/transparent paper protection and documented input/decode/dimension/memory bounds continue to work.
5. Never commit private source portraits, local datasets, generated artifacts or API keys. Experiments use rights-cleared assets and preserve per-image source provenance.
6. Regression tests distinguish exact-path invariants from aesthetic judgments. Numerical or synthetic-only wins do not authorize changing the default renderer.
7. Keep a single [PROJECT_STATE.md](PROJECT_STATE.md) for *accepted* state/decisions; keep this proposal and the [phase evolution ledger](PHASE_EVOLUTION.md) as separate historical/planning documents. Do not create another live status tracker.
8. Do not merge all stacked historical PRs automatically, rewrite shared history, delete branches or change the default/main branch as part of a narrow improvement phase.

## Proposed order and dependencies

| Phase | Theme | Depends on | Exit |
| --- | --- | --- | --- |
| P0 | Reconcile evidence and freeze baseline | Existing P4-A.3 | Local representative baseline is reproducible; real-image assessment documented |
| P1 | Correctness + engineering safety | P0 baseline | P4 has no mandatory P2 control dependency; outputs and budgets are safe |
| P2 | Independent sketch-quality gate | P0, preferably P1 | Held-out, blinded comparisons identify actual visual failure modes |
| P3 | Spatially fair and usefulness-ranked Hatch selection | P2 | Accepted only on measured quality improvement |
| P4 | Better stroke language and bounded optimization | P2; P3 only if accepted | Controlled quality win over simpler alternatives |
| P5 | Performance, stable output contract, repeatability | Accepted renderer from P1–P4 | Measured performance and reliable versioned artifacts |
| P6 | Minimal local-first UI and export | P2 quality gate + P5 contract | Safe, accessible end-to-end user workflow |
| P7 | Integrate and publish an honest baseline | Phases actually accepted | CI/quality/repro gates pass on an explicit integration candidate |

Each experimental phase is optional after its gate. Rejection is a successful research outcome. The numbers below are proposed decision thresholds to review before committing to them; they are **not existing measured achievements**.

---

## P0 — Baseline and research truth (NO ALGORITHM EDITS)

**Why:** P4-A.3 has recorded successful engineering checks, but the final chair/mug/plant perceptual review remains pending. A code change before capturing that result destroys a useful control.

**Read-only actions:**
- Inspect branch, worktree, untracked/stashed work and open stacked PRs; start from a verified P4-A.3 SHA, not the README-only main branch.
- Run the existing four-view P4-A.3 harness on the permitted local chair/mug/plant pack with fixed seed 42 and max side 512.
- Preserve P2-B.1, P4-A.2 paths-only, P4-A.3 final and Hatch-only previews plus JSON/CSV and review.html locally.
- Record source hashes, exact commit, flags, build environment, runtime and whether outputs can be regenerated.
- Inspect object recognizability, major edge gaps, relative structure/Hatch dominance, hatch band artifacts and paper cleanliness. Capture explicit FAIL as well as PASS examples.
- Reconcile README/ARCHITECTURE/project-state descriptions *in a later accepted documentation update*; this plan must not overwrite active historical records merely to sound current.

**Existing command** (run from an existing clean checkout after selecting P4-A.3; choose a new output folder):

~~~powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p4a3-pack.ps1" `
  -InputDir ".\experiments\local\p3g1\sources" `
  -OutputDir ".\experiments\local\p4a3\audit-baseline-01" `
  -MaxSize 512 -Seed 42 -RightsConfirmed -NonPortraitConfirmed
Start-Process ".\experiments\local\p4a3\audit-baseline-01\review.html"
~~~

**Acceptance:** Confirm exact P4 layer invariants and a documented per-image visual judgment. If P4-A.3 fails readability, record why and do not automatically begin orientation tuning; decide whether missing structure or hatching is responsible.

**Artifacts:** Local immutable baseline, one accepted P4-A.3 result note in existing phase history, exact hashes and a small failure taxonomy. No new app features.

## P1 — Remove accidental legacy coupling; tighten CLI/output correctness

**Issues:** A01, A09, A10; portions of A07.

**Core changes:**
- In scansketch-core, separate validation of SketchOptions/source dimensions from generate_sketch.
- In long_paths.rs, let P4 Gesture/Form generation operate without always building the P2 segment list. Legacy overlay mode may request the control explicitly.
- In residual_hatch.rs, stop using a mandatory fully generated legacy image as a P4-only prerequisite. Define an output logical-mark cap; keep P2 control counts only in comparison tooling or an explicit diagnostic path.
- Keep P4 exact Gesture/Form preservation, no legacy segments and unchanged Hatch-only parity. Document any intended changes to Hatch budget semantics rather than calling them 'equivalent'.
- In CLI, guard existing PNG/JSON destinations and alias/collision cases; use safe temporary output staging where practical. If the second file fails, clearly report whether a first file remains. Ensure failed runs never silently overwrite the source.
- Print correct straight-segment, path and total logical-mark counts; identify current P4 explicitly while preserving historical opt-ins and old default until the visual gate.

**Required tests:**
1. A fixture whose P2 segment count exceeds requested max_strokes but whose P4 output fits can render in P4-only mode. The old behavior should fail first in a regression test, then pass after the fix.
2. Retain historical P1–P3 path/count/hash controls and P4 exact structure invariants where intended.
3. Check all invalid options/dimensions, zero-structure image, transparent paper, deterministic seeding, boundary geometry and budget exhaustion.
4. Test existing-output rejection, output/source identity with normalized aliases where supported, JSON collision and partial-write failure handling.
5. Run cargo fmt --all -- --check, cargo check --workspace and cargo test --workspace; add Clippy once baseline warnings are understood.

**Safety:** No wholesale CLI rewrite, no changed default mode, no stored image uploads. Isolate minimal code changes in one focused PR.

**Exit:** No mandatory P2 generation for P4-only; cap applies to actual P4 marks; old modes stay valid; safety tests pass. If exact historical P4 output changes, review and record why.

## P2 — Independent visual evaluation, before more heuristics

**Issue:** A06. This is the most important *product-value* phase.

### Dataset and protocol

Start with the existing three permitted real nonportrait sources as a diagnostic pack, then create a small rights-cleared held-out pack. Proposed initial target: **at least 12 images across portraits, manufactured objects, plants/nature, buildings and low-contrast/detailed scenes**, with a separate development pack for tuning. Do not require personal photos to be uploaded publicly.

For each image:
- Fixed preprocessing, resolution, engine SHA and transparent handling; record hash/rights/source category.
- Run P2-B.1, P4-A.2 hierarchy, P4-A.3, and a deliberately simple contour-only comparison; optionally compare a tested VTracer baseline under a genuinely equivalent visual brief.
- Run fixed seeds (e.g. 42, 7, 99) if randomness materially changes output.
- Capture time, actual logical marks, total mark length, white-region contamination, tone RMSE, edge F1 and notable failure modes.
- Present blind, randomized side-by-side comparisons to at least two reviewers where practical, without disclosing which variant is newer.
- Ask separate questions: object recognizable? structure confident? does it read as hand-drawn? is tone appropriately subordinated? are there distracting scan bands?
- Review at the same display size and 100% zoom; never use identical generators/metrics as the sole judge.

**Proposed, not proven promotion gate:** P4 should win a clear majority of paired human-likeness preferences across held-out sources (a starting decision threshold could be 60%, subject to review), without severe recognizability or white-paper regression on any high-value subject. If the sample is too small or reviewers disagree, record uncertainty rather than forcing promotion.

**Diagnostic gate:** Compare failure categories separately for portraits, organic shapes and manufactured objects. A high edge-F1 win over a sparse path-only baseline is not proof of perceptual superiority over the old full reconstruction.

**Exit branches:**
- If P4 is recognizably good and only Hatch orientation looks wrong, isolate a direction-only experiment.
- If important contours are absent, prioritize structural proposal coverage before tonal styling.
- If P4 loses to a simple vector-outline control, stop and challenge the chosen approach rather than accumulating more heuristics.
- If the whole engine already passes, prioritize stability/performance over speculative algorithm expansion.

## P3 — Budget-aware Hatch placement experiment

**Issues:** A02, A04 and A05. **Start only if P2 identifies Hatch distribution as an important failure.**

**Architecture (smallest viable trial):**
- Keep P4 Gesture and Form lists **byte-for-byte / record-for-record frozen**.
- Compute compressed positive residual once.
- Generate a bounded pool of candidate Hatch records, not a stream that immediately spends the output budget.
- Rank candidates with source need, incremental tonal improvement or proxy, structural proximity exclusion and redundancy penalty. Use deterministic tie breaks based on coordinate and candidate index.
- Introduce tile minimum coverage and per-tile maximum allocation, allowing empty/white tiles to receive zero ink. Avoid fixed quotas that invent hatching in bright regions.
- Select under the same logical-mark budget as the control, then order accepted paths top-to-bottom for rendering/playback.
- Keep a clearly labeled legacy-scan allocator available for A/B comparison.

**Test fixtures:** White; upper-dark/lower-object; lower-dark/upper-object; two separated dark masses; diagonal form; transparent black; no-structure gradient; large saturated black region.

**Metrics:** Useful area coverage, residual mass per region, edge/detail retention, source white contamination, count/path-length/ink, runtime and visual judgment.

**Exit:** Promote only if held-out human preference and spatial coverage improve without unacceptable runtime/memory increase. If quotas add mechanical patterns, reject them and retain the simple baseline.

## P4 — Make marks more like purposeful drawing

**Issues:** A03, A04, A05, A08. This is a sequence of **separate experiments**, not one large refactor.

**P4.1: Orientation-only Hatch test**
- Compare mostly-horizontal hatching against tangent/flow-field alignment with identical structure, similar mark counts, seeded offsets and density.
- Change only angle policy. Protect source whites and Gesture/Form corridors.
- Accept only if blinded viewers consistently see better form-following, not merely 'more diagonal lines'.

**P4.2: Smooth path and pressure-language test**
- Compare raw polyline against simplification/curve-fitting for long Gesture paths; preserve edge support and avoid oversmoothing corners.
- Introduce optional pressure/width variation only after defining backward-compatible Sketch serialization and playback behavior.
- Keep a visually testable foundation with start/end taper and no simulated paper/graphite textures yet.
- Compare recognizability and perceived drawn quality with constant mark budgets.

**P4.3: Bounded candidate acceptance optimizer (only if needed)**
- Adapt *ideas* from [Primitive](https://github.com/fogleman/primitive): local mutation, affected-region scoring and keep/reject decisions.
- Example bounded utility: local reconstruction improvement + edge coherence - duplicate-overlap penalty - mark-complexity penalty. Calibrate on development images, not held-out tests.
- Avoid claiming a raw pixel RMSE optimizer produces human-like sketches; maintain a human-preference gate.
- Prefer small local ROI rerendering and a strict candidate/time budget over global exhaustive search.
- Compare against existing greedy P4 on same images/seed and report time-quality tradeoffs.

**Exit per subphase:** One changed variable; identical comparison setup; a visible win on held-out images; exact ordered-output reproducibility. Reject any variant that looks visually worse regardless of a favorable internal score.

## P5 — Engineering hardening, performance and document format

**Issues:** A07–A10. **Do not optimize without a measured profile.**

**Work:**
- Consolidate reusable darkness/edge/orientation computations into a private ImageAnalysis context shared by long and medium path generation.
- Benchmark end-to-end decode, analysis, proposal, selection, rasterization and export separately on small/medium/large permitted fixtures; capture peak allocation where feasible.
- Evaluate incremental tiny-skia rendering or region-scoped error comparison only for confirmed hotspots. Keep simple single-threaded behavior unless profiling justifies more.
- Add a versioned Sketch document envelope with schema version, generator mode, full parameter set, source hash, engine identifier and dimensions; maintain a legacy reader/migration path for old segment/path JSON.
- Define a single playback order or stable layer+sequence schedule without losing compatibility; rendering and measured exports must use the same order.
- Commit a reviewed Cargo.lock for the native CLI, capture a tested Rust toolchain version and strengthen fmt/check/test/Clippy and platform smoke checks.
- For SVG, start with export from existing paths plus a raster/SVG parity comparison. Do not rewrite the renderer or bring in a heavyweight vector package without proof.

**External leverage decision:**
- [VTracer](https://github.com/visioncortex/vtracer): reference/baseline for contour-vector conversion; study what it does better, then test before reusing it.
- [Primitive](https://github.com/fogleman/primitive): reference for candidate-local scoring; retain ScanSketch's pencil marks and experiment hygiene.
- [tiny-skia](https://github.com/linebender/tiny-skia): continue using the current renderer first.
- Verify licensing, targets, dependencies and maintenance at the point of reuse. No blind vendoring.

**Exit:** Reproducible artifact metadata, backwards compatibility, no input data loss, real benchmark evidence and no quality regression. Exact speed targets must be derived from a recorded baseline, not invented in this plan.

## P6 — Minimal local-first product UX (only after accepted visual core)

**Product scope:** One image in, a controlled drawing out; no account or cloud dependency.

**User flow:** Choose local PNG/JPEG -> preview + validated constraints -> render/cancel -> compare source and output -> inspect modes/seed -> export PNG plus editable sketch JSON and optional verified SVG.

**UX requirements:**
- Clearly state whether the selected mode is stable, experimental or historical; surface current recommended default only after its gate.
- Minimal quality presets with progressive disclosure for seed and research parameters.
- Honest progress/cancel states and friendly errors for large/unsupported images or output failures.
- Side-by-side / before-after slider with proper keyboard access, visible focus, alt text, readable contrast and reduced-motion compliance.
- Keep image processing local in a Web Worker if WASM portability and memory budgets are verified; show safeguards instead of promising browser privacy without testing.
- Preserve stroke-by-stroke replay as a possible later experience, but do not let animation dictate the core raster quality algorithm.

**Architecture gate:** Build a small wasm-bindgen compatibility spike and measure real worker memory before choosing React/Vite/Tailwind. The UI should call a stable core API; it must not reproduce generation logic in TypeScript. No unnecessary backend/database.

**Exit:** End-to-end local conversion on target browsers; responsive/accessibility acceptance; render cancellation and failure states verified; output parity with approved native engine.

## P7 — Controlled integration, documentation and a truthful release candidate

**Purpose:** Make the repository represent a real usable state instead of an indefinitely stacked experiment.

**Activities:**
- Review branch graph and uncommitted/stashed work before integration.
- Choose an explicit integration base; merge/cherry-pick only accepted code and required prerequisites after comparing actual tree and tests. Preserve rejected variants as references, not default paths.
- Update [README.md](../README.md), [ARCHITECTURE.md](ARCHITECTURE.md), [ROADMAP.md](ROADMAP.md) and the single [PROJECT_STATE.md](PROJECT_STATE.md) only after reconciling with actual code/Git/CI.
- Separate current engine setup from historical phase journals. Keep local/source assets ignored.
- Verify repo root commands, generated outputs, branch provenance, latest default mode and support limitations.
- Do not merge this documentation PR or any engine PR to main automatically; obtain explicit approval for final integration/release actions.

**Exit:** A cloneable reviewed branch is honestly documented; tests, visual evidence and supported use cases are clear; no claim exceeds observed behavior.

---

## Definition of done for every implementation phase

A phase is complete only when:
- The problem and control baseline are explicit.
- The code diff is scoped and does not silently change historical experiments.
- A failing test or controlled negative baseline is captured when feasible.
- Unit, integration/CLI and synthetic fixture tests relevant to the change are run.
- At least one actual representative image is visually inspected for changes affecting output.
- Output data/white-paper/privacy bounds remain valid.
- Numeric evidence and visual opinions are labeled separately, with seed, source hash, config and commit SHA.
- Rejected hypotheses and remaining uncertainties are written into the existing phase history.
- The branch/commit and verification status are communicated without claiming tests that were not executed.

## Developer verification commands (Windows PowerShell)

Use these commands *after* checking your intended clean checkout and branch. Heavy real-image runs stay local; GitHub CI may use synthetic fixtures.

~~~powershell
git status --short --branch
git branch --show-current
git log -1 --format="%h %s"
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
~~~

For the exact current P4-A.3 research reproduction, see [P4A3_RUN.md](P4A3_RUN.md). Never overwrite existing output folders; record the seed, max-size and source provenance. The baseline's documented CI pass counts are historical claims until rerun for the new target SHA.

## Priority and decision ownership

**Next immediate sequence:** P0 real-image evidence -> P1 correctness/decoupling -> P2 independent quality gate. Do **not** open P3/P4 experimental code until P2 identifies a concrete visual deficit.

**Decisions requiring explicit discussion:** changing the default renderer, changing public JSON contract, replacing a major algorithmic approach, adding cloud/AI dependencies, branching/rebasing stacked historical work, merging into main, or deleting old branches/assets.

**No-go:** building UI or speculative optimization merely to show progress while human-like sketch quality remains unproven.

**Success looks like:** a simpler and reproducible ScanSketch that produces fewer misleading 'wins', better sketches on held-out images, and a core architecture ready to support a focused product without major rewrites.
