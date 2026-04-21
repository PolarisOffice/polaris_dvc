# Run upstream hancom-io/dvc against polaris' golden fixtures on a Windows
# machine and produce DVC.exe's expected outputs for parity comparison.
#
# Designed for a developer who has:
#   - Visual Studio 2022 Community (or Build Tools) with the
#     "Desktop development with C++" workload
#   - Git in PATH
#   - vcpkg (either pre-installed or this script will bootstrap it)
#
# Usage (from the repo root, PowerShell 7 or Windows PowerShell 5.1):
#
#   pwsh -File scripts\parity-windows.ps1                  # build + run, leave outputs under _dvc-output/
#   pwsh -File scripts\parity-windows.ps1 -WriteExpected   # overwrite testdata/golden/<case>/expected.json with DVC.exe output
#   pwsh -File scripts\parity-windows.ps1 -Only 02_fontsize_mismatch,11_table_border_type_mismatch
#
# Flow:
#   1. Clone `hancom-io/dvc` (and its OSS deps) under
#      `$env:TEMP\polaris-dvc-build\` unless `-UpstreamPath` is given.
#   2. Install jsoncpp via vcpkg (x86-windows), patching DVCModel.vcxproj
#      to point at the absolute install path.
#   3. Rewrite v141→v143 / `stdafx.h`→disabled PCH quirks in OWPML vcxprojs.
#   4. Build `Release|x86`.
#   5. Run `ExampleWindows.exe -j -a --file=<out> -t <spec> <doc>` for every
#      fixture under `testdata/golden/*/`.
#   6. Diff against committed `expected.json`, or overwrite per `-WriteExpected`.
#
# Captures what the GitHub Actions `dvc-parity.yml` workflow tries to do,
# but runs in a friendly environment where iteration is fast.

[CmdletBinding()]
param(
    # Local path to a checkout of hancom-io/dvc. Leave blank to auto-clone.
    [string] $UpstreamPath = "",

    # Git ref on hancom-io/dvc to build. Default: main.
    [string] $UpstreamRef = "main",

    # Overwrite `testdata/golden/<case>/expected.json` with DVC.exe's
    # output. Without this flag, outputs land under `_dvc-output/` next
    # to the case files and are diffed but not written back.
    [switch] $WriteExpected,

    # Restrict to the named cases (comma-separated or pipeline). Default:
    # process every directory under testdata/golden/ that holds a doc.hwpx.
    [string[]] $Only = @(),

    # Root of this polaris repo. Defaults to the parent of the scripts
    # directory holding this file.
    [string] $RepoRoot = ""
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

function Resolve-RepoRoot {
    param([string] $Explicit)
    if ($Explicit) { return (Resolve-Path $Explicit).Path }
    $here = Split-Path -Parent $PSCommandPath
    return (Resolve-Path (Join-Path $here '..')).Path
}

function Ensure-Tool {
    param([string] $Name, [string] $Hint)
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $cmd) {
        throw "$Name not found on PATH. $Hint"
    }
    return $cmd.Source
}

function Ensure-Vcpkg {
    [OutputType([string])]
    param()
    $root = $env:VCPKG_ROOT
    if (-not $root) { $root = $env:VCPKG_INSTALLATION_ROOT }
    if (-not $root -or -not (Test-Path (Join-Path $root 'vcpkg.exe'))) {
        $root = Join-Path $env:TEMP 'polaris-vcpkg'
        if (-not (Test-Path (Join-Path $root 'vcpkg.exe'))) {
            Write-Host "Bootstrapping vcpkg into $root..."
            if (-not (Test-Path $root)) {
                # Pipe to Out-Host so progress stays user-visible but doesn't
                # accumulate into the function's output pipeline (which
                # would otherwise make the return value an array of
                # every line git/bootstrap printed, breaking the caller).
                git clone https://github.com/microsoft/vcpkg $root | Out-Host
            }
            & (Join-Path $root 'bootstrap-vcpkg.bat') | Out-Host
        }
    }
    return $root
}

