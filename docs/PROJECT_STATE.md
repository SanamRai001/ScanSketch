# Project State

Last updated: **2026-10-04**. This is the single authoritative progress record; Git/CI, actual run outputs and observed visuals override planning documents.

## Goal and current decision

ScanSketch is a deterministic, CPU-first Rust system for rebuilding images from **actual ordered pencil strokes**, not grayscale pixel painting. Top-to-bottom sampling remains the design identity. After the first portrait iterations, **P2-B.1 is the current strongest subjective visual candidate** but is still not a convincing finished portrait renderer or an objectively established winner.

**Current active phase: P2-C — measurement implementation and real-fixture validation.** The first independent measurement utility is implemented and verified in GitHub Actions; numeric comparisons of the actual portrait and other photo fixtures have **not** yet been performed. Stop stacking additional raw-Sobel contour heuristics without controlled evidence. The leading next prototype to test is multi-scale, direction-aware source-based stroke placement (P3-A), with Primitive-inspired scoring (P3-B) gated behind measurements.

## Stacked GitHub review

- `main`: initial bootstrap only. No branch merges, release or deployment.
- `docs/foundation`: draft PR #1.
- `feat/p1-rust-scanline-baseline`: draft PR #2 into foundation. User's Windows build/check and 11/11 core tests passed; first source/photo preview looked mechanically striped.
- `feat/p2-sketch-stroke-language`: draft PR #3 into P1. GitHub CI 14/14; fragmented marks helped compared with continuous horizontal bars.
- `feat/p2b-contour-reinforcement`: draft PR #4 into P2-A. GitHub CI 20/20. Same-image CLI at 368x512, seed 42: no contours 12,563 strokes, contour-on 13,024 (+461), visually modest structural benefit.
- `feat/p2b1-contour-coherence`: draft PR #5 into P2-B. GitHub CI 23/23. Same portrait preview received and qualitatively reviewed: strongest candidate so far, still an incremental improvement with weak glasses/eyes/lips and hair dominant.
- `docs/p2c-measurement-and-p3-design`: draft PR #6 into P2-B.1; protocol, visual evidence and proposed P3 architecture; CI passed 23/23.
- **Active:** `feat/p2c-measurement-utility`, draft PR #7 into P2-C docs. New metrics and synthetic fixture tooling; existing reconstruction logic unchanged. CI passed: **31/31 Rust tests**, `cargo check --workspace`, and an actual synthetic white-source fixture → old rendering CLI → new metrics CLI smoke. GitHub run: https://github.com/SanamRai001/ScanSketch/actions/runs/37219282372.

Review stack order is foundation → P1 → P2-A → P2-B → P2-B.1 → P2-C docs → P2-C metrics. Do not merge earlier feature history by accident while collecting results.

## Why we changed course

- Tonal run fragmentation had more visible effect than successive generic Sobel refinements.
- P2-B.1 uses blur, directional nonmaximum suppression, tangent continuity ranking, dark-side source checks, tile quotas and stronger sparse contour marks; tests pass, but the key facial anchors remain insufficiently readable.
- More contour ink is not the same as better structure. Avoid judging quality from the stroke count or a single preview.
- No semantic face/eye/glasses recognition, true direction-aware tone generation, search-based candidate scoring, erasure or web UI has been built yet.

Full observed evidence and caveats: [P2 visual review](P2_VISUAL_REVIEW.md). Personal portrait/source images remain local and should not be uploaded to the public repo without permission.

## P2-C deliverables and status

- **Done in this documentation phase:** define fixed fixtures, rights/provenance, identical preprocessing/seed/size, same-output naming, two distinct comparison lanes, tone/highlight/edge metrics, stroke/length complexity, runtime notes, blinded visual criteria and actual exit gate. See [P2-C protocol](P2C_PROTOCOL.md).
- **Implemented on the active branch:** public `measure_sketch` in `scansketch-core`, the `scansketch-measure` local CLI (source PNG/JPEG + output PNG + ordered stroke JSON), and `scansketch-fixtures` to generate six rights-clear synthetic images. Report metrics: linear-light tone RMSE (global and white/mid/dark), highlight ink, fixed-smoothed Sobel edge precision/recall/F1 with ±2px tolerance, and actual JSON stroke count/path length/width/opacity. Includes exact synthetic unit-test invariants. CLI mirrors the existing Triangle resize/white-matte policy and checks mismatched dimensions and bad JSON geometry. See [P2-C runbook](P2C_RUN.md).
- **Observed synthetic result:** CI's 64×64 all-white fixture generated **0 strokes**, `tone_rmse = 0.0` and `unwanted_highlight_ink_fraction = 0.0`; the CI Python sanity assertion read the output JSON and passed. Eight new metrics tests passed alongside 23 prior core tests, for 31/31. More complex photographic results remain unobserved.
- **Not done yet:** source/preview/strokes SHA-256 manifest for real examples, actual portrait numerical scores, matched-budget generator, performance benchmarking, multi-image blinded review. Do not invent missing results.
- Keep P1/P2-A/P2-B/P2-B.1 source branches as experimental controls.

## Next major algorithm decision (proposed, not coded)

Study [P3 multi-scale direction-aware design](P3_DIRECTIONAL_DESIGN.md). P3-A would use coarse/fine gradients and structure-tensor orientation/confidence to propose source-supported short strokes along local geometry during the existing top-to-bottom sweep, reverting to P2-A tonal marks when direction is ambiguous. Avoid semantic portrait hacks. P3-B would later apply bounded Primitive-style candidate scoring if P3-A provides genuine wins at controlled resources.

## Next execution gate

1. Done in Linux CI for all-white end-to-end and synthetic unit tests (31 passed). Next run step/channel/gradient with the actual Windows executable and record observed output hashes and scores, not merely expected values.
2. Run equivalent recorded portrait and a permission-cleared nonportrait object through all preserved versions, preserving seeds, actual resized dimensions, JSON strokes, SHA-256 and build profile.
3. Human A/B evaluation and numeric metrics; document agreements, trade-offs and failures.
4. Only then prototype P3-A behind an opt-in mode and compare at controlled ink/stroke budgets. Do not start P3-B just because the new geometry is interesting.

## Local safety and remaining risks

- A generated `Cargo.lock` exists on the user's Windows checkout and is not committed in this connector branch; preserve it and commit separately after review.
- `sample.jpg`, screenshots, `outputs/` and existing local stash should not be discarded/reset or committed accidentally. `--max-strokes` rejects overflow; it is **not** an equal-budget control.
- Current white-gap invariants have core regression tests. Raster antialiasing, edge score proxy and real-photo aesthetics still need broader checks.
- No erasure/residual graphite, smudging, paper texture or cloud processing until the fundamental sketch is convincing. See `FUTURE_EXPERIMENTS.md`.
