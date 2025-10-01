<#
Runs .tong examples with feature parity to scripts/run_examples.sh

Usage examples:
  powershell -File scripts/run_examples.ps1              # top-level + rosetta
  powershell -File scripts/run_examples.ps1 -All         # top-level + modules + rosetta
  powershell -File scripts/run_examples.ps1 -Rosetta     # only rosetta
  powershell -File scripts/run_examples.ps1 -TongExe C:\path\to\tong.exe

Parameters:
  -TongExe   Optional explicit path to tong executable (overrides detection)
  -All       Include module examples (examples/modules/**) in addition to top-level + rosetta
  -Rosetta   Only run examples in examples/rosetta
  -Help      Show usage and exit

Environment:
  TONG  If set, used as the tong executable path (unless -TongExe provided)
#>
param(
    [string]$TongExe,
    [switch]$All,
    [switch]$Rosetta,
    [switch]$Sdl,
    [switch]$Help
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($Help) {
    Write-Host @'
Run TONG examples.

Parameters:
  -TongExe <path>  Use a specific tong executable
  -All             Include module examples (examples/modules/**)
  -Rosetta         Only rosetta examples (examples/rosetta/*.tong)
    -Sdl             Build (release) with SDL3 feature before running
  -Help            Show this help

Environment:
  TONG             Path to tong executable (fallback if -TongExe not supplied)

Exit codes:
  0 success, non-zero on first failing example.
'@
    exit 0
}

# Root directory (repo root assumed one level above script dir)
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$examplesDir = Join-Path $root "examples"

# Determine tong executable precedence: -TongExe > -Sdl build > $env:TONG > candidate probing > build
if (-not $TongExe -and $env:TONG) { $TongExe = $env:TONG }

# If -Sdl specified and no explicit -TongExe, build a release binary with sdl3 feature
if ($Sdl -and -not $TongExe) {
    Write-Host "[info] Building tong (release, sdl3 feature)..." -ForegroundColor Yellow
    Push-Location (Join-Path $root "rust/tong")
    try {
        cargo build --release --features sdl3 | Out-Null
    }
    finally { Pop-Location | Out-Null }
    $TongExe = (Join-Path $root "rust/tong/target/release/tong.exe")
}
elseif ($Sdl -and $TongExe) {
    Write-Host "[warn] -Sdl specified but -TongExe provided; assuming supplied binary already has SDL support." -ForegroundColor Yellow
}

if (-not $TongExe) {
    $candidates = @(
        (Join-Path $root "rust/tong/target/release/tong.exe"),
        (Join-Path $root "rust/tong/target/debug/tong.exe"),
        (Join-Path $env:USERPROFILE ".cargo/bin/tong.exe")
    )
    foreach ($c in $candidates) { if (Test-Path $c) { $TongExe = $c; break } }
}

if (-not $TongExe) {
    Write-Host "[info] Building tong (debug)..." -ForegroundColor Yellow
    Push-Location (Join-Path $root "rust/tong")
    try { cargo build | Out-Null }
    finally { Pop-Location | Out-Null }
    $TongExe = (Join-Path $root "rust/tong/target/debug/tong.exe")
}

if (-not (Test-Path $TongExe)) { throw "Could not find or build tong executable: $TongExe" }

Write-Host "[using] tong executable: $TongExe" -ForegroundColor Green

# Collect example files
$files = @()

function Add-Files($path) {
    if (Test-Path $path) {
        Get-ChildItem -Path $path -File -Filter *.tong | Sort-Object Name | ForEach-Object { $script:files += $_ }
    }
}

if ($Rosetta) {
    Add-Files (Join-Path $examplesDir "rosetta")
} else {
    # top-level (maxdepth 1) .tong files
    Get-ChildItem -Path $examplesDir -File -Filter *.tong | Sort-Object Name | ForEach-Object { $files += $_ }
    if ($All) {
        # modules (recursive)
        $modulesPath = Join-Path $examplesDir "modules"
        if (Test-Path $modulesPath) {
            Get-ChildItem -Path $modulesPath -Recurse -File -Filter *.tong | Sort-Object FullName | ForEach-Object { $files += $_ }
        }
    }
    # Always include rosetta unless restricted
    Add-Files (Join-Path $examplesDir "rosetta")
}

if ($files.Count -eq 0) {
    Write-Host "[warn] No .tong example files found." -ForegroundColor Yellow
    exit 0
}

foreach ($f in $files) {
    $rel = $f.FullName.Substring($examplesDir.Length) -replace '^[\\/]+',''
    Write-Host "`n=== Running $rel ===" -ForegroundColor Cyan
    & $TongExe $f.FullName
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[fail] Example failed: $rel (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

    Write-Host "`nAll selected examples completed successfully." -ForegroundColor Green
