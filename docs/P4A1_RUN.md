# P4-A.1 — Real Nonportrait Pack Runbook

P4-A.1 is an **early structural-stroke experiment**, not a final renderer. It leaves the old P2-B.1 hatch field untouched and adds a separate long-gesture layer.

The real gate is therefore:

> Do the **paths-only** previews look like useful human-scale construction/contour strokes?

## Reuse the P3-G1 source pack

If the earlier chair/mug/plant photographs are still in:

```text
experiments/local/p3g1/sources/
```

reuse that exact folder. This keeps the source classes constant across the P3→P4 transition.

The folder must contain exactly 3–5 permission-cleared nonportrait PNG/JPEG files.

## Switch branch

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p4a1-long-structural-paths
```

If the branch already exists locally:

```powershell
git switch feat/p4a1-long-structural-paths
git pull --ff-only
```

Optional local engineering check:

```powershell
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
```

## Run the entire pack

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p4a1-pack.ps1" `
  -InputDir ".\experiments\local\p3g1\sources" `
  -OutputDir ".\experiments\local\p4a1\run-a" `
  -MaxSize 512 `
  -Seed 42 `
  -RightsConfirmed `
  -NonPortraitConfirmed
```

One-line equivalent:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p4a1-pack.ps1" -InputDir ".\experiments\local\p3g1\sources" -OutputDir ".\experiments\local\p4a1\run-a" -MaxSize 512 -Seed 42 -RightsConfirmed -NonPortraitConfirmed
```

The output directory must not already exist.

## What is generated

For each source:

```text
p2b1.png          frozen old renderer
p4a1-overlay.png  exact P2-B.1 + long gestures
p4a1-paths.png    long gesture paths ONLY
*.json
*-report.json
summary.json
```

The pack root also contains:

```text
summary.json
summary.csv
review.html
```

Open:

```powershell
Start-Process ".\experiments\local\p4a1\run-a\review.html"
```

## Review order

For every chair/mug/plant case:

1. inspect **P4-A.1 PATHS ONLY first**;
2. ask whether the long marks follow real object structure;
3. only then inspect the overlay;
4. do not penalize the overlay for still containing horizontal hatching — that is intentionally postponed to P4-A.3.

Look for:

- long chair legs / back edges / seat boundaries;
- mug rim, body curve and handle arcs;
- plant pot contour and leaf boundaries;
- useful interior structural paths;
- no wandering across blank paper;
- no obviously unsupported bridges;
- visibly continuous strokes instead of stitched tiny bars.

## Print the evidence

```powershell
$s = Get-Content ".\experiments\local\p4a1\run-a\summary.json" -Raw | ConvertFrom-Json

$s.counts
$s.means

$s.cases |
    Select-Object source_name,accepted_paths,seed_candidates,mean_path_length_px,max_path_length_px,total_path_length_px,overlay_edge_f1_improvement_pct,overlay_tone_improvement_pct,paths_only_edge_f1 |
    Format-Table -AutoSize
```

Share that terminal output plus screenshots of the **paths-only** and overlay review. The original source photos do not need to be uploaded.

## Decision

Proceed to P4-A.2 only if multiple real source classes show useful long structural gestures.

If paths are sparse but correct, that is acceptable: P4-A.2 can fill medium structure.

If paths are long but wrong, wandering, or mostly unhelpful silhouette duplicates, improve P4-A.1 tracing before building more hierarchy.


## Harness verification

The full 3-source batch flow is CI-green in [run 37408490425](https://github.com/SanamRai001/ScanSketch/actions/runs/37408490425). Importantly, a source with no coherent structural path is accepted as a valid `0 paths` result. You do not need to add or replace sources just to force the algorithm to draw something.


## Harness CI status

The three-view pack runner is now CI-green on both positive and zero-path cases. In particular, a smooth gradient with no supported structural seed completes successfully with `accepted=0` instead of failing the experiment script. [CI run 37473050878](https://github.com/SanamRai001/ScanSketch/actions/runs/37473050878).


## CI status

The three-view pack runner is now CI-green on the exact branch implementation. It correctly handles sources with zero accepted long paths. Reuse the prior P3-G1 chair/mug/plant folder for the first real P4-A.1 visual gate.
