# P4-A.2 proof: frozen P4-A.1 Gestures + new medium Form paths.
param(
    [Parameter(Mandatory = $true)][string]$InputImage,
    [Parameter(Mandatory = $true)][string]$OutputDir,
    [ValidateRange(32,1024)][int]$MaxSize = 512,
    [long]$Seed = 42,
    [switch]$RightsConfirmed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
Set-Location $root

function FromRoot([string]$path) {
    if ([System.IO.Path]::IsPathRooted($path)) { return $path }
    return Join-Path $root $path
}
function Read-IntStat([string]$text, [string]$name) {
    $m = [regex]::Match($text, '(?:^|:|\|)\s*' + [regex]::Escape($name) + '=([0-9]+)')
    if (-not $m.Success) { throw "Unable to parse '$name' from: $text" }
    return [int]$m.Groups[1].Value
}
function Read-FloatStat([string]$text, [string]$name) {
    $m = [regex]::Match($text, '(?:^|:|\|)\s*' + [regex]::Escape($name) + '=([-+]?[0-9]+(?:\.[0-9]+)?)')
    if (-not $m.Success) { throw "Unable to parse '$name' from: $text" }
    return [double]$m.Groups[1].Value
}

if ($Seed -lt 0) { throw "-Seed must be nonnegative." }
if (-not $RightsConfirmed) { throw "Pass -RightsConfirmed for an image you may process locally." }

$source = (Resolve-Path -LiteralPath (FromRoot $InputImage) -ErrorAction Stop).Path
$destination = FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Output already exists; choose a NEW -OutputDir: $destination"
}
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$commit = (& git rev-parse HEAD).Trim()
$sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
$rows = @()
$hierarchyStats = $null

foreach ($mode in @("p2b1","p4a1-paths","p4a2-paths","p4a2-overlay")) {
    $png = Join-Path $destination "$mode.png"
    $json = Join-Path $destination "$mode.json"
    $report = Join-Path $destination "$mode-report.json"

    $args = @(
        "run","-p","scansketch-cli","--bin","scansketch","--",
        "--input",$source,
        "--output",$png,
        "--strokes",$json,
        "--max-size",[string]$MaxSize,
        "--seed",[string]$Seed
    )
    if ($mode -eq "p4a1-paths") { $args += "--long-structural-only" }
    if ($mode -eq "p4a2-paths") { $args += "--stroke-hierarchy-only" }
    if ($mode -eq "p4a2-overlay") { $args += "--stroke-hierarchy" }

    Write-Host "Rendering $mode / max-side $MaxSize / seed $Seed"
    $renderOutput = @(& cargo @args)
    $exit = $LASTEXITCODE
    $renderOutput | ForEach-Object { Write-Host $_ }
    if ($exit -ne 0) { throw "Rendering failed for $mode (cargo exit $exit)" }

    if ($mode -eq "p4a2-overlay") {
        $line = $renderOutput | Where-Object { "$_" -like "P4-A.2 hierarchy:*" } | Select-Object -Last 1
        if (-not $line) { throw "Missing P4-A.2 statistics line" }
        $text = "$line"
        $hierarchyStats = [pscustomobject]@{
            gestures = Read-IntStat $text "gestures"
            forms = Read-IntStat $text "forms"
            form_seeds = Read-IntStat $text "form-seeds"
            form_overlap_rejected = Read-IntStat $text "form-overlap-rejected"
            baseline_segments = Read-IntStat $text "baseline-segments"
            gesture_mean = Read-FloatStat $text "gesture-mean"
            form_mean = Read-FloatStat $text "form-mean"
            form_max = Read-FloatStat $text "form-max"
            gesture_share = Read-FloatStat $text "gesture-share"
            form_share = Read-FloatStat $text "form-share"
        }
    }

    $measureArgs = @(
        "run","-p","scansketch-cli","--bin","scansketch-measure","--",
        "--source",$source,
        "--preview",$png,
        "--strokes",$json,
        "--max-size",[string]$MaxSize,
        "--report",$report
    )
    & cargo @measureArgs | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Measurement failed for $mode" }

    $m = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    $rows += [pscustomobject]@{
        mode = $mode
        segment_count = $m.strokes.count
        path_count = $m.paths.count
        gesture_count = $m.paths.gesture_count
        form_count = $m.paths.form_count
        logical_marks = $m.marks.logical_count
        total_path_length_px = $m.paths.total_path_length_px
        mean_path_length_px = $m.paths.mean_path_length_px
        tone_rmse = $m.tone_rmse
        midtone_rmse = $m.midtone_region.rmse
        dark_rmse = $m.dark_region.rmse
        white_rmse = $m.white_region.rmse
        edge_f1 = $m.edges.f1
    }
}

