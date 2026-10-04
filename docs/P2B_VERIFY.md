# P2-B verification — same-photo contour A/B

P2-A already passed GitHub Actions workspace check and 14 core tests, and the first portrait comparison showed broken marks but weak glasses/eye/lip structure. P2-B is a separate branch; do not overwrite or rebase the original images or pop the P1 stash into it.

## Prepare

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p2b-contour-reinforcement
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
```

If Git refuses to switch because of local source changes, stash only the tracked code files after checking `git diff`; preserve `sample.jpg`, `Cargo.lock` and `outputs/` locally. If the branch is already tracked, use `git switch feat/p2b-contour-reinforcement` and `git pull --ff-only`.

## Produce both controls with identical P2-B binary

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null

cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\portrait-p2a-control.png" `
  --strokes ".\outputs\portrait-p2a-control.json" `
  --max-size 512 --seed 42 --no-contours

cargo run -p scansketch-cli -- `
  --input ".\sample.jpg" `
  --output ".\outputs\portrait-p2b-contours.png" `
  --strokes ".\outputs\portrait-p2b-contours.json" `
  --max-size 512 --seed 42 --contour-threshold 0.28

Start-Process ".\outputs\portrait-p2a-control.png"
Start-Process ".\outputs\portrait-p2b-contours.png"
```

Compare original, P1, P2-A control and P2-B at 100% zoom. Specific observation: are spectacles, eyes, lips and hair boundary clearer without strengthening noisy hair texture more than facial features? Inspect blank background and narrow highlights. Test `--contour-threshold 0.40` as a conservative comparison if the default marks are too dense, changing output filenames rather than overwriting.

`--no-contours` should preserve exactly P2-A's ordered strokes (same input, seed and defaults); the core unit test asserts the tonal prefix is unchanged with the pass enabled.

## Acceptance before next phase

- Cargo checks and tests pass.
- Output with optional contour accents differs in stroke records, while repeat runs with identical arguments match byte-for-byte.
- No unwanted white-region contour ink or strokes outside canvas; no cartoon outlines.
- Contour-on result improves readability across more than one image type. If not, retain the opt-out and revise the hypothesis before P2-C.
