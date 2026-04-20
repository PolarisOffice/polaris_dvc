#requires -Version 7
<#
.SYNOPSIS
    Probe matrix for DVC parity diagnostics.

.DESCRIPTION
    Runs a set of orthogonal-variable probes against ExampleWindows.exe
    to localize the 0xC0000005 crash we see with non-trivial rule specs.
    Each probe varies ONE input dimension (size / key / prelude / content)
    while holding the others constant, so the exit-code pattern across
    the matrix points at the real trigger instead of having to guess
    from code reading.

    This script is intentionally separate from `.github/workflows/
    dvc-parity.yml` so iterating on probes does NOT match the workflow
    file's `push.paths` filter — probe-only edits skip CI auto-trigger,
    and reruns reuse the prior artifact via
    `workflow_dispatch.inputs.reuse_artifact_from_run`.

.PARAMETER ExePath
    Path to ExampleWindows.exe. The caller (parity job) resolves this
    from the downloaded DVC binary artifact.

.PARAMETER OutDir
    Directory for probe outputs + synthetic spec files. Created if
    missing.

.PARAMETER FullSpec
    Path to `schemas/jsonFullSpec.json` (the canonical upstream-shaped
    rule spec that triggers the crash).

.PARAMETER RealSamplesDir
    Directory holding real-Hancom-Docs HWPX samples (`empty.hwpx`,
    `테스트.hwpx`, plus an optional >1 MB "complex" one).

.EXAMPLE
    pwsh scripts/parity-probes.ps1 `
        -ExePath (Get-ChildItem dvc-bin -Recurse -Filter ExampleWindows.exe | Select-Object -First 1).FullName `
        -OutDir 'testdata/golden/_dvc-output' `
        -FullSpec 'schemas/jsonFullSpec.json' `
        -RealSamplesDir 'testdata/real-samples'
#>

param(
    [Parameter(Mandatory)] [string] $ExePath,
    [Parameter(Mandatory)] [string] $OutDir,
    [Parameter(Mandatory)] [string] $FullSpec,
    [Parameter(Mandatory)] [string] $RealSamplesDir
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path $ExePath)) { throw "ExePath not found: $ExePath" }
if (-not (Test-Path $FullSpec)) { throw "FullSpec not found: $FullSpec" }
if (-not (Test-Path $RealSamplesDir)) { throw "RealSamplesDir not found: $RealSamplesDir" }

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

Write-Host "using: $ExePath"

# --- resolve real HWPX inputs ---
$empty   = Join-Path $RealSamplesDir 'empty.hwpx'
$korean  = Join-Path $RealSamplesDir '테스트.hwpx'
if (-not (Test-Path $empty))  { throw "missing $empty" }
if (-not (Test-Path $korean)) { throw "missing $korean" }

# Finds the ONE real-samples file >1 MB, if present (a complex
# real-world document — used to cross-check crash-invariance against
# HWPX size).
$complex = Get-ChildItem -Path $RealSamplesDir -Filter '*.hwpx' |
             Where-Object { $_.Length -gt 1MB } | Select-Object -First 1
if ($complex) {
    Write-Host "complex sample: $($complex.Name) ($($complex.Length) bytes)"
}

# --- synthetic spec files ---

# Trivial empty-rules spec (2 bytes) — known-graceful baseline.
$trivSpec = Join-Path $OutDir 'spec-trivial.json'
'{}' | Out-File -FilePath $trivSpec -Encoding ascii -NoNewline

# `{}` padded with ~27 KB whitespace (isolates raw size as variable).
$padSpec = Join-Path $OutDir 'spec-padded-27k.json'
('{' + (' ' * 27000) + '}') | Out-File -FilePath $padSpec -Encoding ascii -NoNewline

# Single KNOWN-registered key.
$charshapeSpec = Join-Path $OutDir 'spec-charshape-only.json'
'{"charshape":{}}' | Out-File -FilePath $charshapeSpec -Encoding ascii -NoNewline

# Single UNREGISTERED key.
$nonexistentSpec = Join-Path $OutDir 'spec-nonexistent.json'
'{"nonexistent":{}}' | Out-File -FilePath $nonexistentSpec -Encoding ascii -NoNewline

# Specific spec-level key that appears in jsonFullSpec but isn't in
# the CheckList dispatch map.
$objpropSpec = Join-Path $OutDir 'spec-objproperty.json'
'{"objproperty":{}}' | Out-File -FilePath $objpropSpec -Encoding ascii -NoNewline

# Full jsonFullSpec with the `[Json schema]\n` prelude stripped.
$noPreludeSpec = Join-Path $OutDir 'spec-noprelude.json'
$fullText = Get-Content $FullSpec -Raw
$noPrelude = $fullText.Substring($fullText.IndexOf('{'))
Set-Content $noPreludeSpec -Value $noPrelude -NoNewline

