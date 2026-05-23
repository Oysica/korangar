# Scans the korangar source for characters used in UI strings, merges them
# with characters dumped from iteminfo.lub, and regenerates the MSDF font
# atlas (NotoSansTC.png + NotoSansTC.csv.gz) so all referenced characters
# render in-game.
#
# Usage:
#   .\tools\update-font-charset.ps1

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path "$PSScriptRoot/.."
$fontDir  = Join-Path $repoRoot "korangar/archive/data/font"
$tool     = Join-Path $fontDir "msdf-atlas-gen.exe"
$ttf      = Join-Path $fontDir "NotoSansTC.ttf"
$csv      = Join-Path $fontDir "NotoSansTC.csv"
$png      = Join-Path $fontDir "NotoSansTC.png"
$gz       = "$csv.gz"
$charset  = Join-Path $fontDir "charset.txt"

$scanRoots = @(
    (Join-Path $repoRoot "korangar/src"),
    (Join-Path $repoRoot "korangar/archive/data/languages"),
    (Join-Path $repoRoot "korangar-interface/src")
)

Write-Host "Scanning source for non-ASCII characters..."

$chars = [System.Collections.Generic.HashSet[char]]::new()
# Match every character above ASCII printable that is not a surrogate or
# private-use codepoint. Covers CJK ideographs, Hangul, kana, full-width
# punctuation, arrows, and miscellaneous symbols in one shot.
$pattern = "[" + [char]0x00a1 + "-" + [char]0xd7ff + [char]0xe000 + "-" + [char]0xffef + "]"
$rx = [regex]$pattern

foreach ($root in $scanRoots) {
    if (-not (Test-Path $root)) { continue }
    Get-ChildItem -Path $root -Recurse -File -Include *.rs,*.ron,*.toml | ForEach-Object {
        $content = [System.IO.File]::ReadAllText($_.FullName, [System.Text.Encoding]::UTF8)
        foreach ($m in $rx.Matches($content)) {
            [void]$chars.Add([char]$m.Value[0])
        }
    }
}

# Keep whatever was already baked so we never lose previously supported glyphs.
if (Test-Path $charset) {
    $existing = [System.IO.File]::ReadAllText($charset, [System.Text.Encoding]::UTF8)
    foreach ($m in $rx.Matches($existing)) {
        [void]$chars.Add([char]$m.Value[0])
    }
}

# Merge characters dumped from iteminfo.lub by KORANGAR_DUMP_ITEM_CHARS=1.
$itemDump = Join-Path $repoRoot "iteminfo-charset.txt"
if (Test-Path $itemDump) {
    $dumped = [System.IO.File]::ReadAllText($itemDump, [System.Text.Encoding]::UTF8)
    foreach ($m in $rx.Matches($dumped)) {
        [void]$chars.Add([char]$m.Value[0])
    }
    Write-Host "Merged characters from iteminfo dump ($itemDump)"
}

$sorted = $chars | Sort-Object
$joined = -join $sorted

Write-Host "Found $($chars.Count) unique non-ASCII characters."

# msdf-atlas-gen charset file format: a quoted literal string and numeric
# ranges separated by commas. We always include the ASCII printable range
# [32, 126] for Latin glyphs, plus the full CJK Unified Ideographs block
# [0x4e00, 0x9fff] (~21,000 chars) so any Chinese character a user might
# type in chat renders without a regen.
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
$charsetContent = "`"$joined`", [32, 126], [0x4e00, 0x9fff]`n"
[System.IO.File]::WriteAllText($charset, $charsetContent, $utf8NoBom)
Write-Host "Wrote charset to $charset (explicit + full CJK Unified Ideographs)"

Write-Host "Running msdf-atlas-gen (this can take a few minutes for ~21K glyphs)..."
& $tool -charset $charset -pxrange 6 -size 32 -yorigin top -dimensions 8192 8192 -type msdf -format png -font $ttf -csv $csv -imageout $png
if ($LASTEXITCODE -ne 0) { throw "msdf-atlas-gen failed with exit code $LASTEXITCODE" }

if (Test-Path $gz) { Remove-Item $gz }
Add-Type -AssemblyName System.IO.Compression
$inStream  = [System.IO.File]::OpenRead($csv)
$outStream = [System.IO.File]::Create($gz)
$gzStream  = [System.IO.Compression.GZipStream]::new($outStream, [System.IO.Compression.CompressionLevel]::Optimal)
try {
    $inStream.CopyTo($gzStream)
}
finally {
    $gzStream.Dispose()
    $outStream.Dispose()
    $inStream.Dispose()
}
Remove-Item $csv

Write-Host "Done. Updated NotoSansTC.png and NotoSansTC.csv.gz."
