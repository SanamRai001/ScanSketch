# P2-C: paired photograph/object test using the exact same checked-out renderer.
# Research outputs stay in an ignored, NEW local directory. No network or Git writes.
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

function Resolve-FromRoot([string]$path) {
    if ([System.IO.Path]::IsPathRooted($path)) { return $path }
    return Join-Path $root $path
}
if ($Seed -lt 0) { throw '-Seed must be nonnegative.' }
if (-not $RightsConfirmed) {
    throw "Confirm you own or have permission to use this local input by passing -RightsConfirmed. This script never uploads the image."
}
$source = (Resolve-Path -LiteralPath (Resolve-FromRoot $InputImage) -ErrorAction Stop).Path
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
    throw "Input must be a PNG or JPEG file."
}
if ([System.IO.Path]::GetExtension($source).ToLowerInvariant() -notin @(".png",".jpg",".jpeg")) {
    throw "Input must end in .png, .jpg, or .jpeg."
}
$destination = Resolve-FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Refusing to overwrite an existing experiment folder: $destination. Choose a NEW -OutputDir."
}

$commit = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
    throw "Could not read the checked-out Git commit."
}
$inputHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
New-Item -ItemType Directory -Path $destination -Force | Out-Null
$rows = @()

foreach ($mode in @("p2a","p2b1")) {
    $preview = Join-Path $destination "$mode.png"
    $strokes = Join-Path $destination "$mode-strokes.json"
    $report = Join-Path $destination "$mode-report.json"
    $renderArgs = @(
        "run","-p","scansketch-cli","--bin","scansketch","--",
        "--input",$source,"--output",$preview,"--strokes",$strokes,
        "--max-size",[string]$MaxSize,"--seed",[string]$Seed
    )
    if ($mode -eq "p2a") { $renderArgs += "--no-contours" }

    Write-Host "Rendering $mode at max side $MaxSize (seed $Seed)"
    & cargo @renderArgs
    if ($LASTEXITCODE -ne 0) { throw "Render failed for $mode" }

    $measureArgs = @(
        "run","-p","scansketch-cli","--bin","scansketch-measure","--",
        "--source",$source,"--preview",$preview,"--strokes",$strokes,
        "--max-size",[string]$MaxSize,"--report",$report
    )
    Write-Host "Measuring $mode"
    & cargo @measureArgs | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Measurement failed for $mode" }
    $m = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    if ([long]$m.seed -ne $Seed) { throw "Unexpected seed in $mode output" }

    $rows += [pscustomobject]@{
        mode = $mode
        protocol_revision = $m.protocol_revision
        git_sha = $commit
        seed = $Seed
        max_side = $MaxSize
        width = $m.width
        height = $m.height
        source_sha256 = $inputHash
        preview_sha256 = (Get-FileHash -LiteralPath $preview -Algorithm SHA256).Hash
        strokes_sha256 = (Get-FileHash -LiteralPath $strokes -Algorithm SHA256).Hash
        stroke_count = $m.strokes.count
        total_path_length_px = $m.strokes.total_path_length_px
        tone_rmse = $m.tone_rmse
        white_rmse = $m.white_region.rmse
        midtone_rmse = $m.midtone_region.rmse
        dark_rmse = $m.dark_region.rmse
        unwanted_highlight_ink_fraction = $m.unwanted_highlight_ink_fraction
        edge_source_pixels = $m.edges.source_pixels
        edge_preview_pixels = $m.edges.preview_pixels
        edge_precision = $m.edges.precision
        edge_recall = $m.edges.recall
        edge_f1 = $m.edges.f1
        preview_path = $preview
        strokes_path = $strokes
        report_path = $report
    }
}

$summaryPath = Join-Path $destination "summary.json"
[pscustomobject]@{
    protocol_revision = "p2c-v1"
    comparison_lane = "same-binary natural-output; NOT matched-budget"
    source_name = [System.IO.Path]::GetFileName($source)
    rights_confirmed_by_operator = $true
    source_sha256 = $inputHash
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    fixtures = $rows
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summaryPath -Encoding UTF8

$rows | Select-Object mode, stroke_count, tone_rmse, white_rmse, dark_rmse, edge_f1, unwanted_highlight_ink_fraction | Format-Table -AutoSize
Write-Host "Paired run complete. Local-only manifest: $summaryPath"
Write-Host "Inspect BOTH paired previews at 100%. Never publish the original photo or generated outputs without permission."
