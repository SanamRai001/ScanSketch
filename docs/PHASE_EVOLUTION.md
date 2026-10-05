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
| P3-A.0 — Directional tonal prototype | New **opt-in** `--directional` orientation-guided rotation of original tonal segments; fine/coarse source tensors, strict unsmoothed footprint support, original P2-B.1 contours | Implementation on [draft PR #8](https://github.com/SanamRai001/ScanSketch/pull/8), GitHub CI 41/41 tests; paired synthetic step altered **24/193** geometries at same stroke count/ink metadata ([run](https://github.com/SanamRai001/ScanSketch/actions/runs/37256450858)) | First portrait visual + numeric A/B completed: **no clear visual win and numeric regression at equal stroke count**. P2-B.1→P3-A.0: tone RMSE **0.397436→0.401782**, midtone **0.304446→0.311332**, dark **0.447258→0.451867**, edge F1 **0.288706→0.285884**; white RMSE improves slightly **0.016081→0.015727**. Both: **12,671 strokes**, essentially identical total path length; **581 stroke geometries changed** in P3-A.0 (~4.6% of the drawing). [Full result](P3A0_FIRST_VISUAL_REVIEW.md) | REJECT rotation-only configuration as a quality candidate; preserve experiment; P2-B.1 stays baseline; design P3-A.1 placement-aware proposals |
| P3-A.1 — Placement-aware proposals | Opt-in `--placement-aware`: source-dark anchors, coarse/fine/tonal pools, strict support, unchanged P2-B.1 contour append | Draft PR #9; CI **47/47**; portrait **12,671 vs 12,671 strokes**, strong nonhorizontal **97→2367** | Portrait tone RMSE **0.397436→0.529120**, dark RMSE **0.447258→0.601276**, path +3.06%; visibly more directional but loses tonal mass | **REJECT** placement-dominant renderer; preserve result; design P3-A.2 hybrid augmentation |
| P2-C — Evidence tooling | Fixed experiment protocol, source/preview/stroke measurement CLI and six synthetic fixtures | Draft docs [PR #6](https://github.com/SanamRai001/ScanSketch/pull/6), tool [PR #7](https://github.com/SanamRai001/ScanSketch/pull/7); CI **31/31**, successful white-image render→measure smoke | First real-photo same-binary **P2-A vs P2-B.1** comparison recorded; B.1 edge F1 **0.18406141 → 0.28870589** (+~56.9% *relative proxy*, at +108 strokes/+~0.63% path length). Overall tone RMSE changes only ~0.000835; white mask unchanged. Local generation of all **six** synthetic fixture sources reported successful; CI paired step/channel/gradient renders and measurements now passed, with **mixed** outcomes (see [synthetic CI result](P2C_SYNTHETIC_CI_RESULT.md)); Windows full-precision comparison **now completed successfully**; exact metrics show mixed contour outcomes (see [Windows synthetic A/B](P2C_SYNTHETIC_WINDOWS_RESULT.md)); image/mask inspection pending | Continue multi-fixture validation; don't claim equal-budget/human-quality superiority |

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

### Second portrait (initially mislabeled as an object)

The user ran a paired real-photo A/B into the local folder `object-results/` and then uploaded two previews. **Both are portraits, not a nonportrait object.** This must not be counted as cross-subject verification. At 400×512/seed 42, P2-A had 10,929 strokes, RMSE 0.35607746 and edge F1 0.48701302; P2-B.1 had 11,247 strokes, RMSE 0.35345628 and F1 0.66215220. The visual comparison still shows mostly horizontal hatching and weak small facial landmarks. Keep [the exact second-portrait result](P2C_SECOND_PORTRAIT_AB.md) alongside the first portrait; the nonportrait gate remains open.

## How every new phase gets recorded

1. **Before coding:** capture phase ID, motivating defect, hypothesis, baseline branch+commit, source/fixture IDs and rights, controlled settings, chosen acceptance criteria. Do not silently change the metric between variants.
2. **After implementation:** record PR, branch+exact commit, behavior added/removed, CLI arguments, tests and actual CI/local outcome. Preserve previous phase code/results.
3. **After rendering:** save a paired original/preview/ordered stroke JSON locally, with seed, working resolution, flags, hashes and machine/build profile. For meaningful images, compare at **100%** and include failures. No unapproved personal images in the public repository.
4. **After measurement:** append the real metric report or a clearly labeled anonymized summary; separate natural output budgets from truly matched-budget comparisons. Write both improvements and regressions, and note unmeasured values.
5. **At gate:** decision **retain / revise / reject / inconclusive** and the next test. Link the evidence and template. A successful build never automatically means visual approval.
6. **For future presentation:** use this ledger to build a consistent chronological image gallery. Do not draw comparisons using different sources, seeds, cropping, rendering sizes or unpublished proprietary reference imagery.

### Next entry reserved: P2-C synthetic fixture comparison

The user successfully created the following **source inputs only**, in `experiments/local/p2c-v1/`: `white.png`, `transparent-black.png`, `step.png`, `square-white-channel.png`, `gradient.png`, `thin-lines.png`. CI has since run and measured paired step/channel/gradient fixtures successfully: [P2-C synthetic CI result](P2C_SYNTHETIC_CI_RESULT.md). Step F1 was slightly worse in B.1 at displayed precision (0.82→0.79); channel showed a rounded ~0.03 white-ink rate in both modes; gradient had 0.00 displayed edge F1 in both. Thus this is **not** universal evidence in favor of contour accents.

The user subsequently reran [the synthetic A/B batch](P2C_RUN.md) locally, successfully generating six previews, six stroke JSONs, six reports and a local SHA manifest. Full-precision results: [Windows synthetic A/B](P2C_SYNTHETIC_WINDOWS_RESULT.md). The user uploaded three differently zoomed screenshots of the source and two previews. The protected vertical channel appears visually continuous, but the **53 affected pixels cannot be localized from screenshots**. The read-only [white pixel diagnostic](P2C_WHITE_CHANNEL_AUDIT.md) subsequently passed GitHub CI on raw synthetic PNGs: **0 of the 288 channel pixels** affected in both modes; all **53** affected pixels exterior, all within 2px of source nonwhite (49 within 1px). This resolves the question of *where* the 2.548% whole-white-mask ink occurs, not the exact antialias/cap mechanism. Next use [the paired nonportrait script](P2C_RUN.md) on a permission-cleared object photograph; individual gradient edge counts and human quality comparison remain open. The script's synthetic CI smoke is not a substitute for a nonportrait photographic trial. P3-A directional generation is still a design proposal, not a new outcome.

### P3-A.0 experiment opened (not a measured phase outcome yet)

The first prototype follows [P3-A.0 runbook](P3A0_VERIFY.md): rotate a bounded number of existing P2-A tonal proposals along fine/coarse structure-tensor tangents, retain all ink metadata and optional existing P2-B.1 contour marks. This is *opt-in*, with P2-B.1 as the unchanged control. The engineering/CI milestone passed: **41 Rust tests** and synthetic step A/B with **24 changed stroke geometries out of 193**, preserving original stroke count and width/opacity. First same-portrait photographic preview pair has now been visually reviewed: there is no compelling improvement over P2-B.1 at the uploaded scale and the face may be softer. [Qualitative evidence and caveats](P3A0_FIRST_VISUAL_REVIEW.md). The paired numeric table is now available and confirms the visual result: P3-A.0 worsens tone/midtone/dark RMSE and edge F1 at exactly the same stroke count and nearly identical path length. Only white RMSE improves slightly. The portrait-specific `geometrically_changed_strokes` count is now confirmed: **581/12,671**. That is materially nonzero but still a minority of the image. Combined with worse tone/edge metrics and no visual gain, this rejects the **rotation-only abstraction**, not merely its activation path. The required actual nonportrait P2-C trial remains outstanding.


### P3-A.1 experiment opened

P3-A.0 demonstrated that changing direction alone was insufficient. P3-A.1 therefore changes **proposal origin**: source-driven anchors are created before orientation is chosen. The frozen P2-B.1 tonal count is used only as a comparison budget; its tonal coordinates are not reused except for explicitly counted sparse-input fallback. See [P3-A.1 verification](P3A1_VERIFY.md). No visual or numeric outcome is recorded until CI and user A/B evidence exist.


### P3-A.1 portrait outcome

The first portrait pair decisively rejects whole-field placement-aware direction as the main tonal carrier. P3-A.1 changed the visual language dramatically (**2367 strongly nonhorizontal strokes vs 97 baseline**) while keeping total count at 12,671, yet overall tone RMSE worsened **33.13%** and dark-region RMSE **34.44%**. The result is more directional but less faithful. [Exact result](P3A1_FIRST_PORTRAIT_RESULT.md). This motivates P3-A.2: preserve P2 tonal mass and use only a small, earned structural reinforcement budget.


### P3-A.2 architecture opened

After P3-A.1 proved that whole-field direction damages tonal mass, the next experiment is deliberately hybrid: preserve P2 tone exactly and spend only the existing P2-B.1 contour budget on smarter residual-aware structural accents, falling back to original contours when needed. Architecture frozen in [P3A2_ARCHITECTURE.md](P3A2_ARCHITECTURE.md); no outcome claimed yet.


### P3-A.2 implementation opened

The hybrid architecture is now executable behind `--hybrid-structural`. It preserves the exact tonal prefix and total P2-B.1 stroke count, replacing only qualified structural-budget slots with residual-aware multiscale accents. No quality result claimed until CI and A/B review. [Verification](P3A2_VERIFY.md).


### P3-A.2 first engineering evidence

CI passed **53/53** tests. On the step fixture, P3-A.2 preserved all 173 tonal strokes and the 193 total-stroke budget, replacing **6 of 20** structural slots with residual-aware accents and falling back to 14 original contours. Rounded step metrics improved slightly in tone/dark/edge F1 but white RMSE also rose slightly. No phase win is claimed until the portrait and genuine nonportrait are reviewed. [Runbook](P3A2_VERIFY.md).


### P3-A.2 portrait outcome

P3-A.2 preserved the exact 12,563 tonal prefix and 12,671 total strokes. It nevertheless replaced **all 108** baseline structural strokes (2,024 hybrid candidates, 0 fallback). Metrics changed only slightly: overall tone −0.41% RMSE (better), dark −0.54% (better), midtone +1.91% (worse), edge F1 −0.10% (worse), white unchanged; path +0.10%. The previews are nearly indistinguishable. **Result: promising hybrid architecture, inconclusive replacement policy.** [Exact result](P3A2_FIRST_PORTRAIT_RESULT.md). Next refine to selective contour-vs-hybrid replacement rather than increasing budget.


### P3-A.2.1 architecture opened

Selective hybrid replacement is now the next controlled hypothesis. Rather than replace all P2-B.1 contours, score both baseline structural strokes and hybrid candidates under one residual/structure utility; replace only weakest baseline slots when a hybrid clears a 15% + 0.001 margin, capped at 40% of the structural budget. No outcome claimed yet. [Architecture](P3A21_ARCHITECTURE.md).


### P3-A.2.1 implementation opened

The selective hybrid architecture is executable as a separate experiment: exact tonal prefix, exact total stroke count, same structural-tail length, direct baseline-vs-hybrid utility comparison, 15% + 0.001 replacement margin and 40% cap. CI and visual outcome pending. [Verification](P3A21_VERIFY.md).


### P3-A.2.1 first engineering evidence

CI passed **56/56** tests. On the step fixture, P3-A.2.1 preserved 173 tonal strokes and all 193 total strokes, replacing only **6 of 20** baseline structural slots under a cap of 8 and retaining 14 original contours. Rounded tone/dark/edge proxies improved slightly, with a small white-RMSE increase. No quality claim until portrait/nonportrait review. [Verification](P3A21_VERIFY.md).


### P3-A.2.1 portrait outcome

Selective replacement produced the first substantial P3 edge gain without tonal collapse: edge F1 **+15.22%**, overall tone ~0.247% better, dark ~0.301% better, white unchanged, at only +0.072% path length. Midtone worsened ~0.740%. It replaced 43/108 structural slots and retained 65. The visual delta is still subtle and concentrated mostly in upper hair/silhouette. **Promising, not promoted.** Next target missing baseline structure rather than strongest source structure. [Exact result](P3A21_FIRST_PORTRAIT_RESULT.md).


### P3-A.2.2 architecture opened

The next controlled hypothesis targets **missing baseline structure** rather than strongest source structure. It reuses the exact P3-A.2.1 candidate pool and replacement policy, changing only the utility map to source-edge minus P2-B.1-preview-edge residual. No outcome claimed yet. [Architecture](P3A22_ARCHITECTURE.md).


### P3-A.2.2 implementation opened

The missing-structure experiment is executable as a separate mode. It reuses the exact P3-A.2.1 candidate pool and replacement controls; only baseline/candidate utility changes to emphasize source edges absent from frozen P2-B.1. No quality result claimed until CI and portrait localization. [Verification](P3A22_VERIFY.md).


### P3-A.2.2 first engineering evidence

CI passed **59/59** tests. On the step fixture, missing-structure scoring preserved 173 tonal strokes and all 193 total strokes, replacing **6 of 20** structural slots under the unchanged cap of 8 and retaining 14 baseline contours. The accepted replacements carried mean missing-edge evidence **0.126235**. Rounded tone/dark/edge proxies improved slightly with the same small white-RMSE caution. No phase win is claimed until portrait/nonportrait review. [Verification](P3A22_VERIFY.md).


### P3-A.2.2 portrait outcome

The scoring-only missing-edge experiment **did not improve the portrait**. Edge F1 fell ~0.99% (0.288706→0.285843); midtone worsened ~0.195%; tone/dark improved only ~0.042%/~0.055%; white stayed identical and path changed +0.009%. The visual pair remains nearly unchanged and weak interior structure is not recovered. **Reject scoring-only refinement.** This isolates the candidate pool as the next bottleneck: P3-A.2.3 should propose candidates directly from missing-structure peaks while freezing selective-replacement controls. [Exact result](P3A22_FIRST_PORTRAIT_RESULT.md).


### P3-A.2.3 architecture opened

The next controlled hypothesis changes proposal coverage rather than scoring. Candidate anchors are driven by missing-edge residual while P3-A.2.2 scoring/replacement controls remain frozen. This tests whether missing structure failed because useful strokes never entered the candidate pool. No result claimed yet. [Architecture](P3A23_ARCHITECTURE.md).
