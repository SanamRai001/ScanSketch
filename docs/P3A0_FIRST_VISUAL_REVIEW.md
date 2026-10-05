# P3-A.0 — First Portrait Visual A/B (provisional)

**Date:** 2026-10-05. **Status:** two user-uploaded preview images inspected; **the actual paired `summary.json`, per-mode `*-report.json`, source hashes and numerical metrics have not yet been supplied**. This is a qualitative finding only, not a final reject/accept decision. The input is the original locally held private portrait and must not be checked into the public repository without explicit redistribution permission.

## Identity and context

- Baseline: P2-B.1 (`generate_sketch`, contours enabled).
- Experiment: P3-A.0 (`--directional`, bounded fine/coarse tensor reorientation of existing tonal marks plus identical contour algorithm).
- Intended identical source/seed/size from `scripts/run-p3a-ab.ps1`; **actual local run manifest still needed** to verify the exact source hash, working dimensions, stroke counts and geometry changes.
- Two images were uploaded in chat after the instructions to open `experiments/local/p3a/portrait-a/p2b1.png` then `.../p3a.png`. **Assuming upload order matches that open order**, the first image is P2-B.1 and second P3-A.0. They appear at different viewer framing/zoom; preserve this caveat.
- The upstream [GitHub CI run](https://github.com/SanamRai001/ScanSketch/actions/runs/37256450858) verified 41/41 Rust tests and confirmed 24/193 geometries changed on a **synthetic step** with identical stroke count/ink metadata. That number **is not** the portrait's changed-stroke count.

## Visual comparison

- Both images show recognizable broad hair/face silhouette, large light horizontal eyewear/face region, white background and a large mass of repeated fine hatching.
- Directional version appears somewhat softer, but the dense horizontal tonal rhythm is still dominant. There is **no clear perceived improvement** to the facial anchors (eyewear, eyes, nose/lips), and some central structure looks weaker at this screenshot scale.
- The visible improvement, if any, is **not** enough to call P3-A.0 a better sketch. This is an inconclusive-to-negative first artistic trial of **reorienting a subset of existing marks**. Do not quietly promote the P3-A.0 output as the new default or overwrite P2-B.1.
- Mechanistic hypothesis (not yet a measured cause): rotating an already-selected horizontal tonal fragment cannot redesign its initial anchor locations, band spacing, overlap or coarse/form-vs-texture budget. Conservative source-footprint support and per-tile quotas may also leave most marks unchanged. This suggests a future P3-A.1 **independent direction-aware proposal placement** rather than just boosting rotation opacity/thresholds.
- A different screenshot zoom/frame prevents reliable assessment of tiny facial feature differences. The actual images at 100% plus changed-stroke count and `p2c-v1` metrics should govern the decision.

## Next evidence requested — NO new renderer changes yet

The existing script already saved `experiments/local/p3a/portrait-a/summary.json` and paired reports (if it printed its success line). Share only these **nonprivate fields**:

```powershell
$s = Get-Content ".\experiments\local\p3a\portrait-a\summary.json" -Raw | ConvertFrom-Json
"Changed stroke geometries: $($s.geometrically_changed_strokes)"
$s.fixtures | Select-Object mode,width,height,strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink | Format-Table -AutoSize
```

The actual source photograph, SHA-256 hashes and private local filesystem paths need not be shared. The standard paired reports live beside `summary.json`, and the experiment script rejects overwriting existing output folders. Compare the original full-resolution `p2b1.png` and `p3a.png` at the same 100% scale if uncertain.

## Decision gate

**Retain P2-B.1 by default. Keep P3-A.0 as a documented experiment.** Depending on metrics and number of actual rotations:
- If rotations are very few, instrument rejection reasons/confidence/footprint checks before redesigning.
- If rotations are substantial but appearance is still mechanical, test a truly direction-aware source-based proposal/placement approach (P3-A.1) with controlled tonal/structural budgets, preserving the current output as baseline.
- If numerical metrics regress or white protection deteriorates, reject this rotation-only configuration rather than amplifying it.
- The **genuine nonportrait photographic P2-C gate is still outstanding**; a previous `object-results/` test turned out to be another portrait. Neither prototype can be accepted across source classes until that gap is closed.

Record the numeric follow-up as an append to this same phase entry rather than rewriting the original provisional observation.