function Locate-Msbuild {
    # Prefer `vswhere` to find any VS install with MSBuild.
    $vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path $vswhere) {
        $install = & $vswhere -latest -products * `
            -requires Microsoft.Component.MSBuild `
            -property installationPath
        if ($install) {
            $msbuild = Join-Path $install 'MSBuild\Current\Bin\MSBuild.exe'
            if (Test-Path $msbuild) { return $msbuild }
        }
    }
    # Fall back to PATH.
    $msbuild = Get-Command MSBuild.exe -ErrorAction SilentlyContinue
    if ($msbuild) { return $msbuild.Source }
    throw "MSBuild not found. Install VS2022 (Community or Build Tools) with the 'Desktop development with C++' workload."
}

function Detect-WindowsSdk {
    $roots = @(
        'C:\Program Files (x86)\Windows Kits\10\Include',
        'C:\Program Files\Windows Kits\10\Include'
    )
    foreach ($r in $roots) {
        if (Test-Path $r) {
            $v = Get-ChildItem $r -Directory |
                Where-Object { $_.Name -match '^10\.' } |
                Sort-Object Name -Descending |
                Select-Object -First 1
            if ($v) { return $v.Name }
        }
    }
    throw "No Windows 10 SDK found."
}

function Clone-Upstream {
    param([string] $TargetDir, [string] $Ref)
    if (Test-Path (Join-Path $TargetDir '.git')) {
        Write-Host "Upstream already present at $TargetDir — fetching."
        git -C $TargetDir fetch --depth 1 origin $Ref
        git -C $TargetDir checkout FETCH_HEAD
    } else {
        New-Item -ItemType Directory -Force -Path (Split-Path $TargetDir) | Out-Null
        git clone --depth 1 --branch $Ref https://github.com/hancom-io/dvc.git $TargetDir
    }
}

function Fetch-OssDeps {
    param([string] $DvcDir)
    $oss = Join-Path $DvcDir 'opensource'
    New-Item -ItemType Directory -Force -Path $oss | Out-Null
    Push-Location $oss
    try {
        if (-not (Test-Path 'hwpx-owpml-model\.git')) {
            Write-Host "cloning hwpx-owpml-model"
            git clone --depth 1 https://github.com/hancom-io/hwpx-owpml-model.git
        }
        if (-not (Test-Path 'rapidjson\.git')) {
            Write-Host "cloning rapidjson (pinned)"
            git clone https://github.com/Tencent/rapidjson.git
            git -C rapidjson reset --hard 8261c1ddf43f10de00fd8c9a67811d1486b2c784
        }
        if (-not (Test-Path 'jsoncpp\.git')) {
            Write-Host "cloning jsoncpp (pinned)"
            git clone https://github.com/open-source-parsers/jsoncpp.git
            git -C jsoncpp reset --hard 8190e061bc2d95da37479a638aa2c9e483e58ec6
        }
        Push-Location jsoncpp
        try {
            if (-not (Test-Path 'dist\jsoncpp.cpp')) {
                Write-Host "amalgamating jsoncpp"
                python amalgamate.py
            }
        } finally { Pop-Location }
    } finally { Pop-Location }
}

