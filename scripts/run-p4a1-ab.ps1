# P4-A.1 proof: frozen P2-B.1 + long structural overlay + gesture layer only.
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
if (-not $RightsConfirmed) { throw "Pass -RightsConfirmed for an image you may process locally. Nothing is uploaded." }

$source = (Resolve-Path -LiteralPath (FromRoot $InputImage) -ErrorAction Stop).Path
$destination = FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Output already exists; choose a NEW -OutputDir: $destination"
}
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$commit = (& git rev-parse HEAD).Trim()
$sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
$rows = @()
$longStats = $null

foreach ($mode in @("p2b1","p4a1-overlay","p4a1-paths")) {
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
    if ($mode -eq "p4a1-overlay") { $args += "--long-structural" }
    if ($mode -eq "p4a1-paths") { $args += "--long-structural-only" }

    Write-Host "Rendering $mode / max-side $MaxSize / seed $Seed"
    $renderOutput = @(& cargo @args)
    $exit = $LASTEXITCODE
    $renderOutput | ForEach-Object { Write-Host $_ }
    if ($exit -ne 0) { throw "Rendering failed for $mode (cargo exit $exit)" }

    if ($mode -eq "p4a1-overlay") {
        $line = $renderOutput | Where-Object { "$_" -like "P4-A.1 long paths:*" } | Select-Object -Last 1
        if (-not $line) { throw "Missing P4-A.1 statistics line" }
        $text = "$line"
        $longStats = [pscustomobject]@{
            seeds = Read-IntStat $text "seeds"
            accepted = Read-IntStat $text "accepted"
            baseline_segments = Read-IntStat $text "baseline-segments"
            total_path = Read-FloatStat $text "total-path"
            mean_path = Read-FloatStat $text "mean-path"
            max_path = Read-FloatStat $text "max-path"
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
        logical_marks = $m.marks.logical_count
        path_length_px = $m.paths.total_path_length_px
        mean_path_length_px = $m.paths.mean_path_length_px
        tone_rmse = $m.tone_rmse
        midtone_rmse = $m.midtone_region.rmse
        dark_rmse = $m.dark_region.rmse
        white_rmse = $m.white_region.rmse
        edge_f1 = $m.edges.f1
        preview_path = $png
        json_path = $json
        report_path = $report
    }
}

if (-not $longStats) { throw "P4-A.1 statistics missing" }

$base = Get-Content -LiteralPath (Join-Path $destination "p2b1.json") -Raw | ConvertFrom-Json
$overlay = Get-Content -LiteralPath (Join-Path $destination "p4a1-overlay.json") -Raw | ConvertFrom-Json
$pathsOnly = Get-Content -LiteralPath (Join-Path $destination "p4a1-paths.json") -Raw | ConvertFrom-Json

$baseStrokes = @()
$overlayStrokes = @()
$onlyStrokes = @()
if ($base.PSObject.Properties.Name -contains "strokes") { $baseStrokes = @($base.strokes) }
if ($overlay.PSObject.Properties.Name -contains "strokes") { $overlayStrokes = @($overlay.strokes) }
if ($pathsOnly.PSObject.Properties.Name -contains "strokes") { $onlyStrokes = @($pathsOnly.strokes) }

if ($baseStrokes.Count -ne $overlayStrokes.Count) {
    throw "P4-A.1 overlay changed frozen baseline segment count"
}
for ($i=0; $i -lt $baseStrokes.Count; $i++) {
    $a=$baseStrokes[$i]; $b=$overlayStrokes[$i]
    if ($a.x0 -ne $b.x0 -or $a.y0 -ne $b.y0 -or $a.x1 -ne $b.x1 -or $a.y1 -ne $b.y1 -or $a.width -ne $b.width -or $a.opacity -ne $b.opacity) {
        throw "P4-A.1 overlay changed frozen baseline segment at index $i"
    }
}
if ($onlyStrokes.Count -ne 0) {
    throw "P4-A.1 paths-only output unexpectedly contains legacy segments"
}

$overlayPaths = @()
$onlyPaths = @()
if ($overlay.PSObject.Properties.Name -contains "paths") { $overlayPaths = @($overlay.paths) }
if ($pathsOnly.PSObject.Properties.Name -contains "paths") { $onlyPaths = @($pathsOnly.paths) }
if ($overlayPaths.Count -ne $onlyPaths.Count) {
    throw "Overlay/path-only gesture counts differ"
}
if ($overlayPaths.Count -ne $longStats.accepted) {
    throw "Reported accepted path count differs from JSON path count"
}
$overlayCompact = $overlayPaths | ConvertTo-Json -Depth 20 -Compress
$onlyCompact = $onlyPaths | ConvertTo-Json -Depth 20 -Compress
if ($overlayCompact -ne $onlyCompact) {
    throw "Overlay and path-only modes did not produce exact same logical paths"
}

$summary = [pscustomobject]@{
    phase = "p4a1-long-structural-paths"
    git_sha = $commit
    source_name = [System.IO.Path]::GetFileName($source)
    source_sha256 = $sourceHash
    seed = $Seed
    max_side = $MaxSize
    rights_confirmed_by_operator = $true
    baseline_segments_exactly_preserved = $true
    overlay_paths_equal_paths_only = $true
    long_path_stats = $longStats
    fixtures = $rows
}
$summaryPath = Join-Path $destination "summary.json"
$summary | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $summaryPath -Encoding UTF8

$rows |
    Select-Object mode,segment_count,path_count,logical_marks,path_length_px,mean_path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1 |
    Format-Table -AutoSize

Write-Host ("P4-A.1 long paths: seeds={0}; accepted={1}; mean={2}px; max={3}px" -f $longStats.seeds,$longStats.accepted,$longStats.mean_path,$longStats.max_path)
Write-Host "Inspect p4a1-paths.png FIRST, then compare p2b1.png vs p4a1-overlay.png."
Write-Host "Local manifest: $summaryPath"
