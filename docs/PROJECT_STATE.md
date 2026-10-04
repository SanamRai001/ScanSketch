# Project State

Last updated: **2026-10-04**. This is the single authoritative progress record; Git/CI, actual run outputs and observed visuals override planning documents.

## Goal and current decision

ScanSketch is a deterministic, CPU-first Rust system for rebuilding images from **actual ordered pencil strokes**, not grayscale pixel painting. Top-to-bottom sampling remains the design identity. After the first portrait iterations, **P2-B.1 is the current strongest subjective visual candidate** but is still not a convincing finished portrait renderer or an objectively established winner.

**Current active phase: P2-C — measurement and next-algorithm design (documentation/protocol committed; measurement implementation and numeric results pending).** Stop stacking additional raw-Sobel contour heuristics without controlled evidence. The leading next prototype to test is multi-scale, direction-aware source-based stroke placement (P3-A), with Primitive-inspired scoring (P3-B) gated behind measurements.

## Stacked GitHub review

- `main`: initial bootstrap only. No branch merges, release or deployment.
- `docs/foundation`: draft PR #1.
- `feat/p1-rust-scanline-baseline`: draft PR #2 into foundation. User's Windows build/check and 11/11 core tests passed; first source/photo preview looked mechanically striped.
- `feat/p2-sketch-stroke-language`: draft PR #3 into P1. GitHub CI 14/14; fragmented marks helped compared with continuous horizontal bars.
- `feat/p2b-contour-reinforcement`: draft PR #4 into P2-A. GitHub CI 20/20. Same-image CLI at 368x512, seed 42: no contours 12,563 strokes, contour-on 13,024 (+461), visually modest structural benefit.
- `feat/p2b1-contour-coherence`: draft PR #5 into P2-B. GitHub CI 23/23. Same portrait preview received and qualitatively reviewed: strongest candidate so far, still an incremental improvement with weak glasses/eyes/lips and hair dominant.
- **Active:** `docs/p2c-measurement-and-p3-design`, branched from P2-B.1; documentation-only changes, intended to target P2-B.1 via a new stacked draft PR. Keeps all baseline engines intact.

Review stack order is foundation → P1 → P2-A → P2-B → P2-B.1 → P2-C docs. Do not merge earlier feature history by accident while collecting results.

## Why we changed course

- Tonal run fragmentation had more visible effect than successive generic Sobel refinements.
- P2-B.1 uses blur, directional nonmaximum suppression, tangent continuity ranking, dark-side source checks, tile quotas and stronger sparse contour marks; tests pass, but the key facial anchors remain insufficiently readable.
- More contour ink is not the same as better structure. Avoid judging quality from the stroke count or a single preview.
- No semantic face/eye/glasses recognition, true direction-aware tone generation, search-based candidate scoring, erasure or web UI has been built yet.

Full observed evidence and caveats: [P2 visual review](P2_VISUAL_REVIEW.md). Personal portrait/source images remain local and should not be uploaded to the public repo without permission.

## P2-C deliverables and status

- **Done in this documentation phase:** define fixed fixtures, rights/provenance, identical preprocessing/seed/size, same-output naming, two distinct comparison lanes, tone/highlight/edge metrics, stroke/length complexity, runtime notes, blinded visual criteria and actual exit gate. See [P2-C protocol](P2C_PROTOCOL.md).
- **Not done yet:** implementing/running the measurement command, synthetic fixture generation, hashing/manifest for real examples, matched-budget generator, numerical outputs, multi-image blinded review. No invented scores or benchmark result.
- Keep P1/P2-A/P2-B/P2-B.1 source branches as experimental controls.

## Next major algorithm decision (proposed, not coded)

Study [P3 multi-scale direction-aware design](P3_DIRECTIONAL_DESIGN.md). P3-A would use coarse/fine gradients and structure-tensor orientation/confidence to propose source-supported short strokes along local geometry during the existing top-to-bottom sweep, reverting to P2-A tonal marks when direction is ambiguous. Avoid semantic portrait hacks. P3-B would later apply bounded Primitive-style candidate scoring if P3-A provides genuine wins at controlled resources.

## Next execution gate

1. Implement a small reproducible P2-C measurement tool and synthetic fixtures; first verify it on known white/shape/gradient inputs.
2. Run equivalent recorded portrait and a permission-cleared nonportrait object through all preserved versions, preserving seeds, actual resized dimensions, JSON strokes, SHA-256 and build profile.
3. Human A/B evaluation and numeric metrics; document agreements, trade-offs and failures.
4. Only then prototype P3-A behind an opt-in mode and compare at controlled ink/stroke budgets. Do not start P3-B just because the new geometry is interesting.

## Local safety and remaining risks

- A generated `Cargo.lock` exists on the user's Windows checkout and is not committed in this connector branch; preserve it and commit separately after review.
- `sample.jpg`, screenshots, `outputs/` and existing local stash should not be discarded/reset or committed accidentally. `--max-strokes` rejects overflow; it is **not** an equal-budget control.
- Current white-gap invariants have core regression tests. Raster antialiasing, edge score proxy and real-photo aesthetics still need broader checks.
- No erasure/residual graphite, smudging, paper texture or cloud processing until the fundamental sketch is convincing. See `FUTURE_EXPERIMENTS.md`.