function Patch-UpstreamVcxproj {
    param([string] $DvcDir, [string] $JsoncppInclude, [string] $JsoncppLib)

    # 1. Disable stdafx-flavored PCH only in OWPML projects. DVCModel
    #    itself uses pch.h (v143-default) and stays untouched.
    $owpmlDir = Join-Path $DvcDir 'opensource\hwpx-owpml-model'
    Get-ChildItem $owpmlDir -Recurse -Filter '*.vcxproj' | ForEach-Object {
        $text = Get-Content $_.FullName -Raw
        if ($text -match 'PrecompiledHeader') {
            $text = $text -replace '<PrecompiledHeader>Use</PrecompiledHeader>', '<PrecompiledHeader>NotUsing</PrecompiledHeader>'
            $text = $text -replace '<PrecompiledHeader>Create</PrecompiledHeader>', '<PrecompiledHeader>NotUsing</PrecompiledHeader>'
            Set-Content -Path $_.FullName -Value $text
            Write-Host "  patched PCH in $($_.Name)"
        }
    }

    # 2. Rewrite DVCModel.vcxproj's relative jsoncpp paths to absolute
    #    vcpkg install locations. Cover both Debug|Win32 and Release|Win32
    #    flavors by matching either `./opensource/jsoncpp/...` or the
    #    vcpkg-style `./opensource/vcpkg/packages/jsoncpp_x86-windows/...`.
    $vcxproj = Join-Path $DvcDir 'DVCModel.vcxproj'
    $text = Get-Content $vcxproj -Raw
    $include = $JsoncppInclude            # <pkg>/include
    $includeJson = Join-Path $include 'json'
    $bothInc = "$include;$includeJson"

    $text = $text -replace '\./opensource/vcpkg/packages/jsoncpp_x86-windows/include/json', $bothInc
    $text = $text -replace '\./opensource/vcpkg/packages/jsoncpp_x86-windows/lib', $JsoncppLib
    $text = $text -replace '\./opensource/jsoncpp/include/json', $bothInc
    $text = $text -replace '\./opensource/jsoncpp/json_git/lib/Release', $JsoncppLib
    Set-Content -Path $vcxproj -Value $text
    Write-Host "  patched include/lib paths in DVCModel.vcxproj"
}

