# P3-A.2.2 — Missing-Structure Residual Verification

**Status:** implementation passed GitHub CI: **59/59 Rust tests**, all inherited P2-C/P3 checks, and the P3-A.2.2 missing-structure smoke. Real-image quality remains pending. [CI run 37345281232](https://github.com/SanamRai001/ScanSketch/actions/runs/37345281232).

## Frozen comparison controls

P3-A.2.2 changes **only** the structural utility map relative to P3-A.2.1.

Frozen:
- P2-B.1 baseline;
- exact P2 tonal prefix;
- exact total stroke count;
- P3-A.2 candidate generator;
- tile/spacing preselection;
- 40% replacement cap;
- 15% + 0.001 replacement margin;
- source-support/white checks;
- in-place structural substitution.

New:
- continuous P2-C-family edge-strength map;
- `missing_edge = max(source_edge - frozen_P2B1_preview_edge, 0)`;
- shared baseline/candidate utility dominated by missing-edge evidence.

The P2-C binary edge metric threshold remains unchanged.

## CI requirements

The synthetic smoke must verify:
- exact total count;
- exact tonal prefix;
- replacements + retained == structural budget;
- replacements <= cap;
- changed structural slots == replacements;
- all previous tests and P3 modes remain green.

The step fixture is an engineering fixture, not proof that missing-edge scoring improves real drawings.

## Windows portrait A/B after green CI

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p3a22-missing-structure-residual
$env:CARGO_BUILD_JOBS = "2"

cargo check --workspace
cargo test --workspace

powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a22-ab.ps1" -InputImage ".\sample.jpg" -OutputDir "experiments/local/p3a22/portrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

If already local:

```powershell
git switch feat/p3a22-missing-structure-residual
git pull --ff-only
```

Inspect:
- `p2b1.png`
- `p3a22.png`

Print:

```powershell
$s = Get-Content ".\experiments\local\p3a22\portrait-a\summary.json" -Raw | ConvertFrom-Json

"P3-A.2.2 mix: tone=$($s.missing_structure_stats.tone_count); budget=$($s.missing_structure_stats.structural_budget); max=$($s.missing_structure_stats.max_replacements); replacements=$($s.missing_structure_stats.replacements); retained=$($s.missing_structure_stats.baseline_retained); candidates=$($s.missing_structure_stats.candidate_count); mean-missing=$($s.missing_structure_stats.mean_missing)"
"Changed structural slots: $($s.changed_structural_slots)"

$s.fixtures |
  Select-Object mode,strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink |
  Format-Table -AutoSize
```

## Visual question

Do not judge only by aggregate edge F1.

The specific hypothesis is spatial:
- do replacements move away from already-strong upper hair/silhouette?
- do previously underrepresented interior structures gain useful marks?
- does the drawing remain tonally stable?

If the edge score rises but changed pixels remain concentrated in already-obvious contours, the missing-edge magnitude proxy is not enough and should be revised rather than rewarded.

A genuine nonportrait photo remains required before any generality claim.


## First CI observation

On the 64×64 step fixture:

- total strokes: **193 vs 193**;
- exact tonal prefix: **173**;
- structural budget: **20**;
- maximum replacements: **8**;
- replacements made: **6**;
- baseline structural strokes retained: **14**;
- candidate pool: **55**;
- weakest baseline missing-structure utility: **0.120863**;
- strongest hybrid missing-structure utility: **0.366960**;
- mean missing-edge evidence at accepted replacements: **0.126235**;
- rounded path length: ~1256.27→1259.85 px;
- rounded tone RMSE: ~0.32→0.31;
- rounded dark RMSE: ~0.45→0.44;
- rounded edge F1: ~0.79→0.80;
- rounded white RMSE: ~0.00→0.01.

This is engineering evidence only. The important result is that the new missing-structure utility is active while the **candidate generator, 40% cap and 15%+0.001 replacement margin remain frozen** from P3-A.2.1.
