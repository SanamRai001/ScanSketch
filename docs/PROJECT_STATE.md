# Project State

Last updated: **2026-10-04**. This is the single authoritative progress record; Git/CI, actual run outputs and observed visuals override planning documents.

## Goal and current decision

ScanSketch is a deterministic, CPU-first Rust system for rebuilding images from **actual ordered pencil strokes**, not grayscale pixel painting. Top-to-bottom sampling remains the design identity. After the first portrait iterations, **P2-B.1 is the current strongest subjective visual candidate** but is still not a convincing finished portrait renderer or an objectively established winner.

**Current active design: P3-A.2 Hybrid Structural Reinforcement, after rejecting P3-A.0 rotation-only and P3-A.1 placement-dominant rendering. P2-B.1 remains the frozen tonal baseline. P2-C's genuine nonportrait photo gate remains OPEN.** The first independent measurement utility is implemented and verified in GitHub Actions; the first **same-binary portrait A/B numeric comparison** (P2-A tonal-only versus P2-B.1 with contours) is now recorded. Multi-fixture, repeated-seed, signed tone-bias and matched-budget experiments remain pending. Stop stacking additional raw-Sobel contour heuristics without controlled evidence. The leading next prototype to test is multi-scale, direction-aware source-based stroke placement (P3-A), with Primitive-inspired scoring (P3-B) gated behind measurements.

## Stacked GitHub review

- `main`: initial bootstrap only. No branch merges, release or deployment.
- `docs/foundation`: draft PR #1.
- `feat/p1-rust-scanline-baseline`: draft PR #2 into foundation. User's Windows build/check and 11/11 core tests passed; first source/photo preview looked mechanically striped.
- `feat/p2-sketch-stroke-language`: draft PR #3 into P1. GitHub CI 14/14; fragmented marks helped compared with continuous horizontal bars.
- `feat/p2b-contour-reinforcement`: draft PR #4 into P2-A. GitHub CI 20/20. Same-image CLI at 368x512, seed 42: no contours 12,563 strokes, contour-on 13,024 (+461), visually modest structural benefit.
- `feat/p2b1-contour-coherence`: draft PR #5 into P2-B. GitHub CI 23/23. Same portrait preview received and qualitatively reviewed: strongest candidate so far, still an incremental improvement with weak glasses/eyes/lips and hair dominant.
- `docs/p2c-measurement-and-p3-design`: draft PR #6 into P2-B.1; protocol, visual evidence and proposed P3 architecture; CI passed 23/23.
- `feat/p2c-measurement-utility`, draft PR #7 into P2-C docs. New metrics and synthetic fixture tooling; existing reconstruction logic unchanged. CI passed the later **34/34 Rust tests** (31 original + three white-audit tests), workspace check, white fixture smoke, all six paired synthetic runs and exact protected-channel audits; [run 37222335502](https://github.com/SanamRai001/ScanSketch/actions/runs/37222335502).

