# P2-C v1 measurement runbook (Windows PowerShell)

GitHub CI has verified workspace compilation, **31/31 tests** and an actual all-white fixture → render → measure run: 0 strokes, `tone_rmse=0`, unwanted highlight ink fraction 0. [Successful run](https://github.com/SanamRai001/ScanSketch/actions/runs/37219282372).

This is a **diagnostic tool**, not an automatic aesthetic winner selector. Read [P2-C protocol](P2C_PROTOCOL.md) before comparing versions. It uses the same `darkness_map` as the Rust renderer; source and preview are both compared in linear-light darkness with RGBA matted onto white.

## Setup without losing your existing work

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p2c-measurement-utility
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
cargo run -p scansketch-cli --bin scansketch-measure -- --help
```

If your branch is already tracked, use `git switch feat/p2c-measurement-utility; git pull --ff-only`. Don't reset or discard the local `Cargo.lock`, `sample.jpg`, `outputs/` or earlier `git stash`. We explicitly set `default-run = "scansketch"`, so your existing `cargo run -p scansketch-cli -- ...` remains valid even with the two additional binaries.

## 1. Generate rights-clear synthetic source fixtures

Choose a new local output folder. All generated fixtures have **64×64 RGBA PNG** with original synthetic pixel data; no externally sourced or personal photographs.

```powershell
cargo run -p scansketch-cli --bin scansketch-fixtures -- `
  --output-dir ".\experiments\local\p2c-v1"
```

Generated:
- `white.png` (no ink expected);
- `transparent-black.png` (white after alpha compositing);
- `step.png` (black/white half-plane, known edge);
- `square-white-channel.png` (white-gap protection);
- `gradient.png` (tonal progression);
- `thin-lines.png` (structure retention).

The generator refuses overwrites: use another directory name for a second experiment. This folder is intentionally ignored by Git.

## 2. First end-to-end sanity run (white paper)

```powershell
cargo run -p scansketch-cli -- `
  --input ".\experiments\local\p2c-v1\white.png" `
  --output ".\experiments\local\p2c-v1\white-render.png" `
  --strokes ".\experiments\local\p2c-v1\white-strokes.json" `
  --seed 42 --max-size 64

cargo run -p scansketch-cli --bin scansketch-measure -- `
  --source ".\experiments\local\p2c-v1\white.png" `
  --preview ".\experiments\local\p2c-v1\white-render.png" `
  --strokes ".\experiments\local\p2c-v1\white-strokes.json" `
  --max-size 64 `
  --report ".\experiments\local\p2c-v1\white-measurement.json"
```

Expected: `tone_rmse = 0`, `unwanted_highlight_ink_fraction = 0`, `strokes.count = 0`, no source/preview edges, and `edges.f1 = 1` by the documented both-empty convention. If any fail, stop before measuring photographs.

The CLI validates that image/stroke dimensions match; `--max-size` must match the generation command, and the renderer's Triangle resize is reproduced exactly. The report file is never overwritten.

## 3. Measure the portrait (same source, size and seed)

Your earlier P2-B.1 PNG may not have an accompanying `--strokes` JSON because the first example did not require exporting it. Generate a **new paired PNG+JSON** rather than trying to score the old PNG using an unrelated sketch record.

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null
cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\p2c-p2b1-512.png" `
  --strokes ".\outputs\p2c-p2b1-512-strokes.json" `
  --max-size 512 --seed 42 --contour-threshold 0.28

cargo run -p scansketch-cli --bin scansketch-measure -- `
  --source ".\sample.jpg" `
  --preview ".\outputs\p2c-p2b1-512.png" `
  --strokes ".\outputs\p2c-p2b1-512-strokes.json" `
  --max-size 512 `
  --report ".\outputs\p2c-p2b1-512-measurement.json"
```

To measure P2-A from exactly the same P2-B.1 binary, repeat the render with unique P2-A filenames and `--no-contours`, then measure the corresponding generated JSON. For older historical engines use their own checkout/branch (and record SHA and old defaults) to prevent accidental mixing of versions.

### Record file identities and environment

```powershell
git rev-parse HEAD
rustc --version
cargo --version
Get-FileHash ".\sample.jpg", ".\outputs\p2c-p2b1-512.png", ".\outputs\p2c-p2b1-512-strokes.json" -Algorithm SHA256
```

Keep hashes and the full flags alongside the report in a private run manifest; the first utility does not yet auto-embed SHA-256, CPU profiling or wall-clock generation time. `null` in a report indicates an absent region or no strokes, **not** a score of zero.

## 4. Same-binary P2-A control — NEXT MEASUREMENT

The user has successfully run the P2-B.1 render and measurement on Windows. Actual source 368×512 at max-side 512, seed 42; output: `outputs/p2c-portrait.png`, paired `outputs/p2c-portrait-strokes.json`, report `outputs/p2c-portrait-report.json` (recorded in [first portrait result](P2C_FIRST_PORTRAIT_RESULT.md)).

Do **not** change branches or rerun the existing report file. On the **same** `feat/p2c-measurement-utility` checkout, render tonal-only by disabling contours, and save to new filenames:

```powershell
cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\p2c-portrait-p2a-control.png" `
  --strokes ".\outputs\p2c-portrait-p2a-control-strokes.json" `
  --max-size 512 --seed 42 --no-contours

cargo run -p scansketch-cli --bin scansketch-measure -- `
  --source ".\sample.jpg" `
  --preview ".\outputs\p2c-portrait-p2a-control.png" `
  --strokes ".\outputs\p2c-portrait-p2a-control-strokes.json" `
  --max-size 512 `
  --report ".\outputs\p2c-portrait-p2a-control-report.json"
```

For the previous portrait, `--no-contours` produced 12,563 P2-A tonal strokes at the same 368×512 / seed 42. **That count is a consistency expectation, not proof of byte-for-byte equality.** Share the *new report text*, not the private original image. Retain the earlier P2-B.1 numeric report and compare: total/region RMSE, highlight ink, edge precision/recall/F1, stroke count and path length. Stroke count differences alone are not a quality judgement.

For local provenance:

```powershell
git rev-parse HEAD
Get-FileHash ".\sample.jpg", ".\outputs\p2c-portrait.png", ".\outputs\p2c-portrait-strokes.json", ".\outputs\p2c-portrait-p2a-control.png", ".\outputs\p2c-portrait-p2a-control-strokes.json" -Algorithm SHA256
```

Hash values are not required to be posted publicly; keep them with the private local experiment.

**Important limit:** current RMSE is unsigned. A higher dark-region RMSE does not by itself prove the sketch is too pale or too dark. Consider a *separately versioned future metric* of mean signed error (preview darkness minus target darkness) across fixed source masks; don't silently alter the v1 report definition.

## Metrics and limitations

- `tone_rmse`: root-mean-square error of target vs preview linear-light darkness; regional masks: white `D<=0.04`, dark `D>=0.65`, midtone between them.
- `unwanted_highlight_ink_fraction`: fraction of *source white* pixels (`D<=0.04`) with preview darkness strictly above 0.04. Mean extra highlight darkness reports positive preview-minus-source on that same mask.
- `edges`: symmetric source/preview binomial-smoothed Sobel masks using a fixed 0.22 threshold. Precision and recall use a ±2-pixel neighborhood (Chebyshev). Both-empty masks receive score 1 by convention.
- `strokes`: count, Euclidean path length, mean/median width and opacity calculated from the actual ordered JSON list. Metrics do **not** re-render JSON or prove that a separate preview PNG came from exactly those records; keep paired render outputs together and use hashes.
- A lower RMSE or higher edge F1 can reward photographic pixel fidelity or mechanical texture instead of a convincing pencil drawing. Use the protocol's blinded visual judgement too. `--max-strokes` is an overflow guard, **not** an equal-budget allocator.

## Remaining work after this first slice

Validate the real metrics on more than one permitted photo, check the source/preview hash/manifest process, gather runtime and memory explicitly, and add an equal-budget experiment design before P3-A. Do not upload personal photographs to this public repository.