function Build-Dvc {
    param([string] $DvcDir, [string] $Msbuild, [string] $Sdk)
    Push-Location $DvcDir
    try {
        Write-Host "msbuild DVCModel.sln Release|x86 (toolset v143, SDK $Sdk)"
        & $Msbuild DVCModel.sln `
            /p:Configuration=Release `
            /p:Platform=x86 `
            /p:PlatformToolset=v143 `
            /p:WindowsTargetPlatformVersion=$Sdk `
            /m /v:minimal
        if ($LASTEXITCODE -ne 0) { throw "msbuild failed (exit $LASTEXITCODE)" }
    } finally { Pop-Location }
}

function Find-Example-Exe {
    param([string] $DvcDir)
    $exe = Get-ChildItem $DvcDir -Recurse -Filter 'ExampleWindows.exe' |
        Where-Object { $_.FullName -match 'Release' } |
        Select-Object -First 1
    if (-not $exe) { throw "ExampleWindows.exe not found under $DvcDir" }
    return $exe.FullName
}

function Run-Cases {
    param(
        [string] $Exe,
        [string] $GoldenRoot,
        [string[]] $Filter,
        [switch] $WriteExpected
    )
    $results = @()
    $cases = Get-ChildItem $GoldenRoot -Directory | Where-Object {
        Test-Path (Join-Path $_.FullName 'doc.hwpx')
    }
    if ($Filter -and $Filter.Count -gt 0) {
        $cases = $cases | Where-Object { $Filter -contains $_.Name }
    }
    $outRoot = Join-Path $GoldenRoot '_dvc-output'
    New-Item -ItemType Directory -Force -Path $outRoot | Out-Null

    foreach ($c in $cases) {
        $name = $c.Name
        $doc = Join-Path $c.FullName 'doc.hwpx'
        $spec = Join-Path $c.FullName 'spec.json'
        $actualPath = if ($WriteExpected) { Join-Path $c.FullName 'expected.json' } else { Join-Path $outRoot "$name.json" }
        Write-Host "→ $name"
        & $Exe -j -a "--file=$actualPath" "-t" "$spec" "$doc" | Out-Null
        $status = if (Test-Path $actualPath) { 'ran' } else { 'no-output' }
        if ($WriteExpected) {
            $verdict = 'wrote-expected'
        } else {
            $expected = Join-Path $c.FullName 'expected.json'
            if (-not (Test-Path $actualPath)) {
                $verdict = 'missing'
            } elseif (-not (Test-Path $expected)) {
                $verdict = 'no-committed-expected'
            } else {
                $a = (Get-Content $actualPath -Raw).Trim()
                $e = (Get-Content $expected -Raw).Trim()
                $verdict = if ($a -eq $e) { 'MATCH' } else { 'DIFFER' }
            }
        }
        $results += [pscustomobject]@{ Case = $name; Result = $verdict }
    }
    return $results
}

# ---- Driver ----

$RepoRoot = Resolve-RepoRoot $RepoRoot
Write-Host "polaris repo root: $RepoRoot"

Ensure-Tool git "Install Git for Windows from https://git-scm.com/download/win"
$msbuild = Locate-Msbuild
$sdk = Detect-WindowsSdk
$vcpkgRoot = Ensure-Vcpkg
Write-Host "msbuild:        $msbuild"
Write-Host "Windows SDK:    $sdk"
Write-Host "vcpkg:          $vcpkgRoot"

# Install jsoncpp (idempotent — vcpkg no-ops when already present).
#
# On Windows ARM64 hosts, vcpkg's default host triplet is
# arm64-windows, which requires an arm64-native MSVC toolchain.
# Standard "Desktop development with C++" workloads don't include
# that; they ship amd64-native + amd64_x86 / amd64_arm64 cross
# compilers. Force the host triplet to x64-windows so vcpkg's
# internal helper ports (vcpkg-cmake-config, etc.) build with the
# always-available amd64 toolchain, which ARM64 Windows runs via
# the x86-64 emulation layer. The target triplet stays x86-windows
# to match DVCModel's Release|Win32 linkage.
$hostTriplet = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { 'x64-windows' } else { 'x64-windows' }
& (Join-Path $vcpkgRoot 'vcpkg.exe') install jsoncpp:x86-windows `
    --host-triplet=$hostTriplet --recurse
if ($LASTEXITCODE -ne 0) { throw "vcpkg install jsoncpp failed" }
$jsoncppPkg = Join-Path $vcpkgRoot 'packages\jsoncpp_x86-windows'
$jsoncppInclude = Join-Path $jsoncppPkg 'include'
$jsoncppLib = Join-Path $jsoncppPkg 'lib'
if (-not (Test-Path (Join-Path $jsoncppInclude 'json\json.h'))) {
    throw "jsoncpp install missing json.h under $jsoncppInclude"
}

# Prepare upstream DVC checkout.
if (-not $UpstreamPath) {
    $UpstreamPath = Join-Path $env:TEMP 'polaris-dvc-build\dvc'
    Clone-Upstream -TargetDir $UpstreamPath -Ref $UpstreamRef
}
Fetch-OssDeps -DvcDir $UpstreamPath
Patch-UpstreamVcxproj -DvcDir $UpstreamPath -JsoncppInclude $jsoncppInclude -JsoncppLib $jsoncppLib
Build-Dvc -DvcDir $UpstreamPath -Msbuild $msbuild -Sdk $sdk

$exe = Find-Example-Exe -DvcDir $UpstreamPath
Write-Host "ExampleWindows.exe: $exe"

$goldenRoot = Join-Path $RepoRoot 'testdata\golden'
if (-not (Test-Path $goldenRoot)) {
    throw "golden root not found: $goldenRoot"
}

$res = Run-Cases -Exe $exe -GoldenRoot $goldenRoot -Filter $Only -WriteExpected:$WriteExpected
Write-Host "--- Summary ---"
$res | Format-Table -AutoSize

if (-not $WriteExpected) {
    $differ = @($res | Where-Object { $_.Result -eq 'DIFFER' })
    $missing = @($res | Where-Object { $_.Result -eq 'missing' })
    Write-Host ""
    Write-Host "MATCH  : $(@($res | Where-Object { $_.Result -eq 'MATCH' }).Count)"
    Write-Host "DIFFER : $($differ.Count)"
    Write-Host "MISSING: $($missing.Count)"
    Write-Host "Outputs under: $goldenRoot\_dvc-output\"
}
