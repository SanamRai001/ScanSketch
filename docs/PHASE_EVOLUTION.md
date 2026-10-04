# ScanSketch — Phase Evolution and Evidence Ledger

**Purpose:** preserve an honest, reviewable history of how ScanSketch changes, so future demos can show *what* improved, *why* we changed the algorithm and *which evidence supports it*. Maintained alongside [PROJECT_STATE.md](PROJECT_STATE.md), the single current-status record. Last reviewed: **2026-10-04**.

Do **not** overwrite an old result when a new phase begins. Add a row and a result entry using [PHASE_RESULT_TEMPLATE.md](PHASE_RESULT_TEMPLATE.md); link an immutable branch commit/PR and fixed-input preview or local artifact manifest. Mark unknown data as **not measured**, not zero. Only publish source images/outputs when rights and privacy permit. Private portrait files stay local in `outputs/` (Git-ignored).

## At a glance

| Phase | What was introduced | Verification actually observed | Observed visual / numerical outcome | Decision |
| --- | --- | --- | --- | --- |
| P0 — Foundation | Vision, Rust-first architecture, research scope and gated roadmap | Foundation in draft [PR #1](https://github.com/SanamRai001/ScanSketch/pull/1) | Planning-only; **no rendered image**, so no quality metric | Keep as design starting point; not merged into main |
| P1 — Original scanline | White-matted linear darkness, top-to-bottom mostly horizontal source-derived strokes, ordered vector records and PNG preview | User's Windows check, **11/11 tests**, actual portrait render; draft [PR #2](https://github.com/SanamRai001/ScanSketch/pull/2) | Recognizable silhouette and preserved paper, but long bars resembled mechanical engraving; comparable quantitative metric **not collected** | Keep historic baseline; change stroke language |
| P2-A — Broken strokes | Seeded short tonal fragments, gaps, small angles and dark-region second layer | CI **14/14**, portrait render; draft [PR #3](https://github.com/SanamRai001/ScanSketch/pull/3) | Fewer continuous barcode-like rows; face/eyewear/hair structure still weak. Later same-binary tonal-only control: **12,563 strokes**, tone RMSE **0.39827129**, edge proxy F1 **0.18406141** at 368×512/seed 42 | Keep tonal control; test structure |
| P2-B — Raw Sobel contours | Sparse tangent contour accents with source support, independently toggleable | CI **20/20**, user A/B images; draft [PR #4](https://github.com/SanamRai001/ScanSketch/pull/4) | Same-photo tonal-only **12,563** vs contour-on **13,024 strokes** (+461). Visually subtle: facial anchors still insufficient. Numeric edge score for raw P2-B **not measured** | Preserve baseline; refine selection instead of increasing all contours |
| P2-B.1 — Coherence ranking | Mild 3×3 smoothing, directional NMS, tangent continuity ranking and tile quotas; original-darkness support checks | CI **23/23**, user screenshot; draft [PR #5](https://github.com/SanamRai001/ScanSketch/pull/5) | **12,671 strokes** at 368×512/seed 42 (+108 over P2-A). In later same-binary measurement, tone RMSE **0.39743606**, edge F1 **0.28870589**, white RMSE unchanged from P2-A. Subjective visual gain remains incremental | Current strongest visual candidate and measured structural reference, **not** a finished sketch |
| P2-C — Evidence tooling | Fixed experiment protocol, source/preview/stroke measurement CLI and six synthetic fixtures | Draft docs [PR #6](https://github.com/SanamRai001/ScanSketch/pull/6), tool [PR #7](https://github.com/SanamRai001/ScanSketch/pull/7); CI **31/31**, successful white-image render→measure smoke | First real-photo same-binary **P2-A vs P2-B.1** comparison recorded; B.1 edge F1 **0.18406141 → 0.28870589** (+~56.9% *relative proxy*, at +108 strokes/+~0.63% path length). Overall tone RMSE changes only ~0.000835; white mask unchanged. Local generation of all **six** synthetic fixture sources reported successful; complex fixture *rendering/measurement still pending* | Continue multi-fixture validation; don't claim equal-budget/human-quality superiority |

**Measurement note:** P2-A and P2-B.1 numbers above are from one portrait, **same P2-C executable** with contour pass toggled and unchanged `p2c-v1` metric thresholds. Historical P1/P2-B variants were not retrospectively measured under those exact conditions. The source is private and not uploaded. The edge proxy uses binomial-smoothed Sobel masks with 0.22 threshold and ±2px matching; **56.9% better edge F1 does not mean 56.9% better-looking art**. This natural-default comparison uses unequal accepted stroke counts and is not an equal-budget trial. See [full A/B table](P2C_PORTRAIT_AB.md).

## Visual development timeline

These are **local artifact references**, not public links or proof that every historical file still exists. Keep original outputs, unique filenames and associated JSON/seed where available. A future publicly displayable evolution gallery should use approved/licensed sources and reproducible renders of all historical engine revisions at the same size.

| Stage | Known local preview / paired records | What to show later |
| --- | --- | --- |
| P1 | Original portrait preview; exact filename not confirmed | Continuous scan bars and preserved white space |
| P2-A | `outputs/sample-p2a.png`, later `outputs/portrait-p2a-control.png`; measured control `outputs/p2c-portrait-p2a-control.png` + `-strokes.json` | Broken lines vs P1 |
| P2-B | `outputs/portrait-p2b-contours.png` | Original raw-Sobel overlay vs same-photo no-contours control |
| P2-B.1 | `outputs/portrait-p2b1-ranked.png`; measured `outputs/p2c-portrait.png` + `outputs/p2c-portrait-strokes.json` | Ranked-coherence refinement vs earlier contours; crop around eyewear and lips |
| P2-C | `outputs/p2c-portrait-report.json`, `outputs/p2c-portrait-p2a-control-report.json`; synthetic inputs under `experiments/local/p2c-v1/` | A/B metrics beside the corresponding preview, including unchanged white mask |

## How every new phase gets recorded

1. **Before coding:** capture phase ID, motivating defect, hypothesis, baseline branch+commit, source/fixture IDs and rights, controlled settings, chosen acceptance criteria. Do not silently change the metric between variants.
2. **After implementation:** record PR, branch+exact commit, behavior added/removed, CLI arguments, tests and actual CI/local outcome. Preserve previous phase code/results.
3. **After rendering:** save a paired original/preview/ordered stroke JSON locally, with seed, working resolution, flags, hashes and machine/build profile. For meaningful images, compare at **100%** and include failures. No unapproved personal images in the public repository.
4. **After measurement:** append the real metric report or a clearly labeled anonymized summary; separate natural output budgets from truly matched-budget comparisons. Write both improvements and regressions, and note unmeasured values.
5. **At gate:** decision **retain / revise / reject / inconclusive** and the next test. Link the evidence and template. A successful build never automatically means visual approval.
6. **For future presentation:** use this ledger to build a consistent chronological image gallery. Do not draw comparisons using different sources, seeds, cropping, rendering sizes or unpublished proprietary reference imagery.

### Next entry reserved: P2-C synthetic fixture comparison

The user successfully created the following **source inputs only**, in `experiments/local/p2c-v1/`: `white.png`, `transparent-black.png`, `step.png`, `square-white-channel.png`, `gradient.png`, `thin-lines.png`. This establishes fixture generation, **not yet successful renders or metric scores** for the latter five.

Next run [the synthetic A/B batch](P2C_RUN.md) (step / channel / gradient, each with `--no-contours` and B.1), collect the resulting local summary, inspect the PNGs and add an evidence row here. Then use a permitted **nonportrait** object photograph. P3-A directional generation is still a design proposal, not a new outcome.
