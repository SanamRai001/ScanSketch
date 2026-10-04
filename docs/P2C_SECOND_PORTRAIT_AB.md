# P2-C Portrait B — Paired Visual and Numeric Review

**Date:** 2026-10-04. **Classification correction:** the selected local `photo.jpg` is another **portrait of a person**, as established by the two subsequently uploaded sketch previews. Despite the output directory `object-results/` and the previous assistant's provisional label, this **does not satisfy the independent nonportrait photograph gate**. A filename, the script name `run-p2c-photo-ab.ps1` and `-RightsConfirmed` do not establish subject class. Keep original photos, generated PNGs and JSONs private/local; only the user-supplied anonymous measurements and qualitative observations are in this public document.

## Controlled conditions

The Windows `run-p2c-photo-ab.ps1` script completed both renders, exports and measurements with the same selected local source, unchanged P2-C renderer binary, max-side 512, **actual 400×512 working image**, seed **42** and protocol `p2c-v1`. Natural-output budgets differ: P2-B.1 enables optional contour reinforcement; P2-A has `--no-contours`.

The two uploaded previews appear in the same order the script produces results (**first presumed P2-A, second presumed P2-B.1**); labels were not included in the uploads themselves. Do not claim pixelwise or hash-confirmed visual correspondence from differently framed screenshots.

| Metric | P2-A tonal-only | P2-B.1 contours | Observed comparison |
| --- | ---: | ---: | --- |
| Accepted stroke count | 10929 | 11247 | +318 (+2.91%) |
| Overall tone RMSE ↓ | 0.356077459408093 | 0.3534562796180163 | −0.00262118 (~0.74%) |
| White-region RMSE ↓ | 0.02575703358740595 | 0.02575703358740595 | Unchanged |
| Dark-region RMSE ↓ | 0.44707993485129777 | 0.44360196389367607 | −0.00347797 |
| Edge F1 ↑ (fixed Sobel/2px proxy) | 0.487013015465261 | 0.6621522006396106 | +0.175139 (~36% relative) |
| Unwanted source-white ink fraction ↓ | 0.0011995030630167502 | 0.0011995030630167502 | Unchanged |

The script saved `experiments/local/p2c-v1/object-results/{p2a,p2b1}.png`, paired stroke JSON, full `*-report.json` and `summary.json` with SHA-256 locally. The folder name is historical/misleading, not evidence the image is a nonportrait. The user copied the summary table but did not provide those hash values here.

## Qualitative review of uploaded previews

Both previews display a recognizably human head, hair, ears, neck and upper torso. The mostly horizontal, short repetitive tonal marks remain visually dominant across hair and face. Local source-derived structure is somewhat richer with the contour-on version, but the improvement to easily recognized eyes, nose, lips and face planes is subtle at the uploaded scale. The silhouette and protected white background survive both variants; the overall drawing still reads more like ordered hatch reconstruction than deliberate form-following pencil work. There is no evidence here of a new subject category being successfully reconstructed.

**Do not interpret +36% edge-proxy F1 as +36% perceived sketch quality.** The edge metric rewards short-distance gradient alignment, not explicit human landmarks. The stronger numeric score motivates retaining P2-B.1 as a structural baseline; the still-horizontal visual language motivates a genuine P3-A orientation-field hypothesis.

## Updated decision and next gate

- Retain P2-B.1 for structural comparisons and P2-A as tonal control. Record mixed synthetic behavior separately; do not overwrite either branch or prior results.
- **Nonportrait photographic gate remains OPEN.** Use an actual photo of a mug, chair, bicycle, architectural detail or other nonhuman object, where usage permission is clear. Choose the input via Windows file picker and confirm the selected image depicts the intended subject *before* invoking `run-p2c-photo-ab.ps1`; save to a **new** output directory such as `nonportrait-results/`.
- P3-A multiscale/direction-aware proposal design is justified by repeated mechanical horizontal texture. A guarded/opt-in research prototype can be planned, but **P2-C should not be declared complete nor P3-A accepted as an improvement** until compared with baseline on a genuinely nonportrait photo and permitted portrait under documented conditions.
- The existing protected-white-channel audit passed: 0/288 pixels contaminated inside the channel in both modes; its 53 affected source-white pixels were all exterior and within two pixels of source-dark material.
