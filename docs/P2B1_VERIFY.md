# P2-B.1 verification — ranked contour A/B experiment

P2-B compiled and passed 20/20 tests; P2-B.1 compiled and passed 23/23 tests in GitHub Actions (https://github.com/SanamRai001/ScanSketch/actions/runs/37217765227). Its first portrait generated 13,024 strokes vs 12,563 P2-A tonal-only, but the qualitative improvement was subtle. The hypothesis for P2-B.1 is that smoother, coherence-ranked structural candidates give useful marks more priority and avoid excess texture.

## Update safely in Windows PowerShell

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p2b1-contour-coherence
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
```

If a local branch already exists, `git switch feat/p2b1-contour-coherence; git pull --ff-only`. Do not run `git reset --hard`, discard your local `Cargo.lock` or `sample.jpg`, or pop old P1 stashes onto this branch. Branch switching should retain the untracked input and outputs.

## Same-photo comparison

Do not overwrite the old `outputs/portrait-p2b-contours.png`; that file is our original P2-B comparison. Generate new filenames:

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null

cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\portrait-p2b1-ranked.png" `
  --strokes ".\outputs\portrait-p2b1-ranked.json" `
  --max-size 512 --seed 42 --contour-threshold 0.28

cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\portrait-p2b1-conservative.png" `
  --max-size 512 --seed 42 --contour-threshold 0.38

cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\portrait-p2b1-control.png" `
  --max-size 512 --seed 42 --no-contours
```

The no-contours control should still contain exactly the old 12,563 tonal marks at this 368x512 size and seed (count alone is insufficient; identical stroke JSON is the stronger repeatability check). Upload the original P2-B and the new ranked result, ideally a 100% zoom crop including spectacles, nose, mouth and hairline.

Evaluate: contour utility near face anchors versus hair texture, unintended white ink, sketch feel vs hard outlines, stroke count, deterministic repeat runs, performance on the 16GB Windows laptop. Use a second uncomplicated input (black-on-white geometry) before accepting this phase.

## Gate

Do not equate passing CI with artistic success. If the ranked variation is still too subtle, proceed to an explicitly measured/reconsidered stroke strategy instead of silently stacking yet another unvalidated effect.
