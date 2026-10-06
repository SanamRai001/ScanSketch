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


### P3-A.2.3 implementation opened

Deficit-driven proposal coverage is now executable as a separate experiment. Only the candidate signal changes from tonal residual to missing-edge residual; P3-A.2.2 scoring/replacement controls remain frozen. No quality result claimed until CI and portrait localization. [Verification](P3A23_VERIFY.md).


### P3-A.2.3 first engineering evidence

CI passed **62/62** tests. On the step fixture, candidate origin changed from tonal residual to missing-edge residual while all replacement controls remained fixed. Candidate count fell **55→26**, replacements stayed 6/20 under the same cap of 8, and mean missing-edge evidence at accepted replacements rose **0.126235→0.127317**. This is the intended proposal-coverage behavior; no quality win is claimed until portrait/nonportrait review. [Verification](P3A23_VERIFY.md).


### P3-A.2.3 portrait outcome

At the same 12,671 strokes, P3-A.2.3 improved edge F1 **8.918%** (0.288706→0.314451), kept white RMSE identical, changed path only +0.0601%, and kept tone/dark nearly flat; midtone worsened ~0.936%. The previews remain visually close. This is **promising proposal-origin evidence**, but P3-A.2.1 still has the higher absolute portrait edge F1. Decision: freeze P3-A.2.3 and test genuine nonportrait generalization before more tuning. [Result](P3A23_FIRST_PORTRAIT_RESULT.md).


### P3-G1 — nonportrait generalization validation

After the promising P3-A.2.3 portrait result, renderer tuning is paused. P3-G1 adds a local-only 3–5 image validation harness: frozen P2-B.1 vs P3-A.2.3 for multiple permission-cleared nonportrait photos, exact invariant checks, aggregate JSON/CSV and local HTML A/B review. This phase tests generality; it does **not** change sketch generation. [Protocol](P3G1_PROTOCOL.md).


### P3-G1 validation infrastructure verified

