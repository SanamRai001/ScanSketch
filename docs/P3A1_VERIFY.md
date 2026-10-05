# P3-A.1 — Placement-aware directional proposals

**Status:** implementation on `feat/p3a1-placement-aware-proposals` passed GitHub CI engineering checks: **47/47 Rust tests**, all inherited P2-C/P3-A.0 smoke checks, and the new P3-A.1 paired step run. Real-photo quality results remain unobserved; no win is claimed. P2-B.1 remains the frozen baseline. P3-A.0 remains preserved as a rejected rotation-only experiment.

## Hypothesis

P3-A.0 changed **581/12,671** portrait stroke geometries but still worsened tone/midtone/dark RMSE and edge F1. Therefore the next test changes **where tonal strokes are proposed**, not merely their angle.

P3-A.1:
- obtains the P2-A tonal count only as a comparison budget;
- never reuses P2-A tonal anchor positions unless a last-resort sparse-input fallback is needed;
- scans small 5×3 source cells, snaps each anchor to an actual source-dark pixel near that cell's weighted darkness centroid, and adds deterministic subpixel jitter;
- chooses coarse/fine orientation from the existing multiscale structure tensor; ambiguous cells use a mild seeded tonal angle prior;
- makes a second offset proposal only in genuinely dark cells;
- enforces strict unsmoothed source-support over centerline, round-cap extension and both lateral pen sides;
- allocates explicit initial proposal targets: **24% coarse structure, 46% fine/form, remainder tonal**, then fills unused quota from the best remaining source-supported candidates;
- if a pathological/sparse input cannot supply the tonal comparison budget, fills the final shortfall with exact baseline tonal strokes and reports the fallback count through `PlacementStats`;
- sorts selected proposals back into deterministic top-to-bottom anchor order;
- appends the unchanged P2-B.1 contour pass afterward.

This is **not** P3-B optimization, a face detector, curves, erasure or a learned perceptual model.

## CLI

Default remains P2-B.1.

- `--directional`: historical/rejected P3-A.0 rotation-only experiment.
- `--placement-aware`: P3-A.1.
- The two experiment flags are mutually exclusive.

P3-A.1 prints placement statistics after generation:

```text
target tone
selected coarse
selected fine
selected tonal
baseline fallback
generated candidate count
```

A high fallback fraction is a failed parameterization, not a success.

## Engineering gates

New core tests require:
1. blank source remains blank;
2. P3-A.1 has the same **total** stroke count as P2-B.1 on the step fixture;
3. source-driven directional/form proposals are actually selected;
4. deterministic repeatability;
5. protected 288px white channel stays clear;
6. directly generated nonhorizontal structure exists;
7. existing contour overflow behavior remains enforced.

GitHub CI also runs `scripts/run-p3a1-ab.ps1` against the rights-clear 64×64 step source. The script requires equal total stroke count and more strongly nonhorizontal strokes than frozen P2-B.1.

## Windows portrait A/B after green CI

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p3a1-placement-aware-proposals
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace

powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a1-ab.ps1" -InputImage ".\sample.jpg" -OutputDir "experiments/local/p3a1/portrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

If the local branch already exists, use `git switch feat/p3a1-placement-aware-proposals; git pull --ff-only`. Never reset/delete the untracked `Cargo.lock`, `sample.jpg`, earlier outputs or prior experiment folders.

The runner creates paired `p2b1.png` / `p3a1.png`, ordered stroke JSON, p2c-v1 measurement reports and `summary.json` with file identities. It refuses to overwrite an existing experiment directory.

Share the nonprivate table:

```powershell
$s = Get-Content ".\experiments\local\p3a1\portrait-a\summary.json" -Raw | ConvertFrom-Json
$s.fixtures | Select-Object mode,strokes,strong_nonhorizontal_strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink | Format-Table -AutoSize
```

Inspect both PNGs at the same 100% zoom. Important visual questions:
- are hair/face masses less mechanically row-like?
- are eyewear, nose, lips and silhouette easier to read?
- do source-driven directions look coherent rather than tangled?
- does white paper remain clean?
- is the improvement worth any path-length/ink-density change?

## Acceptance / rejection

Do **not** accept P3-A.1 from metrics alone. It must beat frozen P2-B.1 visually on at least one permitted portrait and one **genuine nonportrait** image at comparable resource usage, without new white-gap contamination. P2-C's genuine nonportrait photo gate is still open.

If P3-A.1 fails:
- preserve it in `PHASE_EVOLUTION.md`;
- do not silently raise candidate density;
- inspect class/fallback statistics and regional failures;
- reconsider the objective before P3-B optimization.


## First CI observation — engineering pass, quality still open

GitHub Actions run `37265242828` completed successfully. On the 64×64 synthetic step fixture:

- total strokes: **193 vs 193** (P2-B.1 / P3-A.1);
- P3-A.1 tonal target: **173**;
- selected placement classes: **17 coarse, 91 fine, 65 tonal**;
- baseline fallback: **0**;
- generated source-driven candidates: **235**;
- strongly nonhorizontal strokes: **20 → 128**;
- total path length shown by CI: ~**1256.27 → 1271.43px**;
- rounded tone RMSE shown by the CI table: ~**0.32 → 0.49** (worse on this flat half-plane).

The step fixture therefore proves the placement path is active and budget-matched, but **does not prove quality**. In fact, the flat-tone proxy gets worse there. This is an important warning that a vertical/form-following field can reduce uniform dark coverage at the same count. Portrait/nonportrait visual evidence must decide whether the new placement strategy is useful for real structure.
