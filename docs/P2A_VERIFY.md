# P2-A verification — Windows PowerShell

P1 was confirmed to compile and passed 11/11 core tests on the user's machine; the portrait revealed striped/engraving-like output. P2-A is **not yet visually verified**. Always compare using the same original, resize and seed. Preserve the P1 image already saved at `outputs/sample-sketch.png`.

## Update and compile

```powershell
cd D:\Projects\ScanSketch
git switch feat/p2-sketch-stroke-language
git pull --ff-only origin feat/p2-sketch-stroke-language
$env:CARGO_BUILD_JOBS = "2"
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

The P2-A core adds three focused checks beyond P1's eleven. If formatting needs changes, run `cargo fmt --all`, review and commit only intentional formatting. Do not delete any untracked `Cargo.lock`: the CLI workspace needs that file for reproducibility.

## Render the same portrait

Keep the original `sample.jpg` at the repository root or substitute its real input path.

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null
cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\sample-p2a.png" `
  --strokes ".\outputs\sample-p2a-strokes.json" `
  --max-size 512 `
  --seed 42
Start-Process ".\outputs\sample-p2a.png"
```

The P2-A defaults use 24-pixel sampling segments, compared with 8 in P1. Do not overwrite the old P1 preview; inspect both images side-by-side at 100% zoom. Check white paper, eyes/glasses/lips, hair strand/direction, line fragmentation, density, and whether the silhouette is preserved.

Repeat the command with different filenames but identical settings and compare SHA256 of PNG and JSON; hashes must match. A small output from fragmented shading is not proof of portrait quality. Share both images before P2-B.

## Explicit boundaries

No contour following, Primitive optimization, paper texture, smudging or erasing is in P2-A. The white-column protection rule and stroke budget still apply. If stroke budget is exceeded, record input size, options and stroke count assumptions; do not silently truncate.
