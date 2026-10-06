# P4-A.2 - Real Nonportrait Pack Runbook

P4-A.2 is the second stroke-hierarchy gate.

The comparison is:

```text
P4-A.1 = long Gesture paths only
P4-A.2 = exact same Gestures + new 8-32px Form paths
```

The old horizontal hatch field is still deliberately frozen until P4-A.3.

## Reuse the same source pack

Use the existing 3-5 permission-cleared nonportrait photographs, ideally the same chair/mug/plant folder used by P3-G1 and P4-A.1:

```text
experiments/local/p3g1/sources/
```

## Switch branch

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p4a2-medium-form-paths
```

If already present:

```powershell
git switch feat/p4a2-medium-form-paths
git pull --ff-only
```

Optional:

```powershell
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
```

## Run the pack

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p4a2-pack.ps1" `
  -InputDir ".\experiments\local\p3g1\sources" `
  -OutputDir ".\experiments\local\p4a2\run-a" `
  -MaxSize 512 `
  -Seed 42 `
  -RightsConfirmed `
  -NonPortraitConfirmed
```

One-line version:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p4a2-pack.ps1" -InputDir ".\experiments\local\p3g1\sources" -OutputDir ".\experiments\local\p4a2\run-a" -MaxSize 512 -Seed 42 -RightsConfirmed -NonPortraitConfirmed
```

Use a new output directory.

## Open the visual review

```powershell
Start-Process ".\experiments\local\p4a2\run-a\review.html"
```

The page uses ASCII-only headings/punctuation to avoid the encoding artifacts seen in an earlier generated review.

For each source it shows:

1. P4-A.1 Gestures only;
2. P4-A.2 Gestures + Forms only;
3. P4-A.2 hierarchy overlay.

## Review the center column first

Ask:

- What useful structure appeared that P4-A.1 did not contain?
- Are new marks clearly medium-scale and subordinate to Gestures?
- Do they describe internal form rather than redraw silhouette?
- Is the hierarchy still sparse and readable?
- Do any medium marks wander or become arbitrary texture?

Expected examples:

### Chair
- seat plane;
- slats;
- brace/leg transitions;
- back-seat junction.

### Mug
- inner/secondary rim;
- handle interior;
- shorter base/body transitions.

### Plant
- secondary leaf boundaries;
- veins if source-supported;
- pot rim/plane;
- stem details.

## Print evidence

```powershell
$s = Get-Content ".\experiments\local\p4a2\run-a\summary.json" -Raw | ConvertFrom-Json

$s.counts
$s.means

$s.cases |
    Select-Object source_name,gesture_count,form_count,form_seed_count,gesture_mean_px,form_mean_px,form_max_px,form_length_share,hierarchy_edge_f1_change_vs_p4a1_pct |
    Format-Table -AutoSize
```

Share that output plus the P4-A.1/P4-A.2 paths-only review screenshots. Original photographs do not need to be uploaded.

## Decision

P4-A.2 passes if multiple source classes gain useful medium structure without:
- changing Gestures;
- excessive silhouette duplication;
- path-count explosion;
- unsupported wandering.

If it passes, proceed to **P4-A.3 residual hatching rebalance**. That is the phase where ScanSketch finally stops covering the whole object with the old horizontal dash field.
