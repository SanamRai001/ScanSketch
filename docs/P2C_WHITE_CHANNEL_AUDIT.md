# P2-C — Source-white pixel diagnostic, screenshot review

**Status:** screenshot review observed; original raw synthetic PNGs and stroke JSONs remain on the user's Windows laptop. A read-only `scansketch-white-audit` command is now added to locate and count p2c-v1 affected source-white pixels; its result must be observed before making a causal claim.

## Uploaded screenshot review (2026-10-04)

The user shared three images after the local same-binary P2-A/P2-B.1 synthetic A/B: one enlarged sketch preview, the 64×64 black-square/two-bar **source** shown within an image viewer, and a smaller sketch preview. Screenshot zoom/viewer backgrounds differ, and unlabelled screenshots alone do **not** establish which preview belongs to which mode.

**Qualitative observation:** the wide central white gap appears continuous in the generated previews; the dark rectangular bars are represented by many short horizontal gray marks rather than solid filled blocks, and some edge/boundary lightening appears. There is no obvious bold horizontal line bridging the entire protected gap at the presented screenshot scales. This supports additional inspection, **not** a claim that no individual interior pixels were darkened.

The previous P2-C metric found `unwanted_highlight_ink_fraction = 0.02548076923076923` in both modes, meaning **53 out of 2080 pixels in the entire SOURCE-white mask** have output darkness greater than 0.04. That mask includes the exterior white paper and the 6px-wide internal channel. The **channel contains exactly 288 target-white pixels** (x=29..34, y=8..55). None of the screenshots can reliably identify which 53 raw source coordinates crossed the threshold.

## Read-only diagnostic, without altering the original scoring

`scansketch-core::audit_white_pixels` uses **the same `darkness_map` and 0.04 source/output thresholds** as `p2c-v1`. Its Rust/CLI input limits, source preprocessing and white matting match the existing measurement utility. It reports for each mode:

- white target pixels and affected preview pixels (which must agree with old `p2c-v1` counts);
- source-white and affected pixels **inside the explicitly supplied channel rectangle** versus affected pixels **outside** that rectangle;
- of all affected pixels, how many lie within 1px or 2px of target nonwhite pixels, how many are farther than 2px;
- up to the first 128 exact pixel coordinates in stable row-major order, including output darkness, inside/outside ROI and proximity flags (53 therefore fit in full);
- optional new JSON audit report, with **create-new/no-overwrite** semantics.

It cannot alone prove why a pixel has ink: border antialiasing, raster cap spread and vector stroke footprint need geometry/PNG correlation after pixel location is known. No contour/Tone/Sketch generation changes or `p2c-v1` score-definition changes accompany the audit.

### Channel inspection on the Windows checkout

After `git pull --ff-only` and `cargo test --workspace`, run these as **single lines** from `D:\Projects\ScanSketch`:

```powershell
cargo run -p scansketch-cli --bin scansketch-white-audit -- --source ".\experiments\local\p2c-v1\square-white-channel.png" --preview ".\experiments\local\p2c-v1\results\square-white-channel-p2a.png" --max-size 64 --roi "29,8,6,48" --report ".\experiments\local\p2c-v1\results\channel-p2a-white-audit.json"

cargo run -p scansketch-cli --bin scansketch-white-audit -- --source ".\experiments\local\p2c-v1\square-white-channel.png" --preview ".\experiments\local\p2c-v1\results\square-white-channel-p2b1.png" --max-size 64 --roi "29,8,6,48" --report ".\experiments\local\p2c-v1\results\channel-p2b1-white-audit.json"
```

If those report filenames already exist, choose new names rather than deleting/overwriting prior evidence. For a compact summary:

```powershell
@("p2a","p2b1") | ForEach-Object {
  $mode = $_
  $a = Get-Content ".\experiments\local\p2c-v1\results\channel-$mode-white-audit.json" -Raw | ConvertFrom-Json
  [pscustomobject]@{
    Mode = $mode
    SourceWhite = $a.source_white_pixels
    AffectedWhite = $a.affected_white_pixels
    ChannelWhite = $a.roi_source_white_pixels
    AffectedInsideChannel = $a.roi_affected_white_pixels
    AffectedOutsideChannel = $a.outside_roi_affected_white_pixels
    NearDark1px = $a.near_source_nonwhite_1px
    NearDark2px = $a.near_source_nonwhite_2px
    FartherThan2px = $a.farther_than_2px
  }
} | Format-Table -AutoSize
```

Paste the summary (safe synthetic data) here. The individual reports retain coordinates for deeper analysis. An *inside-channel count of zero* would narrow the concern to exterior/background borders; a nonzero count requires inspecting interior pixel positions and the actual stroke paths before deciding on a fix.

## Status gate

CI runs both audits on newly-generated synthetic source/preview data and checks equivalence with `p2c-v1` white-mask counts plus exactly 288 protected target-white ROI pixels. Record the **actual CI/local outcomes** only after they run. Proceed to one permitted nonportrait comparison once the 53-pixel distribution is understood; P3-A remains gated.
