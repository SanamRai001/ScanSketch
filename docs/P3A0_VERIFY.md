# P3-A.0 — Opt-in Direction-aware Tonal Proposals

**Status:** first testable research implementation; **not** P3-A completed or accepted. P2-C still requires a confirmed genuinely nonportrait photographic source. Use the current staged PR branch `feat/p3a-directional-tonal-prototype`; parent `feat/p2c-measurement-utility` remains unmodified.

## Hypothesis and what actually changed

The same dominant horizontal short marks seen in two portraits may be improved by modifying **tonal stroke directions**, not by adding another overlay. The baseline P2-A tonal proposals still establish anchor order, number of marks, width, opacity and original path length. In the `--directional` mode only:

1. Compute the existing unsmoothed white-matted linear-darkness map.
2. Derive **fine** (one separable [1,2,1] binomial pass, roughly sigma 0.71px) and **coarse** (eight passes, roughly sigma 2px) darkness scales, at working resolution.
3. Calculate Sobel gradients, smooth their outer-product **structure tensor**, and estimate a tangent to coherent source structure. Gate uncertain or weak directions using separate confidence/signal thresholds and small local search.
4. Sweep the original P2-A stroke array **in its existing top-to-bottom anchor order**; rotate selected segments *about their midpoints* (same length/width/opacity) only if their projected rounded-cap footprint is supported by the **original, unsmoothed** target. If not, retain the exact original segment. Bounded rotation: at most 28% tonal count and 12 per 32×32px tile.
5. Append the **unchanged** optional P2-B.1 contour pass after tonal marks, with the original separate RNG and budget. Rotation never adds extra strokes in this first experiment.

This deliberately tests directional geometry instead of inventing new shade/ink density. **The same stroke count/path length does not guarantee equal raster-deposited darkness after rotation**, so report both the actual count and path length. Drawing order refers to the original stroke **anchor** bands; rotated segments may extend past their anchor band. This is an **orientation-guided transformation of current proposals**, not yet an independent placement scheduler, residual optimizer, face detector, fine texture hierarchy, or true matched-raster-ink experiment.

## Invariants

- CLI without `--directional`: exactly original P2-B.1, including output names, original JSON schema and seeded RNG. No existing CLI flags changed.
- With `--directional`: same original tonal count, stroke order, width/opacity, no new schema/dependencies. Stable original contours, if enabled.
- If source is flat, white or lacks reliable supported direction: fallback to exact baseline marks.
- All proposed rotated marks must stay inside image and have centerline, round-cap extension and both lateral pen sides supported by unblurred target darkness. Reject a rotation instead of clipping into a white gap.
- Bounds: one 1024px maximum side, existing `max_strokes` overflow behavior, no erasure/semantic recognition/optimization.
- Exact source/seed/size must match between A/B; use existing `p2c-v1` metric constants. No P2-C retroactive score changes.

## CI-first synthetic check

Workspace compilation, inherited test suite and added checks for flat-image fallback, vertical step orientation, equal count/opacity/width/path length, deterministic repeatability, clear white internal channel, empty paper and budget handling.

The checked-in `scripts/run-p3a-ab.ps1` is smoke-tested in GitHub CI against the existing synthetic `step.png`. It runs P2-B.1 and `--directional` with the **same binary**, seed 42, max-side 64; exports paired PNG/JSON/report, refuses to overwrite files, records SHA-256 identities and verifies stroke-count and ink metadata equality. **A successful script execution is not proof of better quality.** Check the commit's CI before visual review.

## Same-portrait A/B on Windows (after green CI)

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p3a-directional-tonal-prototype
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a-ab.ps1" -InputImage ".\sample.jpg" -OutputDir "experiments/local/p3a/portrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

If a tracked local branch already exists, `git switch feat/p3a-directional-tonal-prototype; git pull --ff-only`. Your original `sample.jpg`, untracked `Cargo.lock`, `outputs/` and prior `experiments/local/` must remain untouched. Only choose `-RightsConfirmed` for inputs that may be locally processed; never upload personal sources to the public repo.

The command saves `p2b1.png`, `p3a.png`, corresponding `*-strokes.json`, `*-report.json`, and `summary.json`. Compare **both** at 100%, including facial landmarks, hair texture, white gap/background, and whether the new directions look intentional rather than patchy. Share the anonymized metric table and images only when permission allows. Count `geometrically_changed_strokes` in the manifest; if it is zero on the portrait, that is a failed/ineffective experimental parameterization, not a success.

## Nonportrait gate is still open

Our earlier `object-results/` turned out to show another *person*, not an object. For a second, actually nonportrait input, visually confirm a mug, chair, bicycle or building detail before passing its real file-picker path. Use a **NEW** directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a-ab.ps1" -InputImage $nonportrait -OutputDir "experiments/local/p3a/nonportrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

Also execute the P2-C tonal-only versus P2-B.1 photo script on that nonportrait subject to close the previous cross-category requirement; its outcomes are not retroactively supplied by P3-A. At least two input classes, actual preview inspection and the previous p2c-v1 metrics are necessary before claiming an improvement. Record numerical and visual regressions in [PHASE_EVOLUTION.md](PHASE_EVOLUTION.md).

**Next design gate:** if directional rotation changes too few strokes or yields distracting edge-aligned hatching, measure why: confidence gate, source-support rejections and tile quota. Revise a falsifiable P3-A.1 placement proposal instead of blindly boosting angular density. P3-B candidate optimization remains deferred.
