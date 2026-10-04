# ScanSketch P2-C: paired same-binary synthetic fixture experiment.
# Local-only output. Does not mutate Git, source fixture images or past reports.
# Run from any directory: powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-p2c-synthetic.ps1
param(
    [string]$FixtureDir = "experiments/local/p2c-v1",
    [string]$OutputDir = "experiments/local/p2c-v1/results"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $root

function Resolve-FromRoot([string]$path) {
    if ([System.IO.Path]::IsPathRooted($path)) { return $path }
    return (Join-Path $root $path)
}
$sourceDir = Resolve-FromRoot $FixtureDir
$destination = Resolve-FromRoot $OutputDir
$names = @("step", "square-white-channel", "gradient")

if (-not (Test-Path -LiteralPath $sourceDir -PathType Container)) {
    throw "Fixture folder missing: $sourceDir. Generate it using scansketch-fixtures first."
}
foreach ($name in $names) {
    if (-not (Test-Path -LiteralPath (Join-Path $sourceDir "$name.png") -PathType Leaf)) {
        throw "Fixture source missing: $name.png. Stop before running partial experiments."
    }
}
# Never replace the prior experiment. Supply -OutputDir with a NEW path for a rerun.
if (Test-Path -LiteralPath $destination) {
    throw "Output path already exists: $destination. Choose a new -OutputDir to preserve prior results."
}
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$commit = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw "Unable to resolve git HEAD" }
$rows = @()
foreach ($name in $names) {
    $source = Join-Path $sourceDir "$name.png"
    $sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
    foreach ($mode in @("p2a", "p2b1")) {
        $prefix = "$name-$mode"
        $png = Join-Path $destination "$prefix.png"
        $strokes = Join-Path $destination "$prefix-strokes.json"
        $report = Join-Path $destination "$prefix-report.json"
        $renderArgs = @(
            "run", "-p", "scansketch-cli", "--bin", "scansketch", "--",
            "--input", $source, "--output", $png, "--strokes", $strokes,
            "--max-size", "64", "--seed", "42"
        )
        if ($mode -eq "p2a") { $renderArgs += "--no-contours" }

        Write-Host "Rendering $name / $mode"
        & cargo @renderArgs
        if ($LASTEXITCODE -ne 0) { throw "Render failed: $name / $mode" }

        $measurementArgs = @(
            "run", "-p", "scansketch-cli", "--bin", "scansketch-measure", "--",
            "--source", $source, "--preview", $png, "--strokes", $strokes,
            "--max-size", "64", "--report", $report
        )
        Write-Host "Measuring $name / $mode"
        & cargo @measurementArgs | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Measurement failed: $name / $mode" }

        $m = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
        $rows += [pscustomobject]@{
            fixture = $name
            mode = $mode
            seed = 42
            max_side = 64
            git_sha = $commit
            source_sha256 = $sourceHash
            preview_sha256 = (Get-FileHash -LiteralPath $png -Algorithm SHA256).Hash
            strokes_sha256 = (Get-FileHash -LiteralPath $strokes -Algorithm SHA256).Hash
            stroke_count = $m.strokes.count
            total_path_length_px = $m.strokes.total_path_length_px
            tone_rmse = $m.tone_rmse
            white_rmse = $m.white_region.rmse
            unwanted_highlight_ink_fraction = $m.unwanted_highlight_ink_fraction
            edge_precision = $m.edges.precision
            edge_recall = $m.edges.recall
            edge_f1 = $m.edges.f1
            render_path = $png
            strokes_path = $strokes
            report_path = $report
        }
    }
}

$summaryPath = Join-Path $destination "summary.json"
[pscustomobject]@{
    protocol_revision = "p2c-v1"
    comparison_lane = "same-binary natural-output; NOT matched-budget"
    git_sha = $commit
    fixtures = $rows
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summaryPath -Encoding UTF8

$rows | Select-Object fixture, mode, stroke_count, tone_rmse, edge_f1, unwanted_highlight_ink_fraction | Format-Table -AutoSize
Write-Host "All 6 paired runs succeeded. Local manifest: $summaryPath"
Write-Host "Inspect corresponding PNGs before judging artistic quality; share summary metrics, not private source images."
