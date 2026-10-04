# P2-C Synthetic A/B — CI Execution (First Observation)

Date: **2026-10-04**. Source: [GitHub Actions run 37221162546](https://github.com/SanamRai001/ScanSketch/actions/runs/37221162546), job paired synthetic experiment, commit `4d9dbcd94af3d092400f060b8245a883aa9be79f`.

**Status:** the newly added PowerShell batch actually generated **six renders** (P2-A tonal-only vs P2-B.1 enabled, same binary) and **six reports** for three rights-clear synthetic fixtures. GitHub CI passed Rust workspace compilation, **31/31 tests**, the all-white zero-ink smoke test and the six-pair experiment script. This confirms runnable tooling; it does **not** certify aesthetic quality.

Fixtures are programmatically generated 64×64 RGBA, max-side 64, seed 42, default P2-B.1 contour threshold 0.28, `p2c-v1` measurement constants. Summary below is copied from **CI's terminal display, rounded by PowerShell's table formatting to 2 decimal places**. Full-precision reports/summary were produced inside the ephemeral runner and were not uploaded as downloadable CI artifacts; do not treat the table's rounded 0.00 as mathematically exact zero.

| Fixture | Mode | Strokes | Tone RMSE (display rounded) | Edge F1 (rounded) | Unwanted white-mask ink fraction (rounded) |
| --- | --- | ---: | ---: | ---: | ---: |
| Step half-plane | P2-A | 173 | 0.32 | 0.82 | 0.00 |
| Step half-plane | P2-B.1 | 193 | 0.32 | **0.79** | 0.00 |
| Square with white channel | P2-A | 196 | 0.34 | 0.99 | **0.03** |
| Square with white channel | P2-B.1 | 236 | 0.32 | 0.99 | **0.03** |
| Smooth gradient | P2-A | 290 | 0.39 | 0.00 | 0.00 |
| Smooth gradient | P2-B.1 | 290 | 0.39 | 0.00 | 0.00 |

### Observations / risks

- **Mixed outcomes:** adding contours appears to lower the step edge's F1 at the rounded values (0.82→0.79), leaves channel F1 similar (0.99→0.99), and gives no visible metric difference on the smooth gradient. The square/white-channel case shows a rounded ~0.03 unwanted-white-ink fraction **in both modes**. The latter may include boundary antialiasing or genuine contamination; investigate the actual PNGs and masks instead of assuming a cause.
- The gradient's rounded edge F1 0.00 is **not** proof that the image is visually hopeless: the fixed Sobel magnitude/threshold proxy may be unsuitable to smooth low-contrast inputs. Inspect source/preview edge counts and the output before concluding.
- These runs are **natural output budgets**, not equal-count comparisons: the contour-on step adds 20 strokes and channel adds 40; gradient adds none.
- The private portrait's measured proxy gain (**F1 0.1841→0.2887**) therefore **does not generalize automatically** to all geometry. Keep P2-B.1 as the current candidate and P2-A as control, but the P2-C acceptance gate is still open.

### Next local experiment

The user has separately generated all six source PNGs on Windows under `experiments/local/p2c-v1/`. Run `scripts/run-p2c-synthetic.ps1` on that local set, inspect actual paired PNGs and keep its **full-precision `results/summary.json`** (including SHA-256 file identities). Inspect the narrow white channel at 100%; if ink appears there, distinguish renderer stroke crossing, raster antialiasing and metric threshold behavior before changing the algorithm. Then test one permission-cleared nonportrait photo under both modes. Record results in [PHASE_EVOLUTION.md](PHASE_EVOLUTION.md).