# Only the `charshape` subtree of jsonFullSpec — manual brace-match
# extraction. Can't use ConvertFrom-Json here: pwsh's parser rejects
# jsonFullSpec around `charshape.fontsize` line 9 (stricter than
# jsoncpp about duplicate keys / trailing commas).
$charshapeFullSpec = Join-Path $OutDir 'spec-charshape-subtree.json'
$csOk = $false
try {
    $keyIdx = $noPrelude.IndexOf('"charshape"')
    if ($keyIdx -ge 0) {
        $colonIdx = $noPrelude.IndexOf(':', $keyIdx)
        $oBraceIdx = $noPrelude.IndexOf('{', $colonIdx)
        $depth = 0
        $i = $oBraceIdx
        while ($i -lt $noPrelude.Length) {
            $c = $noPrelude[$i]
            if ($c -eq '{') { $depth++ }
            elseif ($c -eq '}') {
                $depth--
                if ($depth -eq 0) { break }
            }
            $i++
        }
        if ($depth -eq 0) {
            $block = $noPrelude.Substring($oBraceIdx, $i - $oBraceIdx + 1)
            $spec = '{"charshape":' + $block + '}'
            Set-Content $charshapeFullSpec -Value $spec -NoNewline
            $csOk = $true
            Write-Host "charshape subtree extracted: $((Get-Item $charshapeFullSpec).Length) bytes"
        }
    }
} catch {
    Write-Host "charshape subtree extraction failed: $_"
}

# --- probe runner ---
function Invoke-Probe {
    param(
        [Parameter(Mandatory)] [string] $Label,
        [Parameter(Mandatory)] [string] $Doc,
        [Parameter(Mandatory)] [string] $OutJson,
        [Parameter(Mandatory)] [string] $SpecPath
    )
    Write-Host "::group::$Label"
    Write-Host "  doc:  $Doc ($((Get-Item $Doc).Length) bytes)"
    Write-Host "  spec: $SpecPath ($((Get-Item $SpecPath).Length) bytes)"
    & $ExePath -j -o "--file=$OutJson" $SpecPath $Doc 2>&1 |
        ForEach-Object { Write-Host "  > $_" }
    Write-Host "  exit_code=$LASTEXITCODE"
    if (Test-Path $OutJson) {
        $len = (Get-Item $OutJson).Length
        Write-Host "  --- OUTPUT PRODUCED ($len bytes) ---"
        Get-Content $OutJson | Select-Object -First 5 |
            ForEach-Object { Write-Host "  $_" }
        if ($len -gt 200) { Write-Host "  ...(truncated)" }
    } else {
        Write-Host "  (no output file produced)"
    }
    Write-Host "::endgroup::"
}

# --- baseline re-runs (previously-observed behavior) ---
Invoke-Probe -Label 'probe 4a  baseline: empty.hwpx + `{}`    (expect: exit 0, no output)' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe4a.json') -SpecPath $trivSpec
Invoke-Probe -Label 'probe 5a  baseline: empty.hwpx + jsonFullSpec (expect: CRASH)' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe5a.json') -SpecPath $FullSpec

# --- Additional HWPX axis: complex real-world doc ---
if ($complex) {
    Invoke-Probe -Label 'probe 4d  complex.hwpx + `{}`        -> graceful hold on larger HWPX?' `
                 -Doc $complex.FullName -OutJson (Join-Path $OutDir 'probe4d.json') -SpecPath $trivSpec
    Invoke-Probe -Label 'probe 5c  complex.hwpx + jsonFullSpec -> crash invariant to HWPX size?' `
                 -Doc $complex.FullName -OutJson (Join-Path $OutDir 'probe5c.json') -SpecPath $FullSpec
}

# --- Variable A: size only ---
Invoke-Probe -Label 'probe 6a  `{}` padded to 27 KB whitespace -> isolates SIZE' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe6a.json') -SpecPath $padSpec

# --- Variable B: key identity ---
Invoke-Probe -Label 'probe 6b  {"charshape":{}}      -> known-registered key' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe6b.json') -SpecPath $charshapeSpec
Invoke-Probe -Label 'probe 6c  {"nonexistent":{}}    -> known-unregistered key' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe6c.json') -SpecPath $nonexistentSpec
Invoke-Probe -Label 'probe 6d  {"objproperty":{}}    -> spec key not in CheckList map' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe6d.json') -SpecPath $objpropSpec

# --- Variable C: prelude format ---
Invoke-Probe -Label 'probe 6e  jsonFullSpec no-prelude -> is `[Json schema]\n` the trigger?' `
             -Doc $empty -OutJson (Join-Path $OutDir 'probe6e.json') -SpecPath $noPreludeSpec

# --- Variable D: content subset ---
if ($csOk) {
    Invoke-Probe -Label 'probe 6f  charshape subtree only -> problem inside charshape subtree?' `
                 -Doc $empty -OutJson (Join-Path $OutDir 'probe6f.json') -SpecPath $charshapeFullSpec
} else {
    Write-Host "::group::probe 6f  SKIPPED (subtree extraction failed during setup)"
    Write-Host "::endgroup::"
}

Write-Host ""
Write-Host "=== MATRIX SUMMARY ==="
Write-Host "(read exit codes per group above; 0=graceful, -1073741819=AV)"