The multi-photo harness passed CI end-to-end: three local-style cases, per-case frozen invariant checks, aggregate JSON/CSV and side-by-side HTML review. Synthetic smoke results were mixed rather than universally positive, which is desirable for an evidence tool. Real nonportrait photographic evidence remains the actual gate. [Run 37354095490](https://github.com/SanamRai001/ScanSketch/actions/runs/37354095490).


### P3-G1 real-pack outcome

The first genuine nonportrait pack (chair, mug, plant) rejected broad P3-A.2.3 promotion: edge wins 1/3 and exploratory mean edge-F1 change -2.137%; tone/midtone means also regress. More importantly, visual review shows the shared P2/P3 primitive language is still dominated by 2–10px horizontal fragments. **Long human gesture/contour strokes do not exist yet.** Decision: end this line of selection-only tuning and move to P4 stroke hierarchy. [Exact result](P3G1_FIRST_RESULT.md).


### P4-A — human stroke hierarchy architecture

Opened after P3-G1. The project now explicitly separates three mark scales: long structural/gestural paths, medium form-following paths and short hatch/texture marks. Current 2–10px scan fragments remain useful only as the shortest layer. P4 begins with a first-class editable path primitive, then long tracing, medium form strokes and residual hatching. [Architecture](P4A_HUMAN_STROKE_HIERARCHY.md).


### P4-A.0 — path primitive foundation

Implementation opened for one logical editable curved/polyline stroke. Old straight-only JSON remains readable and empty path collections are omitted, so historical P1–P3 outputs/scripts remain compatible. Rendering and metrics gain path awareness; automatic source tracing remains deferred to P4-A.1. [Verification](P4A0_VERIFY.md).


### P4-A.0 engineering result

First-class path support passed 69 tests and all inherited experiment checks. A seven-point curved gesture rendered as one logical 82.87px mark while old stroke-only reports stayed compatible. The representation bottleneck is cleared; P4-A.1 can now test actual long-path extraction from image structure.


### P4-A.1 — long structural path tracing

Opened the first real human-scale mark experiment. P4-A.1 traces edge-supported tangent streamlines into first-class Gesture paths, targeting roughly 24–96px instead of 2–10px fragments. Overlay mode preserves frozen P2-B.1 exactly; paths-only mode exposes the structural layer without scanline noise. No tone rebalance yet. [Verification](P4A1_VERIFY.md).


### P4-A.1 first engineering result

Automatic source tracing now produces real human-scale marks: two 48.5px-average gesture paths on the vertical-step smoke, versus the historical 2-10px primitive scale. All 74 tests and inherited experiments remain green. This validates tracing capability only; the next evidence must show whether chair/mug/plant path layers follow meaningful object structure.


### P4-A.1 multi-image validation harness verified

The real-pack workflow now handles 0/1/many path cases robustly, generates baseline/overlay/paths-only views, aggregate JSON/CSV and local HTML, and passed the synthetic 3-source smoke. This closes tooling risk; the remaining question is visual quality on chair/mug/plant.


### P4-A.1 validation tooling complete

The three-view multi-image harness is now CI-green, including a legitimate zero-path gradient case. Synthetic step/thin-line fixtures produced 47–49px logical gestures while paths-only stayed free of legacy scan fragments. P4-A.1 now waits only on real chair/mug/plant structural review before deciding whether to refine tracing or advance to medium form strokes.


### P4-A.1 validation harness verified

The three-view batch runner is green and correctly preserves neutral/no-path cases. On synthetic sources, long gestures appeared on step and thin-lines but not gradient, confirming the tracer can abstain. The next evidence must come from chair/mug/plant path layers, not from more synthetic tuning.


### P4-A.1 real-image outcome

Chair, mug and plant all produce meaningful long Gesture paths with ~45-55px mean lengths and up to ~80px maximum. The paths-only review finally shows a new visual vocabulary: continuous structural lines instead of short horizontal fragments. P4-A.1 is accepted as the long-stroke layer. It is intentionally sparse; next add an 8-32px Form layer rather than increasing gesture density. [Result](P4A1_FIRST_REAL_RESULT.md).


### P4-A.2 architecture opened

The second human-stroke scale is now defined: exact P4-A.1 long Gestures stay frozen, while 8-32px `Form` paths target missing interior structure. Medium seeds prefer edges with source content on both sides and avoid accepted Gesture corridors, so the new layer should describe form rather than redraw silhouette. [Architecture](P4A2_ARCHITECTURE.md).


### P4-A.2 implementation opened

The second stroke scale is now executable: P4-A.1 Gestures are preserved exactly, then 8-32px Form paths are added from denser edge proposals with interior preference and Gesture suppression. The key visual comparison is P4-A.1 paths-only versus P4-A.2 paths-only; old hatching remains untouched until P4-A.3. [Verification](P4A2_VERIFY.md).


### P4-A.2 first engineering result

The medium layer clears its engineering gate: 79 tests green, exact P4-A.1 Gesture preservation, and a 15px Form path added to the short internal feature while Gestures retain ~92% of structural path length. The hierarchy is behaving as intended; real-object visual review is next.


### P4-A.2 validation tooling verified

The multi-image hierarchy harness is CI-green, including no-op sources. Medium Forms appear only on the dedicated internal-detail fixture, while step/gradient add none. This is desirable selectivity; the next evidence is the real chair/mug/plant paths-only delta.


### P4-A.2 first real outcome

Medium Form hierarchy succeeds across chair/mug/plant: six Forms per source, 12–16px mean Form length versus 45–55px Gestures, ~25–31% Form path-length share, and paths-only edge-F1 gains of ~24–32% on all three. The medium layer is visibly subordinate and useful. Freeze Gesture+Form geometry. The remaining visual problem is the old full horizontal tone field, so P4-A.3 moves to sparse residual hatching rather than overlay accumulation. [Result](P4A2_FIRST_REAL_RESULT.md).


### P4-A.3 — residual hatching rebalance architecture

After successful Gesture+Form hierarchy, tone is rebuilt instead of inherited. P4-A.3 renders frozen structure first, computes compressed remaining tonal need, suppresses hatching around structural corridors, and adds sparse short `Hatch` paths. No legacy P2-B.1 segments are part of the P4 candidate. The first experiment changes density/composition only; hatch direction remains mostly horizontal to preserve causal clarity. [Architecture](P4A3_ARCHITECTURE.md).


### P4-A.3 implementation opened

The old scan field is no longer part of the P4 candidate. Gesture+Form render first; compressed source-vs-structure residual then generates short role-tagged Hatch paths with a <=55% control-count budget and protected structural corridors. This isolates density/composition while leaving Hatch direction mostly horizontal for now. [Verification](P4A3_VERIFY.md).


### P4-A.3 first engineering result

The first full P4 composition is operational: 4 Gesture + 1 Form + 42 Hatch marks on the synthetic form-detail fixture, versus 161 legacy control segments. Hatch count falls **73.9%**, structure supplies ~49.3% of path length, all marks are role-aware paths, and no old straight segments remain. 85 tests and every historical smoke stay green. Next evidence must be visual on the real chair/mug/plant pack.


### P4-A.3 multi-image validation harness

The first full P4-only tonal composition now has a 3-5 image batch gate. The harness emphasizes mark hierarchy and Hatch-count reduction rather than RMSE and provides a four-view review so density/composition problems can be separated from remaining mechanical Hatch orientation.
