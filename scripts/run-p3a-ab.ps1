# P3-A.0 proof: same-photo P2-B.1 control versus opt-in directional mode.
# Both use identical source, seed, contours and original tonal proposal COUNT.
# No original/previous results are overwritten; no network calls or Git writes.
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
    return (Join-Path $root $path)
}
if ($Seed -lt 0) { throw "-Seed must be nonnegative." }
if (-not $RightsConfirmed) { throw "Pass -RightsConfirmed for an image you may use. Nothing is uploaded." }
$source = (Resolve-Path -LiteralPath (FromRoot $InputImage) -ErrorAction Stop).Path
if ([System.IO.Path]::GetExtension($source).ToLowerInvariant() -notin @(".png",".jpg",".jpeg")) {
    throw "Use a .png, .jpg or .jpeg input"
}
$destination = FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Output already exists; supply NEW -OutputDir to retain old results: $destination"
}
$commit = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) { throw "Could not read Git HEAD" }
$hash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$rows=@()
foreach ($mode in @("p2b1","p3a")) {
    $png = Join-Path $destination "$mode.png"
    $json = Join-Path $destination "$mode-strokes.json"
    $report = Join-Path $destination "$mode-report.json"
    $renderArgs=@("run","-p","scansketch-cli","--bin","scansketch","--",
        "--input",$source,"--output",$png,"--strokes",$json,
        "--max-size",[string]$MaxSize,"--seed",[string]$Seed)
    if ($mode -eq "p3a") { $renderArgs += "--directional" }
    Write-Host "Rendering $mode / max-side $MaxSize / seed $Seed"
    & cargo @renderArgs
    if ($LASTEXITCODE -ne 0) { throw "Rendering $mode failed" }

    $measureArgs=@("run","-p","scansketch-cli","--bin","scansketch-measure","--",
        "--source",$source,"--preview",$png,"--strokes",$json,
        "--max-size",[string]$MaxSize,"--report",$report)
    & cargo @measureArgs | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Measuring $mode failed" }
    $m=Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    if ([long]$m.seed -ne $Seed) { throw "Seed mismatch" }
    $rows += [pscustomobject]@{
        mode=$mode; protocol_revision=$m.protocol_revision; git_sha=$commit
        source_sha256=$hash; preview_sha256=(Get-FileHash -LiteralPath $png -Algorithm SHA256).Hash
        strokes_sha256=(Get-FileHash -LiteralPath $json -Algorithm SHA256).Hash
        max_side=$MaxSize; seed=$Seed; width=$m.width; height=$m.height
        strokes=$m.strokes.count; path_length_px=$m.strokes.total_path_length_px
        tone_rmse=$m.tone_rmse; midtone_rmse=$m.midtone_region.rmse
        dark_rmse=$m.dark_region.rmse; white_rmse=$m.white_region.rmse
        edge_f1=$m.edges.f1; edge_precision=$m.edges.precision; edge_recall=$m.edges.recall
        highlight_ink=$m.unwanted_highlight_ink_fraction
        preview_path=$png; strokes_path=$json; report_path=$report
    }
}
if ($rows[0].strokes -ne $rows[1].strokes) {
    throw "The first prototype unexpectedly changed the accepted stroke budget"
}
# Inspect stroke geometry, rather than inferring a rotation from output F1.
$a=Get-Content -LiteralPath (Join-Path $destination "p2b1-strokes.json") -Raw | ConvertFrom-Json
$b=Get-Content -LiteralPath (Join-Path $destination "p3a-strokes.json") -Raw | ConvertFrom-Json
if ($a.strokes.Count -ne $b.strokes.Count) { throw "JSON stroke length mismatch" }
$changed=0
$metadataChanged=0
for ($i=0;$i -lt $a.strokes.Count;$i++) {
    $x=$a.strokes[$i]; $y=$b.strokes[$i]
    if ($x.x0 -ne $y.x0 -or $x.y0 -ne $y.y0 -or $x.x1 -ne $y.x1 -or $x.y1 -ne $y.y1) { $changed++ }
    if ($x.width -ne $y.width -or $x.opacity -ne $y.opacity) { $metadataChanged++ }
}
if ($metadataChanged -ne 0) { throw "P3-A.0 should not change width/opacity or contour metadata" }
$manifest=[pscustomobject]@{
    protocol_revision="p2c-v1"
    algorithm_phase="p3a0-proposal-direction-only"
    comparison_lane="same-source, same-binary, same-stroke-count; raster ink area NOT necessarily matched"
    source_name=[System.IO.Path]::GetFileName($source)
    rights_confirmed_by_operator=$true
    source_sha256=$hash; git_sha=$commit; seed=$Seed; max_side=$MaxSize
    geometrically_changed_strokes=$changed
    ink_metadata_changed_strokes=$metadataChanged
    fixtures=$rows
}
$out=Join-Path $destination "summary.json"
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $out -Encoding UTF8
$rows | Select-Object mode,strokes,path_length_px,tone_rmse,dark_rmse,edge_f1,highlight_ink | Format-Table -AutoSize
Write-Host "Changed stroke geometries: $changed. Kept count/ink metadata identical. Summary: $out"
Write-Host "Inspect images at 100% before claiming any directional improvement."