- **Active P3-A.0 prototype:** `feat/p3a-directional-tonal-prototype`, based on P2-C metrics branch, to be reviewed in a stacked draft PR. Only `--directional` opts into a multi-scale structure-tensor rotation of source-supported existing tonal strokes; default P2-B.1 remains unchanged. CI **passed**: 41/41 Rust tests, old P2-C smoke checks and new P3-A same-binary synthetic step test ([run 37256450858](https://github.com/SanamRai001/ScanSketch/actions/runs/37256450858)); the opt-in path rotated **24 of 193** generated segments while matching the baseline stroke count and width/opacity metadata. The user has now uploaded **two first-portrait previews** from the P3-A comparison flow. Assuming the requested opening order (first P2-B.1, second P3-A), visual improvement is **not apparent**: horizontal hatching remains dominant, and central facial features may be weaker in P3-A. The user has now supplied the paired numeric table. At exactly **12,671 strokes** in both variants and effectively identical total path length (80,222.5072px vs 80,222.5070px), P3-A.0 regressed: overall tone RMSE **0.3974361→0.4017817** (+~1.09%), midtone RMSE **0.3044462→0.3113316** (+~2.26%), dark RMSE **0.4472576→0.4518669** (+~1.03%), and edge F1 **0.2887059→0.2858840** (~−0.98%). White RMSE improved slightly **0.0160805→0.0157268** (~−2.20%). Visual review also showed no clear improvement. **Decision: reject P3-A.0 rotation-only as a candidate default; retain it as evidence.** The portrait-specific `geometrically_changed_strokes` count is now confirmed as **581 of 12,671 strokes** (~4.6%). This is enough to show the P3-A.0 geometry path was materially exercised; the regression cannot be explained as a near-no-op. Genuine nonportrait P2-C gate remains open. See [P3-A.0 first visual review](P3A0_FIRST_VISUAL_REVIEW.md) and [runbook](P3A0_VERIFY.md).

- **P3-A.1 rejected:** `feat/p3a1-placement-aware-proposals`, stacked on rejected P3-A.0. CI passed 47/47, and the first portrait massively increased directional geometry (97→2367 strongly nonhorizontal strokes) at the same 12,671 total strokes, but overall tone RMSE worsened 33.13% and dark RMSE 34.44%; visual tone also degraded. Preserve [P3-A.1 result](P3A1_FIRST_PORTRAIT_RESULT.md), but do not promote it. Next: P3-A.2 hybrid structural reinforcement.

Review stack order is foundation → P1 → P2-A → P2-B → P2-B.1 → P2-C docs → P2-C metrics → rejected P3-A.0 → experimental P3-A.1. Do not merge earlier feature history by accident while collecting results.

## Why we changed course

- Tonal run fragmentation had more visible effect than successive generic Sobel refinements.
- P2-B.1 uses blur, directional nonmaximum suppression, tangent continuity ranking, dark-side source checks, tile quotas and stronger sparse contour marks; tests pass, but the key facial anchors remain insufficiently readable.
- More contour ink is not the same as better structure. Avoid judging quality from the stroke count or a single preview.
- No semantic face/eye/glasses recognition, true direction-aware tone generation, search-based candidate scoring, erasure or web UI has been built yet.

Permanent per-phase development history, evidence, outcomes and artifact names: [PHASE_EVOLUTION.md](PHASE_EVOLUTION.md); append each new phase using [PHASE_RESULT_TEMPLATE.md](PHASE_RESULT_TEMPLATE.md). Full portrait observations and caveats: [P2 visual review](P2_VISUAL_REVIEW.md). Personal portrait/source images remain local and should not be uploaded to the public repo without permission.

## P2-C deliverables and status

- **Done in this documentation phase:** define fixed fixtures, rights/provenance, identical preprocessing/seed/size, same-output naming, two distinct comparison lanes, tone/highlight/edge metrics, stroke/length complexity, runtime notes, blinded visual criteria and actual exit gate. See [P2-C protocol](P2C_PROTOCOL.md).
- **Implemented on the active branch:** public `measure_sketch` in `scansketch-core`, the `scansketch-measure` local CLI (source PNG/JPEG + output PNG + ordered stroke JSON), and `scansketch-fixtures` to generate six rights-clear synthetic images. Report metrics: linear-light tone RMSE (global and white/mid/dark), highlight ink, fixed-smoothed Sobel edge precision/recall/F1 with ±2px tolerance, and actual JSON stroke count/path length/width/opacity. Includes exact synthetic unit-test invariants. CLI mirrors the existing Triangle resize/white-matte policy and checks mismatched dimensions and bad JSON geometry. See [P2-C runbook](P2C_RUN.md).
- **Observed synthetic result:** CI's 64×64 all-white fixture generated **0 strokes**, `tone_rmse = 0.0` and `unwanted_highlight_ink_fraction = 0.0`; the CI Python sanity assertion read the output JSON and passed. Eight new metrics tests passed alongside 23 prior core tests, for 31/31. More complex photographic results remain unobserved.
- **User-verified Windows portrait execution (P2-B.1 on P2-C metrics branch):** the first paired PNG/JSON at 368×512, seed 42, produced 12,671 strokes and was successfully measured by `scansketch-measure`. Tone RMSE 0.3974361; white RMSE 0.0160805, midtone RMSE 0.3044462, dark RMSE 0.4472576; white-area unwanted-ink fraction 0.00025962. Preview/source Sobel-edge pixel counts 6,130/2,477; edge precision 0.1840131, recall 0.6697618, F1 0.2887059. Total path length ~80,222.51 working pixels. See [first real-image result](P2C_FIRST_PORTRAIT_RESULT.md) for full metrics and limitations. No source photograph or image has been checked in.
- **User-verified same-binary portrait A/B:** P2-A (`--no-contours`) produced **12,563** strokes, total path length **79,717.37px**, tone RMSE **0.39827129**, edge precision **0.1223022**, recall **0.3718208**, F1 **0.1840614**. The P2-B.1 run from the same binary (12,671 strokes, **+108**) produced tone RMSE **0.39743606**, precision **0.1840131**, recall **0.6697618**, F1 **0.2887059**. Thus the source-derived contours increased edge-proxy F1 by ~56.9% relative with +0.86% strokes / +0.63% path length; white-region RMSE and highlight ink values were identical. The portrait's subjective visual improvement remains modest. See [portrait A/B comparison](P2C_PORTRAIT_AB.md); this is **not** a matched-budget or multi-fixture proof.
- **Not done yet:** source/preview/strokes SHA-256 manifest for real example, additional seeds, matched-budget generator, signed tonal bias, performance benchmarking, second permission-cleared photograph and multi-image blinded review. **The user has now successfully generated all six rights-clear 64×64 synthetic fixture SOURCES** on Windows at `experiments/local/p2c-v1/`: white, transparent black, step, square-white-channel, gradient and thin-lines. A GitHub CI paired **step / square-white-channel / gradient** A/B batch has now also passed end-to-end: six previews, six paired stroke JSON files and six measurement reports; see [synthetic CI result](P2C_SYNTHETIC_CI_RESULT.md). The results are mixed (step F1 lower in B.1 at displayed precision; channel has some white-mask ink in both; gradient shows no measured proxy benefit at displayed precision). The user's full-precision **Windows** A/B run also succeeded across all six modes, with exact results preserved in [P2C_SYNTHETIC_WINDOWS_RESULT.md](P2C_SYNTHETIC_WINDOWS_RESULT.md). Step: RMSE 0.324933→0.316801, edge F1 0.821429→0.789474. White-channel: RMSE 0.341226→0.322536, edge F1 virtually unchanged (0.993884→0.993779), both white-mask unintended-ink fraction 0.025480769 (~53 of 2080 source-white pixels, not necessarily **inside** the channel). Gradient metrics identical across modes (RMSE 0.3867534, F1 0.0, 290 strokes). The user also supplied three screenshot views: both generated versions visibly preserve a continuous central white gap, but screenshot zoom makes exact affected pixels indeterminate; see [white-channel visual assessment and audit](P2C_WHITE_CHANNEL_AUDIT.md). The read-only diagnostic **passed on exact raw synthetic files in CI**: 0/288 affected pixels inside the protected channel in either mode; all 53 affected pixels outside it, 49 within 1px and all within 2px of source nonwhite. The earlier 2.548% figure described the **whole source-white mask**, not the channel. The exact raster antialias/geometry mechanism is not yet isolated. See [documented audit](P2C_WHITE_CHANNEL_AUDIT.md). Individual gradient edge-positive counts and permitted photo data remain pending. No full P2-C completion claim.
- Keep P1/P2-A/P2-B/P2-B.1 source branches as experimental controls.

## Second portrait A/B — classification corrected

The Windows paired photo script ran successfully on another selected `photo.jpg` at **400×512/seed 42**, but the two user-uploaded previews show a human **portrait**, NOT the planned nonportrait object. The output folder `object-results/` does not validate subject type. P2-A: 10,929 strokes, tone RMSE 0.35607746, edge F1 0.48701302; P2-B.1: 11,247 strokes (+318), RMSE 0.35345628, F1 0.66215220 (~+36% relative). White RMSE 0.02575703 and unintended-white fraction 0.001199503 were identical. Visual review: recognizably human, but predominantly horizontal hatching still swamps facial landmarks; improvement remains much smaller to the eye than the metric might suggest. Preserve original images/JSON locally. Detailed result: [P2C_SECOND_PORTRAIT_AB.md](P2C_SECOND_PORTRAIT_AB.md).

This adds **second-portrait evidence**, NOT a completed cross-subject/nonportrait gate. A genuine permission-cleared object/building photograph is still needed; avoid declaring P2-C finished solely from the misleading output path.

## P3-A.0 result — rotation-only variant rejected

An opt-in first prototype implemented [part of the P3 design](P3_DIRECTIONAL_DESIGN.md): coarse/fine gradients and structure-tensor orientation/confidence rotated selected *existing* tonal fragments around their midpoint, reverting to their exact P2-A geometry when uncertain/unsafe. Engineering validation passed, but the first controlled portrait pair **visually and numerically regressed** at the same stroke count/path length. Preserve the branch and [result](P3A0_FIRST_VISUAL_REVIEW.md), but keep P2-B.1 as the baseline. The next P3-A.1 hypothesis should change **proposal placement/allocation**, not merely rotate pre-existing horizontal anchors. P3-B candidate scoring remains deferred.

## Next execution gate

1. Done in Linux CI for all-white end-to-end and synthetic unit tests (31 passed). Next run step/channel/gradient with the actual Windows executable and record observed output hashes and scores, not merely expected values.
2. **Completed:** first same-binary portrait P2-A versus P2-B.1 numeric control recorded in [P2C_PORTRAIT_AB.md](P2C_PORTRAIT_AB.md). **Completed:** user-generated synthetic source fixtures (six PNGs). **Completed on GitHub Linux CI:** [paired synthetic batch](P2C_SYNTHETIC_CI_RESULT.md), with mixed findings. **Completed on Windows:** local [paired synthetic batch](P2C_SYNTHETIC_WINDOWS_RESULT.md) generated full-precision results and retained hashes. **Completed in CI:** [white-channel audit](P2C_WHITE_CHANNEL_AUDIT.md) against raw synthetic files, proving 0 affected pixels inside the 288-pixel protected channel for both modes. **Next:** a genuinely nonportrait paired photographic run remains outstanding. The latest paired run was actually a second **portrait** (see [Portrait B result](P2C_SECOND_PORTRAIT_AB.md)); choose and visually verify an object/architectural source before using [run-p2c-photo-ab.ps1](../scripts/run-p2c-photo-ab.ps1) with a NEW output folder. Then record the human comparison. A P3-A opt-in research design is warranted, but do not declare it validated or P2-C complete before cross-subject assessment. Individual gradient edge-positive counts and repeat seeds remain useful diagnostic follow-ups. Next run paired A/B on a permission-cleared **nonportrait** image. Do not proceed to P3-A based solely on the portrait result. Each result is retained in [phase evolution](PHASE_EVOLUTION.md).
3. Human A/B evaluation and numeric metrics; document agreements, trade-offs and failures.
4. Opt-in P3-A.0 code can be verified in parallel, but **do not declare it a quality win** until paired previews/metrics on the original portrait and genuinely nonportrait image are reviewed. Its unchanged stroke count/path-length intent is stricter than adding extra contours, but rotated raster deposition is not automatically equal. No P3-B until an honest improvement gate.

## Local safety and remaining risks

- A generated `Cargo.lock` exists on the user's Windows checkout and is not committed in this connector branch; preserve it and commit separately after review.
- `sample.jpg`, screenshots, `outputs/` and existing local stash should not be discarded/reset or committed accidentally. `--max-strokes` rejects overflow; it is **not** an equal-budget control.
- Current white-gap invariants have core regression tests. Raster antialiasing, edge score proxy and real-photo aesthetics still need broader checks.
- No erasure/residual graphite, smudging, paper texture or cloud processing until the fundamental sketch is convincing. See `FUTURE_EXPERIMENTS.md`.


## P3-A.1 implementation boundary (experimental)

P3-A.1 uses the frozen P2-A tonal generator only to derive a **target tonal stroke count** for controlled comparison, not to inherit its anchor positions. It scans 5×3 source cells, snaps candidates to real source-dark pixels near weighted darkness centroids, derives coarse/fine tangents from the existing multiscale structure tensor, applies strict unsmoothed footprint support, and sorts accepted anchors back into deterministic top-to-bottom order. Initial allocation targets are 24% coarse, 46% fine/form and the remainder tonal, with unused quota filled by the strongest remaining source-supported candidates. A visible `baseline_fallback` statistic reports any final shortfall filled from historical tonal marks.

The same P2-B.1 contour pass is appended after the new tonal field; expected total stroke count therefore matches frozen P2-B.1 on supported fixtures. This is not equal raster-ink deposition and not P3-B optimization. See [P3A1_VERIFY.md](P3A1_VERIFY.md).


### P3-A.1 first CI observation

PR #9 engineering checks passed: **47/47 Rust tests** and the full inherited workflow. On the 64×64 step, P2-B.1 and P3-A.1 both produced 193 total strokes; P3-A.1 used 17 coarse + 91 fine + 65 tonal placements with **0 baseline fallback**, and strong nonhorizontal strokes increased **20→128**. The CI summary displayed tone RMSE about **0.32→0.49**, a regression on the flat half-plane. This is recorded as a caution, not hidden: P3-A.1 is active, but its real-image artistic benefit remains unproven. See [P3A1_VERIFY.md](P3A1_VERIFY.md).


## P3-A.1 result and next architectural move

P3-A.1 succeeded in changing stroke language but failed reconstruction. With exactly 12,671 strokes, strongly nonhorizontal marks increased **97→2367**, path length rose ~3.06%, overall tone RMSE worsened **0.397436→0.529120**, midtone **0.304446→0.324213**, dark **0.447258→0.601276**, while white RMSE improved slightly **0.016081→0.015784**. The image became more directional but lost stable tonal mass. See [P3A1_FIRST_PORTRAIT_RESULT.md](P3A1_FIRST_PORTRAIT_RESULT.md).

**Next hypothesis: P3-A.2 Hybrid Structural Reinforcement.** Freeze the P2 tonal body, keep a small structural budget, and spend that budget on source-driven accents selected by structural confidence + positive tonal residual. Prefer replacing/reusing the existing P2-B.1 contour budget instead of adding unlimited strokes. P3-B optimization remains gated.


## P3-A.2 architecture freeze

Before implementation, P3-A.2 is defined to preserve the **exact P2 tonal prefix** and reuse the **existing P2-B.1 structural stroke budget**. It will rank multiscale structural candidates using **positive tonal residual + source structure confidence**; any unfilled structural slots fall back to original P2-B.1 contours. Total stroke count must therefore equal P2-B.1. See [P3A2_ARCHITECTURE.md](P3A2_ARCHITECTURE.md).


### P3-A.2 implementation

The architecture-frozen hybrid is now implemented behind `--hybrid-structural`: exact P2 tonal prefix, exact P2-B.1 total count, residual-aware multiscale structural candidates, and original-contour fallback for any unfilled structural slots. CI and portrait quality evidence remain pending until observed. See [P3A2_VERIFY.md](P3A2_VERIFY.md).


### P3-A.2 first CI result

Draft PR #10 passed [GitHub Actions run 37334525616](https://github.com/SanamRai001/ScanSketch/actions/runs/37334525616): **53/53 Rust tests**, all earlier P2-C/P3 smoke checks, and the new exact-budget hybrid step test. Step fixture: same 193 total strokes, exact 173-stroke tonal prefix, 20 structural slots, 6 hybrid-selected + 14 original-contour fallback from 55 candidates. Rounded metrics moved tone ~0.32→0.31, dark ~0.45→0.44, edge F1 ~0.79→0.80, white RMSE ~0.00→0.01; this is engineering evidence only, not a visual quality pass.


## P3-A.2 first portrait result

The first controlled portrait hybrid preserved the exact **12,563-stroke tonal prefix** and **12,671 total strokes**, but replaced **all 108 structural slots** (108 hybrid, 0 contour fallback) from 2,024 candidates. Tone RMSE improved ~0.41%, dark RMSE ~0.54%, white RMSE was identical, while midtone worsened ~1.91% and edge F1 ~0.10%; path length +0.10%. The uploaded previews are extremely similar and do not establish a clear visual win. [Exact result](P3A2_FIRST_PORTRAIT_RESULT.md).

**Decision:** hybrid architecture is promising because it preserves tone, but the replacement policy is too permissive. Next P3-A.2.1 should selectively replace baseline contours only when hybrid utility clearly exceeds baseline structural utility, with a conservative replacement cap. P2-B.1 remains default.


## P3-A.2.1 architecture freeze

P3-A.2 preserved tone but replaced all 108 portrait structural slots and produced no clear visual win. P3-A.2.1 therefore scores **baseline contours and hybrid candidates with the same utility**, replaces only when hybrid utility exceeds the weakest baseline contour by `15% + 0.001`, and caps replacements at **40%** of structural budget. Replacement occurs in the exact baseline contour slots, preserving all retained ordering and the tonal prefix. See [P3A21_ARCHITECTURE.md](P3A21_ARCHITECTURE.md).


## P3-A.2.1 implementation

Selective hybrid replacement is now implemented as a separate opt-in mode. It scores baseline contour strokes and hybrid candidates with the same residual/structure/alignment utility, replaces only when the hybrid exceeds the weakest baseline by `15% + 0.001`, and caps replacement at **40%** of structural budget. Replacements occur in the exact baseline contour slots; tonal prefix and total stroke count remain frozen. See [P3A21_VERIFY.md](P3A21_VERIFY.md).


### P3-A.2.1 CI result

Draft PR #11 passed [GitHub Actions run 37340548611](https://github.com/SanamRai001/ScanSketch/actions/runs/37340548611): **56/56 Rust tests**, all inherited checks, and the selective hybrid step smoke. Step fixture: same 193 strokes, exact 173 tonal prefix, 20 structural slots, cap 8, **6 replacements / 14 retained**, 55 candidates. This confirms the policy is selective rather than full replacement. Real portrait quality gate remains open.


## P3-A.2.1 first portrait result

P3-A.2.1 is the first P3 variant to produce a substantial structural-proxy gain without damaging the tonal body: same 12,671 strokes and exact 12,563 tonal prefix; 43/108 structural slots selectively replaced (65 retained); edge F1 **0.288706→0.332647 (+15.22%)**; tone ~0.247% better, dark ~0.301% better, white unchanged, midtone ~0.740% worse, path +0.072%. The images remain very close and changes cluster mostly in upper hair/silhouette. [Exact result](P3A21_FIRST_PORTRAIT_RESULT.md).

**Next research target:** P3-A.2.2 missing-structure residual. Score candidate value using source edge strength minus frozen baseline-preview edge strength so replacements target structure P2-B.1 actually misses, while retaining exact tone/count and conservative replacement controls.


## P3-A.2.2 architecture freeze

P3-A.2.1 improved edge F1 substantially but spent most visible changes in upper hair/silhouette. P3-A.2.2 changes only the scoring field: compute a continuous **missing-edge residual** from source-edge strength minus frozen P2-B.1 preview-edge strength, then score both baseline contours and existing hybrid candidates against that deficit. Candidate generator, 40% cap, 15%+0.001 margin, tonal prefix and total count remain frozen. See [P3A22_ARCHITECTURE.md](P3A22_ARCHITECTURE.md).


### P3-A.2.2 implementation

Missing-structure scoring is now implemented as a separate mode: continuous edge strength is factored from the unchanged P2-C Sobel family; frozen P2-B.1 is rendered once; `max(source_edge - baseline_edge, 0)` becomes the new structural-deficit field. The P3-A.2.1 candidate generator, cap, replacement margin, prefix/count invariants and slot substitution are unchanged. Engineering CI now passes **59/59 Rust tests** plus the missing-structure smoke; real-image outcome remains pending. See [P3A22_VERIFY.md](P3A22_VERIFY.md).


### P3-A.2.2 first CI evidence

Draft PR #12 passed [GitHub Actions run 37345281232](https://github.com/SanamRai001/ScanSketch/actions/runs/37345281232): **59/59 Rust tests**, all inherited checks, and the new missing-structure smoke. Step fixture: same 193 total strokes, exact 173-stroke tonal prefix, 20 structural slots, cap 8, **6 replacements / 14 retained**, 55 candidates; weakest baseline utility 0.120863, strongest hybrid utility 0.366960, mean missing-edge evidence 0.126235. This confirms only the scoring field changed while the selective controls stayed fixed. Portrait quality gate remains open.


## P3-A.2.2 first portrait result

P3-A.2.2 is a clean negative result. At exact total-count/tone-prefix controls, overall tone improved only ~0.042% and dark ~0.055%, midtone worsened ~0.195%, white stayed identical, path changed only +0.009%, while edge F1 **fell ~0.99% (0.288706→0.285843)**. The uploaded pair does not materially recover weak interior structure. [Exact result](P3A22_FIRST_PORTRAIT_RESULT.md).

**Interpretation:** re-ranking the existing P3-A.2.1 candidate pool is insufficient. The pool is still created from positive tonal residual/dark anchors, so underrepresented edges in lighter/midtone regions may never become candidates. Next: P3-A.2.3 deficit-driven candidate proposals from missing-edge local maxima, retaining the same 40% cap, replacement margin, exact tonal prefix and total stroke count.


## P3-A.2.3 architecture freeze

After P3-A.2.2 showed that missing-edge **scoring** cannot recover structures absent from the proposal pool, P3-A.2.3 changes one variable: the existing short-stroke proposal machinery is driven by **missing-edge residual instead of positive tonal residual**. Exact tonal prefix, total count, 40% cap, 15%+0.001 margin, tensor direction, support checks, spatial fairness and P3-A.2.2 utility remain frozen. See [P3A23_ARCHITECTURE.md](P3A23_ARCHITECTURE.md).


### P3-A.2.3 implementation

The deficit-driven proposal experiment is now implemented as a separate opt-in mode. It reuses the existing P3-A.2 short-stroke proposal machinery but feeds **missing-edge residual** into candidate anchor/local-search selection. P3-A.2.2 scoring, 40% cap, 15%+0.001 margin, exact tonal prefix, exact total count, tensor direction, support checks and in-place replacement remain unchanged. CI/portrait outcome pending. See [P3A23_VERIFY.md](P3A23_VERIFY.md).


### P3-A.2.3 first CI evidence

Draft PR #13 passed [GitHub Actions run 37348529191](https://github.com/SanamRai001/ScanSketch/actions/runs/37348529191): **62/62 Rust tests**, all inherited checks, and the deficit-driven proposal smoke. Step fixture: same 193 total strokes, exact 173 tonal prefix, budget 20, cap 8, **6 replacements / 14 retained**. The proposal pool shrank from P3-A.2.2's 55 candidates to **26 deficit-driven candidates**, while mean missing-edge evidence at accepted replacements rose slightly **0.126235→0.127317**. Portrait quality/localization is the next gate.


## P3-A.2.3 first portrait result

The controlled portrait A/B shows a **promising but provisional** result: same 12,671 strokes, white RMSE unchanged, path +0.0601%, edge F1 **+8.918%**, dark RMSE ~0.0109% better, while tone ~0.0380% and midtone ~0.9357% worsen. Visual difference remains subtle and stable. P3-A.2.1 still has the higher global portrait edge-F1 result, so P3-A.2.3 is not called the numeric winner; its significance is evidence that changing proposal origin can recover structural signal without tonal collapse. [Exact result](P3A23_FIRST_PORTRAIT_RESULT.md).

**Next:** freeze P3-A.2.3 and run P3-G1 multi-image genuine nonportrait validation. No additional renderer tuning until that evidence exists.


## P3-G1 generalization gate opened

P3-A.2.3 is frozen after a promising portrait result. New branch `test/p3g1-nonportrait-generalization` adds **validation infrastructure only**, not renderer changes: a 3–5 image local batch harness that runs frozen P2-B.1 vs P3-A.2.3, verifies prefix/count/slot invariants, aggregates metrics, writes CSV/JSON, and produces a local side-by-side `review.html`. See [P3-G1 protocol](P3G1_PROTOCOL.md) and [runbook](P3G1_RUN.md).


### P3-G1 harness CI result

Draft PR #14 validation tooling passed [GitHub Actions run 37354095490](https://github.com/SanamRai001/ScanSketch/actions/runs/37354095490), including the 3-case batch smoke and aggregate JSON/CSV/HTML outputs. The smoke intentionally showed mixed/no-op behavior (step 6 replacements; thin-lines 0; gradient no structural budget), confirming the harness does not force a difference. **Real 3–5 photo nonportrait evidence is still pending.**


## P3-G1 first real nonportrait result

Chair/mug/plant validation does **not** support promoting P3-A.2.3: edge wins 1/3, tone 0/3, midtone 0/3, dark 1/3; exploratory mean edge change -2.137%, tone -0.596%, midtone -1.869%, dark -0.365%. Visual review revealed the larger issue: both P2-B.1 and P3-A.2.3 still look like scanline reconstructions. Current P2 tonal marks are only **2.0–10.5 px** and P3 structural candidates **2.6–7.2 px** at a 512px work image. Long human-like gestural strokes have **not** been implemented. [Exact P3-G1 result](P3G1_FIRST_RESULT.md).

**Current direction:** stop tuning P3 edge-selection heuristics. Begin P4-A human stroke hierarchy: long structural/gestural paths + medium form strokes + short hatching, with long curved marks as first-class editable records.


## P4-A human stroke hierarchy opened

P3-G1 confirms the renderer's dominant bottleneck is now its primitive vocabulary: P2 tonal fragments are only **2.0–10.5px** and P3 structural candidates **2.6–7.2px** at working scale. These are useful hatching marks but are not long human gestures. P4-A therefore redesigns the mark hierarchy around **long structural paths + medium form paths + short residual hatching**, with long curved paths as first-class editable records rather than chains of unrelated segments. See [P4-A architecture](P4A_HUMAN_STROKE_HIERARCHY.md).


### P4-A.0 implementation opened

First-class logical path support is being added without changing historical renderers. `Sketch.paths` is backward-compatible/omitted when empty; `PathStroke` stores a deterministic polyline + generic role; tiny-skia renders it continuously; metrics expose path stats plus combined logical-mark counts while preserving legacy `strokes` semantics. No automatic long-stroke tracing yet. See [P4-A.0 verification](P4A0_VERIFY.md).


### P4-A.0 CI result

P4-A.0 passed [CI run 37405989296](https://github.com/SanamRai001/ScanSketch/actions/runs/37405989296): **69/69 Rust tests** plus every historical smoke. The seven-point path fixture is stored/measured as **one logical gesture**, length **82.87px**, with legacy `strokes.count=0` and `paths.count=1`. Backward compatibility is therefore proven for the foundation slice. **Next: P4-A.1 source-driven long structural path tracing.**


## P4-A.1 long structural tracing opened

Source-driven long gestures are now implemented as an opt-in P4 experiment. Seeds come from source edge corridors, direction comes from the existing multiscale tensor, forward/backward traces snap to edge support, and accepted paths must be at least ~24px and remain sparse. Two outputs are supported: exact P2-B.1 + paths, and paths-only for direct structural inspection. The old hatch field is deliberately unchanged until P4-A.3. See [P4-A.1 verification](P4A1_VERIFY.md).


### P4-A.1 CI result

P4-A.1 passed [CI run 37406856611](https://github.com/SanamRai001/ScanSketch/actions/runs/37406856611): **74/74 Rust tests** plus every inherited smoke. On the vertical step it generated **2** first-class Gesture paths, mean **48.5px**, max **49px**, total **97px**, while preserving all 193 frozen P2-B.1 segments exactly; paths-only has zero legacy fragments. **Next gate is real chair/mug/plant paths-only visual review.**


### P4-A.1 real-pack harness

A one-command multi-image harness now reuses the prior 3–5 nonportrait source pack and generates **P2-B.1 / P4-A.1 overlay / P4-A.1 paths-only** views per image, aggregate path-length/count evidence, CSV/JSON, and a local three-column review page. Zero accepted paths is treated as valid evidence rather than a script failure. The existing chair/mug/plant folder can be reused directly. [Runbook](P4A1_RUN.md).


### P4-A.1 pack harness CI result

The complete three-view pack harness passed [CI run 37408490425](https://github.com/SanamRai001/ScanSketch/actions/runs/37408490425), including zero-path sources. Synthetic behavior: step 2 paths at 48.5px mean; thin-lines 4 paths at 47px; gradient 0 paths. This verifies that real-image validation can distinguish useful path discovery from legitimate no-op cases without forcing marks. **Real chair/mug/plant review is now the only P4-A.1 gate left.**
