# P3-A.2.1 — Selective Hybrid Replacement Verification

**Status:** implementation passed GitHub CI: **56/56 Rust tests**, all inherited P2-C/P3-A checks, and the P3-A.2.1 selective step smoke. Real-image quality remains pending. [CI run 37340548611](https://github.com/SanamRai001/ScanSketch/actions/runs/37340548611).

## What is different from P3-A.2

P3-A.2 preserved the tonal prefix but replaced all 108 structural strokes on the first portrait. P3-A.2.1 keeps the same hybrid candidate generator but directly scores **baseline contours and hybrid candidates using one utility**.

A replacement occurs only if:

```text
hybrid_utility >= baseline_utility * 1.15 + 0.001
```

and the total replacement count never exceeds **40% of structural budget**.

Replacements are substituted into the exact original contour slots. Every untouched baseline contour remains unchanged and in the same order.

## Engineering invariants

- exact P2 tonal prefix;
- exact P2-B.1 total stroke count;
- replacements + retained contours == structural budget;
- replacements <= 40% cap;
- deterministic same input/seed;
- no-contours mode remains exact P2-A;
- protected white channel remains clear;
- P3-A.2 full-replacement mode remains separately available.

## Windows portrait comparison after green CI

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p3a21-selective-hybrid-replacement
$env:CARGO_BUILD_JOBS = "2"

cargo check --workspace
cargo test --workspace

powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a21-ab.ps1" -InputImage ".\sample.jpg" -OutputDir "experiments/local/p3a21/portrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

If the branch already exists locally, use `git switch feat/p3a21-selective-hybrid-replacement; git pull --ff-only`.

The runner creates:
- `p2b1.png`
- `p3a21.png`
- paired stroke JSON
- paired p2c-v1 reports
- `summary.json`

It verifies total stroke parity, exact tonal prefix, replacement cap, structural budget accounting and exact changed structural slot count.

Print:

```powershell
$s = Get-Content ".\experiments\local\p3a21\portrait-a\summary.json" -Raw | ConvertFrom-Json

"P3-A.2.1 mix: tone=$($s.selective_stats.tone_count); budget=$($s.selective_stats.structural_budget); max=$($s.selective_stats.max_replacements); replacements=$($s.selective_stats.replacements); retained=$($s.selective_stats.baseline_retained); candidates=$($s.selective_stats.candidate_count)"
"Changed structural slots: $($s.changed_structural_slots)"

$s.fixtures |
  Select-Object mode,strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink |
  Format-Table -AutoSize
```

## Decision gate

The experiment is successful only if a **small selective intervention** improves meaningful visual structure while preserving P2-B.1's tonal body.

A zero-replacement result is valid: it means the baseline contours already beat the candidates under the frozen utility/margin.

Do not loosen the 15%+0.001 margin or 40% cap merely because few strokes are replaced. First inspect the result.

A genuine nonportrait test remains required before claiming generality.


## First CI observation

On the 64×64 step fixture:

- total strokes: **193 vs 193**;
- exact tonal prefix: **173**;
- structural budget: **20**;
- max replacements: **8**;
- replacements actually made: **6**;
- baseline contours retained: **14**;
- candidate pool: **55**;
- weakest baseline utility: **0.453165**;
- strongest hybrid utility: **1.057108**;
- rounded path length: ~1256.27→1262.04px;
- rounded tone RMSE: ~0.32→0.31;
- rounded dark RMSE: ~0.45→0.44;
- rounded edge F1: ~0.79→0.81;
- rounded white RMSE: ~0.00→0.01.

This is engineering evidence only. The important result is that the selector is now genuinely selective rather than full replacement.
