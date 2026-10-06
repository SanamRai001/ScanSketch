# P4-A.3 proof: legacy control vs frozen hierarchy vs residual-hatch composition.
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
    $m = [regex]::Match($text, '(?:^|:|\|)\s*' + [regex]::Escape($name) + '=([^\s|]+)')
    if (-not $m.Success) { throw "Unable to parse '$name' from: $text" }
    $value = 0.0
    if (-not [double]::TryParse(
        $m.Groups[1].Value,
        [System.Globalization.NumberStyles]::Float,
        [System.Globalization.CultureInfo]::InvariantCulture,
        [ref]$value
    )) {
        throw "Unable to parse numeric '$name' value '$($m.Groups[1].Value)' from: $text"
    }
    return $value
}
function JsonCompact($value) {
    return (ConvertTo-Json -InputObject @($value) -Depth 30 -Compress)
}

if ($Seed -lt 0) { throw "-Seed must be nonnegative." }
if (-not $RightsConfirmed) {
    throw "Pass -RightsConfirmed only for an image you may process locally."
}

$source = (Resolve-Path -LiteralPath (FromRoot $InputImage) -ErrorAction Stop).Path
$destination = FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Output already exists; choose a NEW -OutputDir: $destination"
}
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$commit = (& git rev-parse HEAD).Trim()
$sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
$rows = @()
$residualStats = $null

foreach ($mode in @("p2b1","p4a2-paths","p4a3-final","p4a3-hatches")) {
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
    if ($mode -eq "p4a2-paths") { $args += "--stroke-hierarchy-only" }
    if ($mode -eq "p4a3-final") { $args += "--residual-hatching" }
    if ($mode -eq "p4a3-hatches") { $args += "--residual-hatching-only" }

    Write-Host "Rendering $mode / max-side $MaxSize / seed $Seed"
    $renderOutput = @(& cargo @args)
    $exit = $LASTEXITCODE
    $renderOutput | ForEach-Object { Write-Host $_ }
    if ($exit -ne 0) { throw "Rendering failed for $mode (cargo exit $exit)" }

    if ($mode -eq "p4a3-final") {
        $line = $renderOutput | Where-Object { "$_" -like "P4-A.3 residual:*" } | Select-Object -Last 1
        if (-not $line) { throw "Missing P4-A.3 statistics line" }
        $text = "$line"
        $residualStats = [pscustomobject]@{
            gestures = Read-IntStat $text "gestures"
            forms = Read-IntStat $text "forms"
            hatches = Read-IntStat $text "hatches"
            control_segments = Read-IntStat $text "control-segments"
            hatch_budget = Read-IntStat $text "hatch-budget"
            residual_pixels = Read-IntStat $text "residual-pixels"
            hatch_mean = Read-FloatStat $text "hatch-mean"
            structure_share = Read-FloatStat $text "structure-share"
            hatch_share = Read-FloatStat $text "hatch-share"
            hatch_reduction = Read-FloatStat $text "hatch-reduction"
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
        hatch_count = $m.paths.hatch_count
        logical_marks = $m.marks.logical_count
        total_path_length_px = $m.paths.total_path_length_px
        tone_rmse = $m.tone_rmse
        midtone_rmse = $m.midtone_region.rmse
        dark_rmse = $m.dark_region.rmse
        white_rmse = $m.white_region.rmse
        edge_f1 = $m.edges.f1
        highlight_ink = $m.unwanted_highlight_ink_fraction
    }
}

if (-not $residualStats) { throw "P4-A.3 statistics missing" }

$base = Get-Content -LiteralPath (Join-Path $destination "p2b1.json") -Raw | ConvertFrom-Json
$hierarchy = Get-Content -LiteralPath (Join-Path $destination "p4a2-paths.json") -Raw | ConvertFrom-Json
$final = Get-Content -LiteralPath (Join-Path $destination "p4a3-final.json") -Raw | ConvertFrom-Json
$hatchOnly = Get-Content -LiteralPath (Join-Path $destination "p4a3-hatches.json") -Raw | ConvertFrom-Json

if ($final.strokes.Count -ne 0 -or $hatchOnly.strokes.Count -ne 0) {
    throw "P4-A.3 must contain zero legacy straight segments"
}

$hierarchyPaths = @()
$finalPaths = @()
$hatchPaths = @()
if ($hierarchy.PSObject.Properties.Name -contains "paths") { $hierarchyPaths = @($hierarchy.paths) }
if ($final.PSObject.Properties.Name -contains "paths") { $finalPaths = @($final.paths) }
if ($hatchOnly.PSObject.Properties.Name -contains "paths") { $hatchPaths = @($hatchOnly.paths) }

$finalStructure = @($finalPaths | Where-Object { $_.role -ne "hatch" })
$finalHatches = @($finalPaths | Where-Object { $_.role -eq "hatch" })

if ($hierarchyPaths.Count -ne $finalStructure.Count) {
    throw "P4-A.3 changed frozen Gesture/Form hierarchy count"
}
if ($hierarchyPaths.Count -gt 0 -and (JsonCompact $hierarchyPaths) -ne (JsonCompact $finalStructure)) {
    throw "P4-A.3 changed frozen Gesture/Form hierarchy"
}
if ($finalHatches.Count -ne $hatchPaths.Count) {
    throw "Final and hatch-only modes generated different Hatch counts"
}
if ($finalHatches.Count -gt 0 -and (JsonCompact $finalHatches) -ne (JsonCompact $hatchPaths)) {
    throw "Final and hatch-only modes generated different Hatch layers"
}
if ($finalHatches.Count -ne $residualStats.hatches) {
    throw "Reported Hatch count differs from serialized Hatch count"
}
if ($base.strokes.Count -ne $residualStats.control_segments) {
    throw "Reported legacy control count differs from P2-B.1 JSON"
}
if ($residualStats.hatches -gt $residualStats.hatch_budget) {
    throw "P4-A.3 exceeded its sparse Hatch budget"
}
if ($residualStats.control_segments -gt 0 -and $residualStats.hatch_reduction -lt 0.40) {
    throw "P4-A.3 failed the initial >=40% Hatch-count reduction gate"
}
foreach ($path in $finalHatches) {
    if ($path.points.Count -ne 2) { throw "P4-A.3 Hatch is not a short two-point mark" }
}

$summary = [pscustomobject]@{
    phase = "p4a3-residual-hatching"
    git_sha = $commit
    source_name = [System.IO.Path]::GetFileName($source)
    source_sha256 = $sourceHash
    seed = $Seed
    max_side = $MaxSize
    rights_confirmed_by_operator = $true
    structure_exactly_preserved = $true
    final_hatches_equal_hatch_only = $true
    residual_stats = $residualStats
    fixtures = $rows
}
$summaryPath = Join-Path $destination "summary.json"
$summary | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $summaryPath -Encoding UTF8

$rows |
    Select-Object mode,segment_count,gesture_count,form_count,hatch_count,logical_marks,total_path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1 |
    Format-Table -AutoSize

Write-Host ("P4-A.3: control={0}; hatches={1}; reduction={2:P1}; hatch-mean={3}px; structure-share={4:P1}" -f $residualStats.control_segments,$residualStats.hatches,$residualStats.hatch_reduction,$residualStats.hatch_mean,$residualStats.structure_share)
Write-Host "Inspect p4a2-paths.png -> p4a3-final.png -> p4a3-hatches.png."
Write-Host "Local manifest: $summaryPath"
