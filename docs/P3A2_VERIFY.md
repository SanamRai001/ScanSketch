# P3-A.2 — Hybrid Structural Reinforcement Verification

**Status:** implementation branch `feat/p3a2-hybrid-structural-reinforcement`; quality is unproven until CI and real A/B evidence.

## Invariants

P3-A.2 must:
- preserve the exact P2 tonal stroke prefix;
- use exactly the structural stroke count frozen P2-B.1 would have used;
- keep total accepted stroke count identical to P2-B.1;
- use residual-aware multiscale structural accents only where source tone is underrepresented;
- fill any unspent structural slots with the original P2-B.1 contour records;
- preserve deterministic output and source-white safety.

## CI fixture

The step fixture is used only as an engineering check:
- same total count;
- exact tonal prefix;
- at least one hybrid structural candidate selected when a structural budget exists;
- budget accounting: `hybrid_selected + contour_fallback == structural_budget`;
- all inherited P2-C white-channel and measurement checks still pass.

A flat step is **not** a quality target. P3-A.1 already demonstrated that directional geometry can worsen flat tonal reconstruction.

## Windows portrait A/B

After green CI:

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p3a2-hybrid-structural-reinforcement
$env:CARGO_BUILD_JOBS = "2"

cargo check --workspace
cargo test --workspace

powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a2-ab.ps1" -InputImage ".\sample.jpg" -OutputDir "experiments/local/p3a2/portrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

If the branch already exists locally, use `git switch feat/p3a2-hybrid-structural-reinforcement; git pull --ff-only`.

The script refuses to overwrite its output directory and writes:
- `p2b1.png`, `p3a2.png`;
- paired stroke JSON;
- paired p2c-v1 reports;
- `summary.json` with SHA-256 identities, exact tonal prefix length and hybrid/fallback statistics.

Display the nonprivate result:

```powershell
$s = Get-Content ".\experiments\local\p3a2\portrait-a\summary.json" -Raw | ConvertFrom-Json

"P3-A.2 mix: tone=$($s.hybrid_stats.tone_count); budget=$($s.hybrid_stats.structural_budget); hybrid=$($s.hybrid_stats.hybrid_selected); fallback=$($s.hybrid_stats.contour_fallback); candidates=$($s.hybrid_stats.candidate_count)"
"Identical tonal prefix: $($s.tonal_prefix_identical_strokes)"

$s.fixtures |
  Select-Object mode,strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink |
  Format-Table -AutoSize
```

Inspect `p2b1.png` and `p3a2.png` at the same 100% zoom.

## Acceptance

P3-A.2 must preserve the tonal mass visually while improving meaningful structure. A higher edge F1 alone is insufficient.

Prefer:
- tone/midtone/dark errors near P2-B.1 rather than P3-A.1;
- no extra white contamination;
- local structural accents that help feature boundaries without turning the entire face/hair into contour drawing;
- a nonzero hybrid-selected count that does not consume the image indiscriminately.

Then repeat on a **genuine permission-cleared nonportrait** before claiming generality.

If it fails, preserve the phase result. Do not silently increase structural budget, because the whole purpose of P3-A.2 is bounded augmentation.