if (-not $hierarchyStats) { throw "P4-A.2 statistics missing" }

$base = Get-Content -LiteralPath (Join-Path $destination "p2b1.json") -Raw | ConvertFrom-Json
$p4a1 = Get-Content -LiteralPath (Join-Path $destination "p4a1-paths.json") -Raw | ConvertFrom-Json
$p4a2 = Get-Content -LiteralPath (Join-Path $destination "p4a2-paths.json") -Raw | ConvertFrom-Json
$overlay = Get-Content -LiteralPath (Join-Path $destination "p4a2-overlay.json") -Raw | ConvertFrom-Json

if ($base.strokes.Count -ne $overlay.strokes.Count) {
    throw "P4-A.2 overlay changed frozen baseline segment count"
}
for ($i=0; $i -lt $base.strokes.Count; $i++) {
    $a=$base.strokes[$i]; $b=$overlay.strokes[$i]
    if ($a.x0 -ne $b.x0 -or $a.y0 -ne $b.y0 -or $a.x1 -ne $b.x1 -or $a.y1 -ne $b.y1 -or $a.width -ne $b.width -or $a.opacity -ne $b.opacity) {
        throw "P4-A.2 overlay changed frozen baseline segment at index $i"
    }
}
if ($p4a1.strokes.Count -ne 0 -or $p4a2.strokes.Count -ne 0) {
    throw "Path-only P4 output unexpectedly contains legacy segments"
}

$p4a1Paths = if ($p4a1.PSObject.Properties.Name -contains "paths") { @($p4a1.paths) } else { @() }
$p4a2Paths = if ($p4a2.PSObject.Properties.Name -contains "paths") { @($p4a2.paths) } else { @() }
$overlayPaths = if ($overlay.PSObject.Properties.Name -contains "paths") { @($overlay.paths) } else { @() }

$p4a2Gestures = @($p4a2Paths | Where-Object { $_.role -eq "gesture" })
$p4a2Forms = @($p4a2Paths | Where-Object { $_.role -eq "form" })

if (($p4a1Paths | ConvertTo-Json -Depth 20 -Compress) -ne ($p4a2Gestures | ConvertTo-Json -Depth 20 -Compress)) {
    throw "P4-A.2 changed the frozen P4-A.1 Gesture layer"
}
if (($p4a2Paths | ConvertTo-Json -Depth 20 -Compress) -ne ($overlayPaths | ConvertTo-Json -Depth 20 -Compress)) {
    throw "P4-A.2 overlay/path-only hierarchy mismatch"
}
if ($p4a2Gestures.Count -ne $hierarchyStats.gestures -or $p4a2Forms.Count -ne $hierarchyStats.forms) {
    throw "P4-A.2 role-count accounting mismatch"
}

function Path-Length([object]$path) {
    $sum = 0.0
    for ($i=1; $i -lt $path.points.Count; $i++) {
        $dx = [double]$path.points[$i].x - [double]$path.points[$i-1].x
        $dy = [double]$path.points[$i].y - [double]$path.points[$i-1].y
        $sum += [math]::Sqrt($dx*$dx + $dy*$dy)
    }
    return $sum
}
foreach ($form in $p4a2Forms) {
    $length = Path-Length $form
    if ($length -lt 7.999 -or $length -gt 32.001) {
        throw "Form path escaped 8-32px target band: $length"
    }
}

$summary = [pscustomobject]@{
    phase = "p4a2-medium-form-paths"
    git_sha = $commit
    source_name = [System.IO.Path]::GetFileName($source)
    source_sha256 = $sourceHash
    seed = $Seed
    max_side = $MaxSize
    rights_confirmed_by_operator = $true
    baseline_segments_exactly_preserved = $true
    p4a1_gestures_exactly_preserved = $true
    overlay_paths_equal_paths_only = $true
    hierarchy_stats = $hierarchyStats
    fixtures = $rows
}
$summaryPath = Join-Path $destination "summary.json"
$summary | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $summaryPath -Encoding UTF8

$rows |
    Select-Object mode,segment_count,path_count,gesture_count,form_count,logical_marks,total_path_length_px,mean_path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1 |
    Format-Table -AutoSize

Write-Host ("P4-A.2 hierarchy: gestures={0}; forms={1}; form-seeds={2}; gesture-mean={3}px; form-mean={4}px; form-max={5}px" -f $hierarchyStats.gestures,$hierarchyStats.forms,$hierarchyStats.form_seeds,$hierarchyStats.gesture_mean,$hierarchyStats.form_mean,$hierarchyStats.form_max)
Write-Host "Inspect p4a1-paths.png vs p4a2-paths.png FIRST; the difference must be useful medium Form structure."
Write-Host "Local manifest: $summaryPath"
