# P4-A.3 — Real Image Runbook

P4-A.3 is the first P4 candidate that does **not** contain the old P2-B.1 straight-stroke field.

## Switch branch

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p4a3-residual-hatching
```

If already local:

```powershell
git switch feat/p4a3-residual-hatching
git pull --ff-only
```

## Run one image

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p4a3-ab.ps1" `
  -InputImage ".\experiments\local\p3g1\sources\mug.jpg" `
  -OutputDir ".\experiments\local\p4a3\mug-a" `
  -MaxSize 512 `
  -Seed 42 `
  -RightsConfirmed
```

Use a new output directory for every run.

## Inspect in this order

```powershell
Start-Process ".\experiments\local\p4a3\mug-a\p4a3-final.png"
Start-Process ".\experiments\local\p4a3\mug-a\p4a2-paths.png"
Start-Process ".\experiments\local\p4a3\mug-a\p4a3-hatches.png"
Start-Process ".\experiments\local\p4a3\mug-a\p2b1.png"
```

Judge `p4a3-final.png` first.

## Print evidence

```powershell
$s = Get-Content ".\experiments\local\p4a3\mug-a\summary.json" -Raw | ConvertFrom-Json

$s.residual_stats

$s.fixtures |
  Select-Object mode,segment_count,gesture_count,form_count,hatch_count,logical_marks,total_path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1 |
  Format-Table -AutoSize
```

## Chair / mug / plant

After the first run looks sane, repeat with the same three sources used in P3-G1/P4-A.1/P4-A.2.

The original photos remain local. Share generated review images only when permitted.

## What to look for

A good result should make this visual transition obvious:

```text
P2-B.1:
horizontal tone field → object

P4-A.3:
Gesture → Form → supporting Hatch tone
```

If P4-A.3 still reads primarily as horizontal bands despite the large count reduction, do not increase or decrease density blindly. The next controlled variable would be Hatch orientation.
