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

## 5. First portrait A/B is now complete — synthetic/nonportrait next

The initial measurement error was expected: the P2-A control PNG and stroke JSON had not yet been generated. Once those two files were created, the one-line measurement command succeeded. The observed **P2-A control had 12,563 strokes**. Compared with P2-B.1's 12,671 strokes, the new contour marks raised this fixed image's edge-proxy F1 from **0.1840614** to **0.2887059** without altering measured white RMSE or highlight-ink fraction.

The detailed table, raw values and caution about unequal accepted stroke counts are in [P2C_PORTRAIT_AB.md](P2C_PORTRAIT_AB.md). Both local report filenames in §4 should be **preserved**, not rerun and overwritten.

Immediate remaining measurements:

1. If not yet created, generate six public synthetic sources with `scansketch-fixtures` (§1). Render `step.png`, `square-white-channel.png` and `gradient.png` with paired PNG/JSON; run `scansketch-measure` for each using `--max-size 64` and unique report paths. Record observed behavior rather than assuming it from unit tests.
2. Choose a permission-cleared nonportrait object image and repeat both original/no-contours modes on the **same** binary at max-side 512 and seed 42 with unique names, recording visual judgement and numeric metrics.
3. Preserve local SHA-256 hashes of sources and paired outputs; do not publish the private source without permission.
4. Signed tone bias (preview darkness minus source darkness) is useful to determine whether dark-region error is caused by underdraw or overdraw, but this metric is not in `p2c-v1` yet. Version any addition and reevaluate the baselines.

## 6. Paired synthetic fixture experiment (sources already generated)

The user's Windows run successfully created all six 64×64 **source** PNGs in `experiments/local/p2c-v1/`:
`white.png`, `transparent-black.png`, `step.png`, `square-white-channel.png`, `gradient.png` and `thin-lines.png`. **CI has already executed and measured the three paired fixture types successfully, with mixed outcome**: [P2-C synthetic CI result](P2C_SYNTHETIC_CI_RESULT.md). The user's local **sources** exist; the full-precision Windows results and image inspection are still next.

A reusable script now runs three meaningfully different source types under the **same binary** with both `--no-contours` (P2-A) and enabled contours (P2-B.1), seed 42, max-side 64, and unique paired outputs. It refuses to overwrite an existing result directory and saves all six detailed reports + a local JSON summary with file SHA-256 hashes.

From repo root, after `git pull --ff-only`, run **one command** in PowerShell:

```powershell
.\scripts\run-p2c-synthetic.ps1 -FixtureDir ".\experiments\local\p2c-v1" -OutputDir ".\experiments\local\p2c-v1\results"
```

If your Windows execution policy disallows local scripts, run the same checked-in script explicitly without changing global policy:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p2c-synthetic.ps1" -FixtureDir "experiments/local/p2c-v1" -OutputDir "experiments/local/p2c-v1/results"
```

The script creates 12 paired outputs (six PNGs and six JSON stroke files), six measurement reports and `summary.json` under the ignored results folder. It prints a compact comparison table including fixture, mode, strokes, tone RMSE, edge F1 and accidental white ink.

View the summary with:

```powershell
Get-Content ".\experiments\local\p2c-v1\results\summary.json" -Raw
```

**Windows execution is complete:** the user supplied the six full-precision table entries, now archived in [P2C_SYNTHETIC_WINDOWS_RESULT.md](P2C_SYNTHETIC_WINDOWS_RESULT.md). The batch produced paired outputs and local SHA hashes in `results/summary.json`. Next inspect the *synthetic* original plus both `square-white-channel-*.png` outputs: the whole-source-white-mask unintended-ink fraction is exactly 0.02548076923076923 **in both modes**. The affected pixel location has **not** been determined, so do not equate this whole-mask rate with inner-channel contamination. If the directory already exists, supply a new `-OutputDir` (for example `results-02`); do not delete the old experiment. The script records the actual run's Git SHA and hashes. See [phase evolution](PHASE_EVOLUTION.md) for what to retain after each phase.

This is the natural-output lane, **not** a matched-budget comparison. Later test a permission-cleared nonportrait photograph, also with both modes, before proceeding to P3-A.

## 7. Targeted inspection after the Windows synthetic run

These commands do **not** overwrite any output:

```powershell
$root = ".\\experiments\\local\\p2c-v1"
$r = "$root\\results"

# Inspect the three safe, programmatically generated images side by side.
Start-Process "$root\\square-white-channel.png"
Start-Process "$r\\square-white-channel-p2a.png"
Start-Process "$r\\square-white-channel-p2b1.png"

# Look at the individual full-precision diagnostic fields, not only the batch table.
@("step","square-white-channel","gradient") | ForEach-Object {
  $fixture = $_
  @("p2a","p2b1") | ForEach-Object {
    $mode = $_
    $v = Get-Content "$r\\$fixture-$mode-report.json" -Raw | ConvertFrom-Json
    [pscustomobject]@{
      Fixture = $fixture; Mode = $mode
      SourceEdges = $v.edges.source_pixels
      PreviewEdges = $v.edges.preview_pixels
      Precision = $v.edges.precision
      Recall = $v.edges.recall
      SourceWhitePixels = $v.white_region.pixels
      WhiteInkFraction = $v.unwanted_highlight_ink_fraction
      ExtraWhiteDarkness = $v.mean_extra_highlight_darkness
    }
  }
} | Format-Table -AutoSize
```

The square fixture has 2080 white-source pixels including **both** exterior background and its internal six-column channel (x=29–34, y=8–55). The reported fraction implies 53 threshold-exceeding preview pixels somewhere in this combined mask. A visual inspection and, if still uncertain, an exact coordinate-level diagnostic are required to distinguish edge antialiasing versus improper mark support. Since both algorithms report the same rate, the new contours do not appear to be its cause on this fixture.

Read the [full Windows findings](P2C_SYNTHETIC_WINDOWS_RESULT.md) and [targeted channel audit commands](P2C_WHITE_CHANNEL_AUDIT.md). The user has uploaded screenshots: the channel appears visually continuous but pixel coordinates cannot be verified from viewer screenshots. Run the new read-only CLI on raw source/preview PNGs, then complete one permitted nonportrait A/B before P3-A.

## Metrics and limitations

- `tone_rmse`: root-mean-square error of target vs preview linear-light darkness; regional masks: white `D<=0.04`, dark `D>=0.65`, midtone between them.
- `unwanted_highlight_ink_fraction`: fraction of *source white* pixels (`D<=0.04`) with preview darkness strictly above 0.04. Mean extra highlight darkness reports positive preview-minus-source on that same mask.
- `edges`: symmetric source/preview binomial-smoothed Sobel masks using a fixed 0.22 threshold. Precision and recall use a ±2-pixel neighborhood (Chebyshev). Both-empty masks receive score 1 by convention.
- `strokes`: count, Euclidean path length, mean/median width and opacity calculated from the actual ordered JSON list. Metrics do **not** re-render JSON or prove that a separate preview PNG came from exactly those records; keep paired render outputs together and use hashes.
- A lower RMSE or higher edge F1 can reward photographic pixel fidelity or mechanical texture instead of a convincing pencil drawing. Use the protocol's blinded visual judgement too. `--max-strokes` is an overflow guard, **not** an equal-budget allocator.

## Remaining work after this first slice

Validate the real metrics on more than one permitted photo, check the source/preview hash/manifest process, gather runtime and memory explicitly, and add an equal-budget experiment design before P3-A. Do not upload personal photographs to this public repository.
