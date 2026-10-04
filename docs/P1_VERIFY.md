# P1 Verification — Windows PowerShell

Status: commands to run locally; **not yet executed by the assistant** (Rust toolchain unavailable in its working environment). Requires Git and a current stable Rust toolchain from https://rustup.rs/.

## 1. Inspect and prepare

```powershell
git clone https://github.com/SanamRai001/ScanSketch.git
cd ScanSketch
git switch feat/p1-rust-scanline-baseline
git status --short
git log -3 --oneline
rustc --version
cargo --version
```

Expected: no unrelated local changes; Rust and Cargo versions print. This branch is based on the still-unmerged documentation foundation.

## 2. Formatting and correctness

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace
cargo test -p scansketch-core
cargo run -p scansketch-cli -- --help
```

Expected: formatting check and Cargo check succeed; 8 core-level tests plus the analysis test should run successfully (9 total in this initial implementation); CLI shows PNG/JPEG input and options. If any command fails, share the **first** error and do not advance. `cargo fmt --all` may change source formatting; review those changes instead of discarding them.

## 3. Test a real photograph

Put an appropriately licensed photo at `sample.jpg` (not committed to Git). Use an existing local output directory:

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null
cargo run -p scansketch-cli -- --input .\sample.jpg --output .\outputs\result-a.png --strokes .\outputs\strokes-a.json --seed 42 --max-size 512
cargo run -p scansketch-cli -- --input .\sample.jpg --output .\outputs\result-b.png --strokes .\outputs\strokes-b.json --seed 42 --max-size 512
Get-FileHash .\outputs\result-a.png, .\outputs\result-b.png -Algorithm SHA256
Get-FileHash .\outputs\strokes-a.json, .\outputs\strokes-b.json -Algorithm SHA256
```

Expected: CLI reports nonzero stroke count for a normal photo; both PNG hashes match each other, and both JSON hashes match each other when source, seed, options and engine build are identical. White-only synthetic fixture is asserted separately in unit tests.

Manual review:
- Is it composed of visible pencil lines rather than gray-filled pixels?
- Are white areas mostly untouched?
- Are basic shapes recognizable at 100% zoom?
- Are black areas too striped, muddy or too faint? Document failures.
- Does processing stay acceptable on the actual laptop?
- Try unusual aspect ratios, a transparent PNG, a blank image and one detailed image.

## 4. Record resolved dependency versions

Cargo normally creates `Cargo.lock` for this workspace upon resolution. Keep and commit it for reproducible CLI builds after the checks succeed. Do **not** commit `target/`, generated images or private test assets.

Report back with the `cargo check`/test result, one screenshot or generated output, and any concerns. We can then fix P1 before advancing to a measured P2 experiment.

## Limits and expected errors

- Input formats: PNG/JPEG only.
- Compressed file: max 16 MiB; decoded dimensions: max 4096 per side; working dimensions: max 1024 per side.
- CLI `--max-size`: 32..=1024. `--band-height`: 1..=16. `--segment-width`: 2..=64. `--white-threshold`: 0..=0.5. `--max-strokes`: 1..=500000.
- Invalid paths, unsupported formats, invalid controls, and exhausted stroke budget should exit nonzero with a clear error.
