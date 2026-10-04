# Project State

Last updated: **2026-10-04**. Single authoritative progress record; Git history, verified CI and visually inspected outputs supersede phase plans.

## Vision

ScanSketch reconstructs images using actual bounded ordered pencil strokes, not grayscale painting. The tonal identity is top-to-bottom sampling. Improvements must be compared against the same input, seed and output size. The initial subject portrait is a research fixture; do not commit the personal photograph or generated outputs without permission.

## Branches / stacked review

- `main`: repository bootstrap only; nothing merged.
- `docs/foundation`: draft PR #1.
- `feat/p1-rust-scanline-baseline`: draft PR #2 into foundation; Windows workspace check, 11/11 core tests and first CLI rendering passed, but continuous horizontal bars looked like engraving.
- `feat/p2-sketch-stroke-language`: draft PR #3 into P1; CI passed (14/14 tests). Fragmented tone reduced long stripe artifacts, but weak face anchors remained.
- `feat/p2b-contour-reinforcement`: draft PR #4 into P2-A; CI passed (20/20 tests). User rendered same original photograph at 368x512 / seed 42: P2-A control **12,563** strokes, P2-B **13,024** (+461). Two images were visually reviewed: difference subtle, glasses/eyes/lips still weak and hair texture prominent. Retain this branch as reference.
- **Active:** `feat/p2b1-contour-coherence`: P2-B.1, branched from P2-B; target P2-B via separate draft PR. Previous branches remain unmodified.

## P2-B.1 implementation

Motivation: P2-B scans candidates from top-left, so fine texture can occupy positions before a stronger nearby contour. Raw Sobel also spends energy on small texture; just increasing opacity is insufficient.

- Apply a mild separable 3x3 Gaussian/binomial smoothing to darkness **for gradient estimation only**.
- Compute Sobel gradient on the smoothed map and suppress local nonmaxima.
- Confirm neighboring gradient alignment along the contour tangent; reject isolated low-coherence candidates.
- Deterministically rank the surviving candidates by gradient strength + tangent continuity, with row-major tie-breaking.
- Use small local occupancy spacing and a 32px-tile quota so one textured neighborhood cannot claim the entire global contour budget.
- Increase structural stroke confidence moderately and retry shorter stroke lengths if longer ones cross a source highlight.
- Validate line, rounded ends and both lateral sides against **unsmoothed** darkness; continue to protect true white gaps, canvas bounds and global stroke budget.
- Keep P2-A tone generation and its RNG unchanged; CLI `--no-contours` remains an exact tonal-only control. CLI retains `--contour-threshold` (default 0.28). No new dependencies or stroke-schema changes.

**Verification:** P2-B.1 GitHub Actions `cargo check --workspace` and `cargo test --workspace` passed (**23/23 tests, 0 failed**), run https://github.com/SanamRai001/ScanSketch/actions/runs/37217765227. Its same-input portrait comparison is still pending; do not claim visual improvement until the image is reviewed. This is a generic nonsemantic edge procedure: there is no understanding of faces, spectacles or lips yet. It may still reinforce long hair strands. Do not jump to optimization based on a theoretical benefit.

## Next gate

Follow `docs/P2B1_VERIFY.md` and preserve outputs: `sample-p2a.png`, `portrait-p2a-control.png`, `portrait-p2b-contours.png`. Compare the saved P2-B result with the new `portrait-p2b1-ranked.png` on the same original. Also test a simple object, white-only fixture and high-contrast object, plus repeatability (identical seed, same output hashes).

If P2-B.1 produces a similarly subtle difference, stop piling on contour heuristics. Move toward P2-C measurement (tone, structural fidelity and edge preservation) and research directional stroke placement, multiscale analysis or a better objective instead of endlessly darkening more Sobel pixels. Primitive-inspired optimization stays gated. No erasure, smudge or graphite residue until the fundamental renderer is satisfying.

## Development notes

`Cargo.lock` currently exists locally on the user's Windows checkout but is not yet committed; preserve and review it. Existing local stash and `sample.jpg` must not be reset or accidentally committed. GitHub CI validates Linux Rust; local Windows and visual checks remain a separate gate.
